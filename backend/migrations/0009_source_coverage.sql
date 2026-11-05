CREATE TABLE source_coverage (
 supermarket_id TEXT NOT NULL,
 branch_id TEXT NOT NULL,
 adapter TEXT NOT NULL,
 observed_at INTEGER NOT NULL,
 total_categories INTEGER NOT NULL,
 selected_categories INTEGER NOT NULL,
 total_items INTEGER NOT NULL,
 selected_items INTEGER NOT NULL,
 total_variants INTEGER NOT NULL,
 selected_variants INTEGER NOT NULL,
 skipped_categories TEXT NOT NULL,
 PRIMARY KEY (supermarket_id, branch_id)
);
