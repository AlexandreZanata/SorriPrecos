CREATE INDEX IF NOT EXISTS offers_supermarket_collected ON offers(supermarket_id, collected_at);
CREATE INDEX IF NOT EXISTS offers_supermarket_product ON offers(supermarket_id, product_id);
CREATE INDEX IF NOT EXISTS branches_supermarket_active ON branches(supermarket_id, active);
