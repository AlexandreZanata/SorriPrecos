-- Keep official survey observations separate from a retailer's live catalogue.
ALTER TABLE source_products ADD COLUMN source_kind TEXT NOT NULL DEFAULT 'retail'
 CHECK(source_kind IN ('retail','public_report'));
UPDATE source_products SET source_kind='public_report'
 WHERE source_id LIKE 'procon-2026-%'
 AND source_url LIKE 'https://www.procon.mt.gov.br/documents/d/procon/tabela-produtos-analisados-pascoa-final#page=%';
CREATE INDEX source_products_origin ON source_products(source_kind,product_id);
-- All public catalogue rows are retained, including rows pending packaging/GTIN review.
CREATE TABLE source_catalog_items (
 supermarket_id TEXT NOT NULL REFERENCES supermarkets(id), branch_id TEXT NOT NULL,
 source_id TEXT NOT NULL, raw_json TEXT NOT NULL, observed_at INTEGER NOT NULL,
 run_id INTEGER REFERENCES collection_runs(id) ON DELETE SET NULL, normalized INTEGER NOT NULL,
 reason TEXT, PRIMARY KEY(supermarket_id,branch_id,source_id),
 FOREIGN KEY(branch_id,supermarket_id) REFERENCES branches(id,supermarket_id)
);
CREATE INDEX source_catalog_run ON source_catalog_items(supermarket_id,run_id);
CREATE TABLE catalogue_collectors (
 supermarket_id TEXT PRIMARY KEY REFERENCES supermarkets(id), adapter TEXT NOT NULL,
 interval_seconds INTEGER NOT NULL, status TEXT NOT NULL DEFAULT 'pending',
 last_started_at INTEGER,last_success_at INTEGER,last_finished_at INTEGER,
 last_error TEXT,requests INTEGER NOT NULL DEFAULT 0,raw_items INTEGER NOT NULL DEFAULT 0,
 normalized_items INTEGER NOT NULL DEFAULT 0,scope_rejected INTEGER NOT NULL DEFAULT 0
);
