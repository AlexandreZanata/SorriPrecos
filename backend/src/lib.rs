pub mod adapters;
pub mod aiqfome;
pub mod api;
pub mod catalog;
pub mod db;
pub mod normalize;

pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

pub mod vip;

mod photos;

pub mod pharmacy;
