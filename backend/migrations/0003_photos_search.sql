ALTER TABLE products ADD COLUMN photo_source_url TEXT;
CREATE TABLE product_photo_cache (
    product_id INTEGER PRIMARY KEY REFERENCES products(id) ON DELETE CASCADE,
    webp BLOB NOT NULL
);
CREATE VIRTUAL TABLE products_vocabulary USING fts5vocab(products_fts, 'col');
