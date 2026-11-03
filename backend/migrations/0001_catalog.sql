CREATE TABLE supermarkets (
 id TEXT PRIMARY KEY, name TEXT NOT NULL, website TEXT,
 integration_status TEXT NOT NULL DEFAULT 'pending' CHECK(integration_status IN ('pending','authorized','demo')),
 audit_class TEXT NOT NULL DEFAULT 'D' CHECK(audit_class IN ('A','B','C','D')),
 is_demo INTEGER NOT NULL DEFAULT 0 CHECK(is_demo IN (0,1))
);
CREATE TABLE branches (
 id TEXT PRIMARY KEY, supermarket_id TEXT NOT NULL REFERENCES supermarkets(id),
 name TEXT NOT NULL, address TEXT NOT NULL, city TEXT NOT NULL CHECK(city='Sorriso'),
 state TEXT NOT NULL DEFAULT 'MT' CHECK(state='MT'), source_id TEXT, active INTEGER NOT NULL DEFAULT 1,
 UNIQUE(id,supermarket_id)
);
CREATE TABLE products (
 id INTEGER PRIMARY KEY, identity_key TEXT NOT NULL UNIQUE, gtin TEXT,
 name TEXT NOT NULL, normalized_name TEXT NOT NULL, brand TEXT NOT NULL,
 quantity INTEGER NOT NULL CHECK(quantity>0), unit TEXT NOT NULL CHECK(unit IN ('g','ml','un')),
 presentation TEXT NOT NULL, category TEXT NOT NULL, is_demo INTEGER NOT NULL CHECK(is_demo IN (0,1))
);
CREATE UNIQUE INDEX products_gtin ON products(gtin,is_demo) WHERE gtin IS NOT NULL;
CREATE VIRTUAL TABLE products_fts USING fts5(name,brand,pack,tokenize='unicode61 remove_diacritics 2');
-- FTS is maintained by the importer: pack includes equivalent kg/g and l/ml tokens.
CREATE TABLE source_products (
 id INTEGER PRIMARY KEY, supermarket_id TEXT NOT NULL REFERENCES supermarkets(id),
 source_id TEXT NOT NULL, product_id INTEGER NOT NULL REFERENCES products(id),
 original_name TEXT NOT NULL, source_url TEXT,
 UNIQUE(supermarket_id,source_id), UNIQUE(id,supermarket_id)
);
CREATE TABLE offers (
 id INTEGER PRIMARY KEY, source_product_id INTEGER NOT NULL, product_id INTEGER NOT NULL REFERENCES products(id),
 supermarket_id TEXT NOT NULL, branch_id TEXT NOT NULL,
 price_cents INTEGER CHECK(price_cents>0 AND price_cents<=100000000),
 promotional_price_cents INTEGER CHECK(promotional_price_cents>0 AND promotional_price_cents<=price_cents),
 offer_type TEXT NOT NULL CHECK(offer_type IN ('regular','promotion','club','wholesale')),
 conditions TEXT, available INTEGER CHECK(available IN (0,1)), source_url TEXT,
 collected_at INTEGER NOT NULL, valid_until INTEGER,
 FOREIGN KEY(branch_id,supermarket_id) REFERENCES branches(id,supermarket_id),
 FOREIGN KEY(source_product_id,supermarket_id) REFERENCES source_products(id,supermarket_id),
 UNIQUE(source_product_id,branch_id)
);
CREATE INDEX offers_product_price ON offers(product_id,price_cents,branch_id);
CREATE INDEX offers_branch ON offers(branch_id,collected_at);
CREATE TABLE collection_runs (
 id INTEGER PRIMARY KEY, supermarket_id TEXT NOT NULL REFERENCES supermarkets(id),
 started_at INTEGER NOT NULL, finished_at INTEGER, status TEXT NOT NULL,
 imported INTEGER NOT NULL DEFAULT 0, error TEXT, requests INTEGER NOT NULL DEFAULT 0, authorization_reference TEXT
);
CREATE INDEX runs_source_time ON collection_runs(supermarket_id,started_at DESC);
CREATE TABLE price_history (
 id INTEGER PRIMARY KEY, offer_id INTEGER NOT NULL REFERENCES offers(id),
 price_cents INTEGER, promotional_price_cents INTEGER, collected_at INTEGER NOT NULL
);
CREATE INDEX history_retention ON price_history(collected_at);
CREATE TABLE source_cursors (
 supermarket_id TEXT PRIMARY KEY REFERENCES supermarkets(id), etag TEXT, last_modified TEXT, last_success INTEGER
);
