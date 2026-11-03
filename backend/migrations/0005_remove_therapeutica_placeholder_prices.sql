DELETE FROM price_history
WHERE offer_id IN (
    SELECT id FROM offers WHERE supermarket_id = 'therapeutica-farmacia' AND price_cents <= 100
);

DELETE FROM offers
WHERE supermarket_id = 'therapeutica-farmacia' AND price_cents <= 100;
