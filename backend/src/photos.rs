use anyhow::{Result, ensure};
use image::{ImageFormat, ImageReader};
use reqwest::Url;
use sqlx::SqlitePool;
use std::{io::Cursor, sync::LazyLock, time::Duration};
use tokio::sync::{Mutex, Semaphore};

const MAX_DOWNLOAD: usize = 5 * 1024 * 1024;
static PRODUCT_LOCKS: [Mutex<()>; 64] = [const { Mutex::const_new(()) }; 64];
static SLOTS: Semaphore = Semaphore::const_new(4);
static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(5))
        .build()
        .expect("cliente HTTP de fotos")
});

pub(crate) fn trusted_url(value: &str) -> Option<Url> {
    if value.len() > 512 {
        return None;
    }
    let url = Url::parse(value).ok()?;
    let prefix = match url.host_str()? {
        "www.therapeuticafarmacia.com.br" => "/storage/empresas/01780384000175/produtos/",
        "paguemenos.vteximg.com.br" | "drogariasp.vteximg.com.br" => "/arquivos/ids/",
        "products-images-edelivery-sp.s3.sa-east-1.amazonaws.com"
        | "ecomm-products-images-sp.s3.sa-east-1.amazonaws.com" => "/",
        "product-data.raiadrogasil.io" => "/images/",
        "files.aiqfome.com" => "/itens/",
        "produto-assets-vipcommerce-com-br.br-se1.magaluobjects.com" => "/250x250/",
        "pedido.egaosushi.com.br" => "/api/fotos/",
        "alloydeliveryimages.ligafiles.com.br" => "/item_images/",
        _ => return None,
    };
    let extension = url.path().rsplit_once('.')?.1.to_ascii_lowercase();
    (url.scheme() == "https"
        && url.port().is_none()
        && url.username().is_empty()
        && url.password().is_none()
        && (url.query().is_none()
            || (matches!(
                url.host_str(),
                Some("paguemenos.vteximg.com.br" | "drogariasp.vteximg.com.br")
            ) && url
                .query()
                .and_then(|query| query.strip_prefix("v="))
                .is_some_and(|value| {
                    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit())
                })))
        && url.fragment().is_none()
        && url.path().starts_with(prefix)
        && matches!(extension.as_str(), "jpg" | "jpeg" | "png" | "webp"))
    .then_some(url)
}

fn thumbnail(bytes: Vec<u8>) -> Result<Vec<u8>> {
    let mut reader = ImageReader::new(Cursor::new(bytes)).with_guessed_format()?;
    ensure!(
        matches!(
            reader.format(),
            Some(ImageFormat::Jpeg | ImageFormat::Png | ImageFormat::WebP)
        ),
        "formato de imagem inválido"
    );
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(4096);
    limits.max_image_height = Some(4096);
    limits.max_alloc = Some(96 * 1024 * 1024);
    reader.limits(limits);
    let mut output = Cursor::new(Vec::new());
    reader
        .decode()?
        .thumbnail(384, 384)
        .write_to(&mut output, ImageFormat::WebP)?;
    Ok(output.into_inner())
}

pub(crate) async fn load(pool: &SqlitePool, id: i64) -> Result<Option<Vec<u8>>> {
    let _product = tokio::time::timeout(
        Duration::from_secs(10),
        PRODUCT_LOCKS[id as usize % PRODUCT_LOCKS.len()].lock(),
    )
    .await?;
    let row = sqlx::query_as::<_, (Option<String>, i64, Option<Vec<u8>>, Option<i64>)>(
        "SELECT p.photo_source_url,p.photo_revision,c.webp,r.retry_at FROM products p
         LEFT JOIN product_photo_cache c ON c.product_id=p.id
         LEFT JOIN product_photo_retry r ON r.product_id=p.id WHERE p.id=?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    let Some((source, revision, cached, retry_at)) = row else {
        return Ok(None);
    };
    if cached.is_some() {
        return Ok(cached);
    }
    let Some(url) = source.as_deref().and_then(trusted_url) else {
        return Ok(None);
    };
    ensure!(
        retry_at.is_none_or(|at| at <= crate::now()),
        "foto aguardando nova tentativa"
    );
    let _permit = tokio::time::timeout(Duration::from_secs(10), SLOTS.acquire()).await??;
    let webp = match fetch(url).await {
        Ok(webp) => webp,
        Err(error) => {
            // Condition both writes on the revision in case an import replaced the source.
            sqlx::query(
                "INSERT INTO product_photo_retry(product_id,failures,retry_at)
                 SELECT id,1,?+60 FROM products WHERE id=? AND photo_revision=?
                 ON CONFLICT(product_id) DO UPDATE SET failures=MIN(failures+1,6),
                 retry_at=?+MIN(3600,60*(1<<failures))",
            )
            .bind(crate::now())
            .bind(id)
            .bind(revision)
            .bind(crate::now())
            .execute(pool)
            .await?;
            return Err(error);
        }
    };
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    let inserted = sqlx::query(
        "INSERT OR IGNORE INTO product_photo_cache(product_id,webp)
         SELECT id,? FROM products WHERE id=? AND photo_revision=?",
    )
    .bind(&webp)
    .bind(id)
    .bind(revision)
    .execute(&mut *tx)
    .await?;
    if inserted.rows_affected() == 0 {
        // A concurrent import invalidated this fetch or another process filled the cache.
        let current = sqlx::query_scalar("SELECT webp FROM product_photo_cache WHERE product_id=?")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;
        tx.commit().await?;
        return Ok(current);
    }
    sqlx::query("DELETE FROM product_photo_retry WHERE product_id=?")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(Some(webp))
}

async fn fetch(url: Url) -> Result<Vec<u8>> {
    let mut response = CLIENT.get(url).send().await?.error_for_status()?;
    ensure!(response.status().is_success(), "resposta de foto inválida");
    ensure!(
        response.content_length().unwrap_or(0) <= MAX_DOWNLOAD as u64,
        "foto acima do limite"
    );
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        ensure!(
            bytes.len() + chunk.len() <= MAX_DOWNLOAD,
            "foto acima do limite"
        );
        bytes.extend_from_slice(&chunk);
    }
    tokio::task::spawn_blocking(move || thumbnail(bytes)).await?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn photo_urls_are_limited_to_the_original_public_paths() {
        assert!(trusted_url("https://files.aiqfome.com/itens/store/item.jpg").is_some());
        assert!(trusted_url("https://files.aiqfome.com@localhost/itens/item.jpg").is_none());
        assert!(trusted_url("https://files.aiqfome.com/restaurantes/avatar/item.jpg").is_none());
        assert!(trusted_url("https://files.aiqfome.com/itens/store/item.svg").is_none());
    }

    #[test]
    fn encodes_a_bounded_webp() {
        let mut original = Cursor::new(Vec::new());
        image::DynamicImage::new_rgb8(800, 400)
            .write_to(&mut original, ImageFormat::Png)
            .unwrap();
        let output = thumbnail(original.into_inner()).unwrap();
        let decoded = image::load_from_memory_with_format(&output, ImageFormat::WebP).unwrap();
        assert_eq!((decoded.width(), decoded.height()), (384, 192));
    }
}
