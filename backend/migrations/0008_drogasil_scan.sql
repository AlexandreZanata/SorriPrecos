CREATE TABLE drogasil_catalogue_scan (
    path TEXT PRIMARY KEY,
    page INTEGER NOT NULL DEFAULT 1 CHECK(page >= 1),
    item INTEGER NOT NULL DEFAULT 0 CHECK(item >= 0),
    last_attempt_at INTEGER NOT NULL DEFAULT 0
);
