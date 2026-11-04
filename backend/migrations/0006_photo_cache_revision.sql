ALTER TABLE products ADD COLUMN photo_revision INTEGER NOT NULL DEFAULT 0;
ALTER TABLE products ADD COLUMN photo_owner_id INTEGER REFERENCES source_products(id);

-- A legacy photo with a single retail SKU has an unambiguous owner.
UPDATE products SET photo_owner_id = (
    SELECT CASE WHEN COUNT(*) = 1 THEN MIN(id) END
    FROM source_products
    WHERE product_id = products.id AND source_kind = 'retail'
) WHERE photo_source_url IS NOT NULL;

CREATE TABLE product_photo_retry (
    product_id INTEGER PRIMARY KEY REFERENCES products(id) ON DELETE CASCADE,
    failures INTEGER NOT NULL,
    retry_at INTEGER NOT NULL
);

CREATE TRIGGER invalidate_product_photo
AFTER UPDATE OF photo_source_url ON products
WHEN OLD.photo_source_url IS NOT NEW.photo_source_url
BEGIN
    UPDATE products SET photo_revision = OLD.photo_revision + 1 WHERE id = NEW.id;
    DELETE FROM product_photo_cache WHERE product_id = NEW.id;
    DELETE FROM product_photo_retry WHERE product_id = NEW.id;
END;
