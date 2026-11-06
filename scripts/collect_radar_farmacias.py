#!/usr/bin/env python3
"""Collect the public Radar/Farmarcas app catalogue for Sorriso pharmacies."""
import argparse
import json
import re
import sqlite3
import subprocess
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

API = "https://api.radarfarmarcas.com.br"
INTERVAL = 600
MAX_PAGES = 2000
COUNT = re.compile(
    r"(?i)(\d+)\s*(comprimidos?|c[aá]psulas?|pastilhas?|unidades?|envelopes?|sach[eê]s?|tiras?|fraldas?|acionamentos?|rolos?|ampolas?|frascos?|un|und|unid)\b"
)
MEASURE = re.compile(r"(?i)(\d+(?:[.,]\d+)?)\s*(kg|g|mg|ml|l|litros?)\b")


class Api:
    def __init__(self, network_id, store_id):
        self.network_id = network_id
        self.store_id = store_id
        self.requests = 0

    def get(self, path, attempt=0):
        if self.requests:
            time.sleep(0.4)
        self.requests += 1
        headers = {
            "X-Pharmacy-Network-Id": self.network_id,
            "X-Pharmacy-Id": self.store_id,
            "X-UUID": self.store_id,
            "Accept": "application/json",
            "User-Agent": "SorriPrecos/1.0 (public-catalogue)",
        }
        request = urllib.request.Request(API + path, headers=headers)
        try:
            with urllib.request.urlopen(request, timeout=30) as response:
                if response.status == 204:
                    return None
                return json.load(response)
        except urllib.error.HTTPError as error:
            if error.code in (429, 500, 502, 503, 504) and attempt < 5:
                retry = error.headers.get("Retry-After")
                delay = max(10, int(retry)) if retry and retry.isdigit() else min(60, 10 * (attempt + 1))
                time.sleep(delay)
                return self.get(path, attempt + 1)
            raise


def package(name):
    counts = list(COUNT.finditer(name))
    if counts:
        return int(counts[-1].group(1)), "un"
    measures = list(MEASURE.finditer(name))
    if measures:
        match = measures[-1]
        value = float(match.group(1).replace(",", "."))
        unit = match.group(2).lower()
        if unit == "kg":
            return max(1, int(value * 1000)), "g"
        if unit in ("l", "litro", "litros"):
            return max(1, int(value * 1000)), "ml"
        if unit == "mg":
            return 1, "un"
        return max(1, int(value)), "g" if unit == "g" else "ml"
    return 1, "un"


def cents(text):
    match = re.fullmatch(r"R\$\s*(\d{1,6}),(\d{2})", (text or "").strip())
    return int(match.group(1)) * 100 + int(match.group(2)) if match else None


def valid_gtin(value):
    if not value or len(value) not in (8, 12, 13, 14) or not value.isdigit() or not value.strip("0"):
        return False
    digits = [int(char) for char in value]
    total = sum(digit * (3 if index % 2 == 0 else 1) for index, digit in enumerate(reversed(digits[:-1])))
    return (10 - total % 10) % 10 == digits[-1]


def row(item, branch_id, observed):
    name = " ".join(item.get("name", "").split())
    price = cents(item.get("displayPrice"))
    if not name:
        return None, "nome inválido"
    quantity, unit = package(name)
    old = cents(item.get("oldPrice"))
    base = price
    promotional = None
    if old and old > price:
        base, promotional = old, price
    gtin = item.get("codigoBarras") if valid_gtin(item.get("codigoBarras")) else None
    result = {
        "source_id": f"radar-{item['id']}",
        "branch_id": branch_id,
        "name": name[:180],
        "brand": (item.get("manufacturer") or "Não informada")[:80],
        "quantity": quantity,
        "unit": unit,
        "presentation": "unidade de venda",
        "category": "Farmácia",
        "gtin": gtin,
        "price_cents": base,
        "promotional_price_cents": promotional,
        "offer_type": "regular",
        "conditions": None,
        "available": item.get("available") is True,
        "source_url": "https://www.drogariasultrapopular.com.br/",
        "photo_source_url": item.get("imageUrl"),
        "collected_at": observed,
        "valid_until": observed + 1800,
        "source_city": "Sorriso",
        "source_state": "MT",
    }
    if not (1 <= result["quantity"] <= 1_000_000) or (price is not None and not 1 <= price <= 100_000_000):
        return None, "preço ou embalagem fora do limite"
    return result, None


def catalogue(api):
    departments = api.get("/mobile/merchandising/v1/products/departments")
    total_categories = len(departments.get("items", [])) if departments else 0
    products = {}
    page = 1
    while page <= MAX_PAGES:
        data = api.get(f"/mobile/merchandising/v1/emphasis/products/showcase-products?page={page}")
        if not data:
            break
        next_pages = []
        for section in data:
            for item in section.get("items", []):
                products[item["id"]] = item
            next_pages.append(section.get("nextPage"))
        if not any(next_pages):
            break
        target = max(value for value in next_pages if value)
        if target <= page:
            break
        page = target
    facets = api.get("/mobile/marketing/v1/products/search/Facets/ByDeviceId/?q=paracetamol")
    terms = list(facets.get("manufacturers", [])) if facets else []
    if departments:
        terms.extend(item.get("name", "") for item in departments.get("items", []))
    terms.extend(
        [
            "comprimidos", "cápsulas", "solução", "xarope", "creme", "shampoo", "sabonete",
            "fralda", "leite", "fórmula", "vitamina", "protetor", "absorvente", "preservativo",
            "teste", "termômetro", "máscara", "álcool", "curativo", "gaze", "seringa", "agulha",
            "pomada", "gel", "spray", "gotas", "óleo", "envelope", "pó", "sachê",
        ]
    )
    for term in dict.fromkeys(term for term in terms if term):
        data = api.get(
            "/mobile/marketing/v1/products/search/Preview/ByDeviceId/?"
            + urllib.parse.urlencode({"q": term, "page": 1, "perPage": 100})
        )
        for item in (data or {}).get("items", []):
            products.setdefault(item["id"], item)
    return list(products.values()), total_categories


def run_import(binary, database_url, rows, authorization, temp):
    path = Path(temp) / "rows.json"
    path.write_text(json.dumps(rows, ensure_ascii=False), encoding="utf-8")
    result = subprocess.run(
        [binary, "--database-url", database_url, "import", "--source", "ultra-popular",
         "--file", str(path), "--authorization", authorization],
        capture_output=True, text=True, timeout=120,
    )
    return result.returncode == 0, (result.stderr or result.stdout).strip()


def align_presentations(rows, database):
    if not Path(database).exists():
        return
    with sqlite3.connect(database) as db:
        existing = {
            gtin: (quantity, unit, presentation)
            for gtin, quantity, unit, presentation in db.execute(
                "SELECT gtin,quantity,unit,presentation FROM products WHERE gtin IS NOT NULL AND is_demo=0"
            )
        }
    for row in rows:
        if row.get("gtin"):
            found = existing.get(row["gtin"].zfill(14))
            if found and (found[0], found[1]) != (row["quantity"], row["unit"]):
                row["gtin"] = None
            elif found:
                row["presentation"] = found[2]
            else:
                row["presentation"] = "unidade de venda"


def import_batch(binary, database_url, rows, authorization):
    rejected = []
    imported = 0
    with tempfile.TemporaryDirectory(prefix="sorri-radar-") as temp:
        ok, _ = run_import(binary, database_url, rows, authorization, temp)
        if ok:
            return len(rows), rejected
        for start in range(0, len(rows), 200):
            chunk = rows[start:start + 200]
            ok, _ = run_import(binary, database_url, chunk, authorization, temp)
            if ok:
                imported += len(chunk)
                continue
            for row in chunk:
                ok, error = run_import(binary, database_url, [row], authorization, temp)
                if ok:
                    imported += 1
                else:
                    rejected.append({"source_id": row["source_id"], "name": row["name"], "error": error[-500:]})
    return imported, rejected


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--config", type=Path, default=Path("config/radar.json"))
    parser.add_argument("--database-url", default="sqlite://data/sorriprecos.db")
    parser.add_argument("--binary", default="target/release/sorriprecos")
    parser.add_argument("--watch", action="store_true")
    parser.add_argument("--once", action="store_true")
    args = parser.parse_args()
    config = json.loads(args.config.read_text(encoding="utf-8"))
    database = args.database_url.removeprefix("sqlite://")
    while True:
        cycle_start = int(time.time())
        total_items = total_imported = total_requests = 0
        failed = False
        failures = []
        rejected = []
        for store in config["stores"]:
            api = Api(config["network_id"], store["store_id"])
            try:
                items, categories = catalogue(api)
                observed = int(time.time())
                rows = []
                for item in items:
                    row_value, reason = row(item, store["branch_id"], observed)
                    if row_value:
                        rows.append(row_value)
                    else:
                        rejected.append({"id": item.get("id"), "name": item.get("name"), "reason": reason})
                align_presentations(rows, database)
                imported, import_rejected = (
                    import_batch(args.binary, args.database_url, rows, config["authorization"])
                    if rows
                    else (0, [])
                )
                rejected.extend(import_rejected)
                total_items += len(items)
                total_imported += imported
                total_requests += api.requests
                with sqlite3.connect(database) as db:
                    if items:
                        db.execute("CREATE TEMP TABLE radar_seen(source_id TEXT PRIMARY KEY)")
                        db.executemany(
                            "INSERT OR IGNORE INTO radar_seen(source_id) VALUES(?)",
                            [(row["source_id"],) for row in rows],
                        )
                        db.execute(
                            "UPDATE offers SET available=NULL WHERE supermarket_id='ultra-popular' AND branch_id=? AND source_product_id IN (SELECT id FROM source_products WHERE supermarket_id='ultra-popular' AND source_id LIKE 'radar-%') AND source_product_id NOT IN (SELECT sp.id FROM source_products sp JOIN radar_seen s ON s.source_id=sp.source_id WHERE sp.supermarket_id='ultra-popular')",
                            (store["branch_id"],),
                        )
                    db.execute(
                        "INSERT INTO source_coverage(supermarket_id,branch_id,adapter,observed_at,total_categories,selected_categories,total_items,selected_items,total_variants,selected_variants,skipped_categories) VALUES(?,?,'radar',?,?,?,?,?,?,?,'[]') ON CONFLICT(supermarket_id,branch_id) DO UPDATE SET observed_at=excluded.observed_at,total_categories=excluded.total_categories,selected_categories=excluded.selected_categories,total_items=excluded.total_items,selected_items=excluded.selected_items,total_variants=excluded.total_variants,selected_variants=excluded.selected_variants",
                        ("ultra-popular", store["branch_id"], observed, categories, categories, len(items), imported, len(items), imported),
                    )
                    db.commit()
                print(json.dumps({"store": store["branch_id"], "items": len(items), "imported": imported, "rejected": len(import_rejected)}, ensure_ascii=False), flush=True)
            except Exception as error:
                failed = True
                failures.append(str(error))
                print(json.dumps({"store": store.get("branch_id"), "error": str(error)}, ensure_ascii=False), flush=True)
        if rejected:
            audit = Path("artifacts") / "radar-rejected.json"
            audit.parent.mkdir(parents=True, exist_ok=True)
            audit.write_text(json.dumps(rejected, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        with sqlite3.connect(database) as db:
            db.execute(
                "INSERT INTO catalogue_collectors(supermarket_id,adapter,interval_seconds,status,last_started_at,last_finished_at,last_success_at,last_error,requests,raw_items,normalized_items) VALUES(?,'radar',600,?,?,?,?,?,?,?,?) ON CONFLICT(supermarket_id) DO UPDATE SET status=excluded.status,last_started_at=excluded.last_started_at,last_finished_at=excluded.last_finished_at,last_success_at=excluded.last_success_at,last_error=excluded.last_error,requests=excluded.requests,raw_items=excluded.raw_items,normalized_items=excluded.normalized_items",
                (
                    "ultra-popular",
                    "failed" if failed else "success",
                    cycle_start,
                    int(time.time()),
                    int(time.time()) if not failed else None,
                    "; ".join(failures)[:500] if failures else None,
                    total_requests,
                    total_items,
                    total_imported,
                ),
            )
            if not failed:
                db.execute("UPDATE supermarkets SET integration_status='authorized' WHERE id='ultra-popular'")
            db.commit()
        if not args.watch:
            if failed:
                raise SystemExit(1)
            break
        time.sleep(INTERVAL)


if __name__ == "__main__":
    main()
