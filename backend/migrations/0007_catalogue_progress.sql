CREATE TABLE vtex_catalogue_scan (
    supermarket_id TEXT PRIMARY KEY REFERENCES supermarkets(id),
    checkpoint TEXT NOT NULL DEFAULT '{}',
    next_at INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE vip_catalogue_scan (
    branch_id TEXT PRIMARY KEY REFERENCES branches(id),
    checkpoint TEXT NOT NULL
);

CREATE TABLE storefront_cooldown (
    origin TEXT PRIMARY KEY,
    retry_at INTEGER NOT NULL
);

CREATE TABLE local_catalogue_schedule (
    id INTEGER PRIMARY KEY CHECK(id=1),
    last_source TEXT,
    next_at INTEGER NOT NULL DEFAULT 0,
    lease_until INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE vtex_offer_checks (
    branch_id TEXT NOT NULL REFERENCES branches(id),
    source_id TEXT NOT NULL,
    checked_at INTEGER NOT NULL,
    PRIMARY KEY (branch_id, source_id)
);
