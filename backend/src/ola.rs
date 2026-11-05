//! Decode the public Nuxt catalogue embedded in an Ola storefront.
use crate::normalize;
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

fn resolve<'a>(table: &'a [Value], reference: &Value) -> Result<&'a Value> {
    let mut index = reference.as_u64().context("referência Nuxt inválida")? as usize;
    for _ in 0..8 {
        let value = table.get(index).context("referência Nuxt fora da tabela")?;
        if let Some(wrapper) = value.as_array()
            && wrapper.len() == 2
            && matches!(
                wrapper[0].as_str(),
                Some("Ref" | "ShallowRef" | "Reactive" | "ShallowReactive")
            )
        {
            index = wrapper[1].as_u64().context("referência Nuxt inválida")? as usize;
            continue;
        }
        return Ok(value);
    }
    anyhow::bail!("referência Nuxt circular")
}

fn field<'a>(table: &'a [Value], value: &Value, name: &str) -> Result<&'a Value> {
    resolve(
        table,
        value
            .get(name)
            .with_context(|| format!("campo Nuxt ausente: {name}"))?,
    )
}

pub(crate) fn parse(body: &[u8], id: &str, address: &str, categories: &[String]) -> Result<Value> {
    let html = std::str::from_utf8(body)?;
    let script = html
        .split_once("id=\"__NUXT_DATA__\">")
        .context("dados Nuxt ausentes")?
        .1
        .split_once("</script>")
        .context("dados Nuxt incompletos")?
        .0;
    let data: Value = serde_json::from_str(script)?;
    let table = data.as_array().context("tabela Nuxt inválida")?;
    let root = resolve(table, &Value::from(0))?;
    let pinia = field(table, root, "pinia")?;
    let company_store = field(table, pinia, "company")?;
    let company = field(table, company_store, "company")?;
    ensure!(
        field(table, company, "id")?.as_str() == Some(id),
        "identidade da vitrine divergente"
    );
    let found_address = field(table, company, "address")?
        .as_str()
        .context("endereço ausente")?;
    ensure!(
        normalize::text(found_address) == normalize::text(address)
            && normalize::text(found_address).contains("sorriso mt"),
        "endereço da vitrine divergente de Sorriso-MT"
    );
    let store = field(table, pinia, "productsCategories")?;
    let sections = field(table, store, "originalProductsCategories")?
        .as_array()
        .context("categorias inválidas")?;
    let mut out_categories = Vec::new();
    let mut out_items = Vec::new();
    let mut total_categories = 0;
    let mut selected_categories = 0;
    let mut total_items = 0;
    let mut selected_items = 0;
    let mut total_variants = 0;
    let mut selected_variants = 0;
    let mut skipped_categories = Vec::new();
    for section_ref in sections {
        let section = resolve(table, section_ref)?;
        let name = field(table, section, "name")?
            .as_str()
            .context("categoria sem nome")?;
        let products = field(table, section, "products")?
            .as_array()
            .context("itens inválidos")?;
        let mut section_variants = 0;
        for item_ref in products {
            let item = resolve(table, item_ref)?;
            section_variants += field(table, item, "product_variants")?
                .as_array()
                .context("variantes inválidas")?
                .len();
        }
        total_categories += 1;
        total_items += products.len();
        total_variants += section_variants;
        if !categories.iter().any(|c| *c == normalize::text(name)) {
            skipped_categories.push(name.to_owned());
            continue;
        }
        selected_categories += 1;
        selected_items += products.len();
        selected_variants += section_variants;
        let section_id = field(table, section, "id")?
            .as_str()
            .context("categoria sem ID")?;
        let visible = field(table, section, "visible")?
            .as_bool()
            .context("categoria sem visibilidade")?;
        out_categories.push(json!({"slug":section_id,"nome":name}));
        for item_ref in products {
            let item = resolve(table, item_ref)?;
            let title = field(table, item, "name")?
                .as_str()
                .context("item sem nome")?;
            let shown = field(table, item, "visible")?
                .as_bool()
                .context("item sem visibilidade")?;
            let stock_enabled = field(table, item, "stock_enabled")?
                .as_bool()
                .context("estoque sem estado")?;
            let modifiers = field(table, item, "modifier_categories_count")?
                .as_u64()
                .context("opções sem estado")?;
            for variant_ref in field(table, item, "product_variants")?
                .as_array()
                .context("variantes inválidas")?
            {
                let variant = resolve(table, variant_ref)?;
                let variant_id = field(table, variant, "id")?
                    .as_str()
                    .context("variante sem ID")?;
                let stock = field(table, variant, "stock")?.as_i64();
                let available = if !visible || !shown || (stock_enabled && stock == Some(0)) {
                    Some(false)
                } else {
                    None
                };
                out_items.push(json!({"id":variant_id,"nome":title,"categoria":section_id,"disponivel":available,
                    "opcoes":[{"rotulo":field(table, variant, "name")?, "preco":field(table, variant, "price")?, "dependent_price":modifiers>0}]}));
            }
        }
    }
    ensure!(
        !out_categories.is_empty(),
        "categorias configuradas não encontradas"
    );
    Ok(json!({
        "categorias":out_categories,
        "itens":out_items,
        "cobertura":{
            "total_categories":total_categories,
            "selected_categories":selected_categories,
            "total_items":total_items,
            "selected_items":selected_items,
            "total_variants":total_variants,
            "selected_variants":selected_variants,
            "skipped_categories":skipped_categories
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_store_and_keeps_stock_unknown_when_not_controlled() {
        fn put(table: &mut Vec<Value>, value: Value) -> usize {
            table.push(value);
            table.len() - 1
        }
        let mut t = vec![Value::Null];
        let id = put(&mut t, json!("store-id"));
        let address = put(&mut t, json!("Rua A, 1 - Sorriso - MT"));
        let company = put(&mut t, json!({"id":id,"address":address}));
        let store = put(&mut t, json!({"company":company}));
        let title = put(&mut t, json!("Bolo"));
        let variant_id = put(&mut t, json!("variant-id"));
        let null_index = put(&mut t, Value::Null);
        let price = put(&mut t, json!(12.50));
        let stock = put(&mut t, json!(100));
        let variant = put(
            &mut t,
            json!({"id":variant_id,"name":null_index,"price":price,"stock":stock}),
        );
        let variants = put(&mut t, json!([variant]));
        let yes = put(&mut t, json!(true));
        let no = put(&mut t, json!(false));
        let zero = put(&mut t, json!(0));
        let one = put(&mut t, json!(1));
        let item = put(
            &mut t,
            json!({"name":title,"visible":yes,"stock_enabled":no,"modifier_categories_count":zero,"product_variants":variants}),
        );
        let items = put(&mut t, json!([item]));
        let category_name = put(&mut t, json!("Bolos"));
        let category_id = put(&mut t, json!("category-id"));
        let category = put(
            &mut t,
            json!({"name":category_name,"id":category_id,"visible":yes,"products":items}),
        );
        let categories = put(&mut t, json!([category]));
        let category_store = put(&mut t, json!({"originalProductsCategories":categories}));
        let pinia = put(
            &mut t,
            json!({"company":store,"productsCategories":category_store}),
        );
        let root = put(&mut t, json!({"pinia":pinia}));
        t[0] = json!(["ShallowReactive", root]);
        let html = format!("<script id=\"__NUXT_DATA__\">{}</script>", json!(t.clone()));
        let result = parse(
            html.as_bytes(),
            "store-id",
            "Rua A, 1 - Sorriso - MT",
            &["bolos".into()],
        )
        .unwrap();
        assert_eq!(result["itens"][0]["id"], "variant-id");
        assert!(result["itens"][0]["disponivel"].is_null());
        assert_eq!(result["itens"][0]["opcoes"][0]["preco"], 12.5);
        t[item]["modifier_categories_count"] = json!(one);
        let with_modifiers = format!("<script id=\"__NUXT_DATA__\">{}</script>", json!(t));
        let result = parse(
            with_modifiers.as_bytes(),
            "store-id",
            "Rua A, 1 - Sorriso - MT",
            &["bolos".into()],
        )
        .unwrap();
        assert_eq!(result["itens"][0]["opcoes"][0]["dependent_price"], true);
        assert!(
            parse(
                html.as_bytes(),
                "other",
                "Rua A, 1 - Sorriso - MT",
                &["bolos".into()]
            )
            .is_err()
        );
        assert!(
            parse(
                html.as_bytes(),
                "store-id",
                "Rua A, 1 - Sinop - MT",
                &["bolos".into()]
            )
            .is_err()
        );
    }
}
