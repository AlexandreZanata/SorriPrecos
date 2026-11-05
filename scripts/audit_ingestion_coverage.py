#!/usr/bin/env python3
"""Summarize persisted ingestion coverage and long-running cursors from SQLite."""
import argparse
import json
import sqlite3
from pathlib import Path

from audit_source_coverage import EXCLUDED, norm


def load_json(value):
    try:
        return json.loads(value)
    except (TypeError, json.JSONDecodeError):
        return None


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--database", type=Path, default=Path("data/sorriprecos.db"))
    parser.add_argument("--output", type=Path, default=Path("artifacts/ingestion-coverage.json"))
    args = parser.parse_args()
    connection = sqlite3.connect(args.database)
    connection.row_factory = sqlite3.Row
    collectors = [dict(row) for row in connection.execute(
        "SELECT supermarket_id,adapter,status,last_started_at,last_success_at,last_finished_at,last_error,requests,raw_items,normalized_items FROM catalogue_collectors ORDER BY supermarket_id"
    )]
    coverage = []
    unexpected = []
    for row in connection.execute("SELECT * FROM source_coverage ORDER BY supermarket_id"):
        item = dict(row)
        item["skipped_categories"] = load_json(item["skipped_categories"]) or []
        coverage.append(item)
        for name in item["skipped_categories"]:
            if norm(name) not in EXCLUDED:
                unexpected.append({"supermarket_id": item["supermarket_id"], "category": name})
    vip = [dict(row, checkpoint=load_json(row["checkpoint"])) for row in connection.execute(
        "SELECT branch_id,checkpoint FROM vip_catalogue_scan ORDER BY branch_id"
    )]
    vtex = [dict(row, checkpoint=load_json(row["checkpoint"])) for row in connection.execute(
        "SELECT supermarket_id,checkpoint,next_at FROM vtex_catalogue_scan ORDER BY supermarket_id"
    )]
    pharmacy = [dict(row) for row in connection.execute(
        "SELECT supermarket_id,next_page,retry_at FROM pharmacy_scan ORDER BY supermarket_id"
    )]
    offers = {row[0]: {"offers": row[1], "available": row[2]} for row in connection.execute(
        "SELECT supermarket_id,COUNT(*),SUM(available=1) FROM offers GROUP BY supermarket_id"
    )}
    report = {
        "collectors": collectors,
        "local_coverage": coverage,
        "unexpected_skipped_categories": unexpected,
        "vip_checkpoints": vip,
        "vtex_checkpoints": vtex,
        "pharmacy_cursors": pharmacy,
        "offers": offers,
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    failed = [item["supermarket_id"] for item in collectors if item["status"] == "failed"]
    local_total = sum(item["total_categories"] for item in coverage)
    local_selected = sum(item["selected_categories"] for item in coverage)
    print(json.dumps({
        "collectors": len(collectors),
        "failed": failed,
        "local_sources": len(coverage),
        "local_categories": [local_total, local_selected],
        "unexpected_skipped_categories": unexpected,
        "vip_open_sweeps": len(vip),
        "vtex_networks": len(vtex),
        "output": str(args.output),
    }, ensure_ascii=False))
    raise SystemExit(1 if failed or unexpected else 0)


if __name__ == "__main__":
    main()
