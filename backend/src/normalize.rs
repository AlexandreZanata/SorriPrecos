use anyhow::{Result, bail};
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
