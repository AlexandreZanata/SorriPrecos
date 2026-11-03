ALTER TABLE supermarkets ADD COLUMN sector TEXT NOT NULL DEFAULT 'market' CHECK(sector IN ('market', 'pharmacy'));
ALTER TABLE products ADD COLUMN sector TEXT NOT NULL DEFAULT 'market' CHECK(sector IN ('market', 'pharmacy'));
DROP INDEX products_gtin;
CREATE UNIQUE INDEX products_gtin ON products(gtin,is_demo,sector) WHERE gtin IS NOT NULL;
CREATE INDEX products_sector_demo_id ON products(sector,is_demo,id);
CREATE TABLE pharmacy_scan (
    supermarket_id TEXT PRIMARY KEY REFERENCES supermarkets(id),
    next_page INTEGER NOT NULL CHECK(next_page >= 0),
    retry_at INTEGER
);
