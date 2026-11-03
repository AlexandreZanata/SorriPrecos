use anyhow::{Context, Result, bail, ensure};
use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};

pub fn text(s: &str) -> String {
    s.nfd()
        .filter(|c| !is_combining_mark(*c))
        .flat_map(char::to_lowercase)
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn valid_gtin(s: &str) -> bool {
    if ![8, 12, 13, 14].contains(&s.len())
        || !s.bytes().all(|c| c.is_ascii_digit())
        || s.bytes().all(|c| c == b'0')
    {
        return false;
    }
    let digits: Vec<u32> = s.bytes().map(|c| (c - b'0') as u32).collect();
    let sum: u32 = digits[..digits.len() - 1]
        .iter()
        .rev()
        .enumerate()
        .map(|(i, d)| d * if i % 2 == 0 { 3 } else { 1 })
        .sum();
    (10 - sum % 10) % 10 == digits[digits.len() - 1]
}

pub fn package(quantity: i64, unit: &str) -> Result<(i64, String)> {
    let (factor, u) = match text(unit).as_str() {
        "kg" => (1000, "g"),
        "g" => (1, "g"),
        "l" => (1000, "ml"),
        "ml" => (1, "ml"),
        "un" => (1, "un"),
        _ => bail!("unidade inválida"),
    };
    let q = quantity
        .checked_mul(factor)
        .filter(|n| (1..=1_000_000).contains(n))
        .ok_or_else(|| anyhow::anyhow!("quantidade inválida"))?;
    Ok((q, u.into()))
}

// Conservative fallback: variant words and presentation remain part of identity.
pub fn identity(
    name: &str,
    brand: &str,
    q: i64,
    unit: &str,
    presentation: &str,
    gtin: Option<&str>,
    demo: bool,
) -> String {
    if let Some(g) = gtin {
        return format!("{}:gtin:{}", demo, g.trim_start_matches('0'));
    }
    format!(
        "{}:name:{}|{}|{}{}|{}",
        demo,
        text(name),
        text(brand),
        q,
        unit,
        text(presentation)
    )
}

pub fn fts_query(q: &str) -> String {
    text(q)
        .split_whitespace()
        .take(8)
        .map(|s| format!("\"{s}\"*"))
        .collect::<Vec<_>>()
        .join(" AND ")
}

pub fn pack_tokens(q: i64, u: &str) -> String {
    let mut s = format!("{q}{u} {q} {u}");
    if q % 1000 == 0 && ["g", "ml"].contains(&u) {
        let alias = if u == "g" { "kg" } else { "l" };
        s.push_str(&format!(" {}{} {} {}", q / 1000, alias, q / 1000, alias));
    }
    s
}

// Aliases are for discovery. Product identity and price aggregation keep their own keys.
pub fn search_terms(query: &str) -> (String, Option<(i64, String)>) {
    use std::sync::LazyLock;
    static SIZE: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r"(?i)\s+(\d+(?:[.,]\d+)?)\s*(kg|quilos?|g|gramas?|ml|l|litros?)\s*$")
            .unwrap()
    });
    const ALIASES: &[&[&str]] = &[
        &["mandioca", "aipim", "macaxeira"],
        &["mucarela", "mussarela", "mozarela"],
        &["acucar", "assucar"],
        &["picanha gril", "picanha grill"],
        &["catchup", "ketchup"],
    ];
    let query = query.trim();
    let mut words = query;
    let mut pack = None;
    if let Some(size) = SIZE.captures(query) {
        let matched = size.get(0).unwrap();
        let prefix = query[..matched.start()].trim_end();
        let prefix_end = prefix.chars().last();
        let is_multipack = prefix.ends_with('×')
            || prefix.ends_with(" x")
            || prefix.ends_with(" X")
            || (prefix.ends_with(['x', 'X'])
                && prefix
                    .chars()
                    .rev()
                    .nth(1)
                    .is_some_and(|c| c.is_ascii_digit()));
        if !is_multipack && prefix_end.is_some() {
            let unit = size[2].to_ascii_lowercase();
            let (factor, base) = match unit.as_str() {
                "kg" | "quilo" | "quilos" => (1000, "g"),
                "l" | "litro" | "litros" => (1000, "ml"),
                "ml" => (1, "ml"),
                _ => (1, "g"),
            };
            if let Ok(quantity) = scaled(&size[1], factor)
                && quantity <= 1_000_000
            {
                pack = Some((quantity, base.into()));
                words = prefix;
            }
        }
    }
    let normalized = text(words);
    let mut tokens = normalized.split_whitespace().take(8).peekable();
    let mut terms = Vec::new();
    while let Some(word) = tokens.next() {
        // A multiword alias matches as a phrase, keeping its product context.
        let phrase = tokens.peek().map(|next| format!("{word} {next}"));
        let aliases = phrase.as_ref().and_then(|phrase| {
            ALIASES
                .iter()
                .find(|group| group.contains(&phrase.as_str()))
        });
        if let Some(aliases) = aliases {
            tokens.next();
            terms.push(alias_query(aliases));
        } else if let Some(aliases) = ALIASES.iter().find(|group| group.contains(&word)) {
            terms.push(alias_query(aliases));
        } else {
            terms.push(format!(r#""{word}"*"#));
        }
    }
    (terms.join(" AND "), pack)
}

fn alias_query(aliases: &[&str]) -> String {
    // Full-token aliases avoid expanding "mandioca" to unrelated longer words.
    let terms: Vec<_> = aliases.iter().map(|word| format!(r#""{word}""#)).collect();
    format!("({})", terms.join(" OR "))
}

pub(crate) fn scaled(text: &str, factor: i64) -> Result<i64> {
    let s = text.trim().replace(',', ".");
    let mut p = s.split('.');
    let digits = p.next().context("número ausente")?;
    ensure!(
        !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()),
        "decimal inválido"
    );
    let whole: i64 = digits.parse()?;
    let frac = p.next().unwrap_or("");
    ensure!(
        p.next().is_none() && frac.len() <= 6 && frac.bytes().all(|b| b.is_ascii_digit()),
        "decimal inválido"
    );
    let denominator = 10i64.pow(frac.len() as u32);
    let numerator = whole
        .checked_mul(denominator)
        .and_then(|v| v.checked_add(frac.parse::<i64>().unwrap_or(0)))
        .context("decimal excede limite")?;
    let scaled = numerator
        .checked_mul(factor)
        .context("decimal excede limite")?;
    ensure!(
        scaled % denominator == 0 && scaled > 0,
        "decimal não representável"
    );
    Ok(scaled / denominator)
}
