#!/usr/bin/env python3
"""Import the public D'Tália Sorriso menu from the store's own website."""
import argparse
import datetime as dt
import hashlib
import html
import json
import re
import sqlite3
import subprocess
import tempfile
import time
import urllib.parse
import urllib.request
from pathlib import Path
from zoneinfo import ZoneInfo

ORIGIN = 'https://sorriso.pizzariadtalia.com'
MENU = f'{ORIGIN}/cardapio'
SUPERMARKET = 'pizzaria-dtalia'
BRANCH = 'pizzaria-dtalia-sorriso'
MAX_HTML = 3_000_000
INTERVAL = 6 * 3600
TZ = ZoneInfo('America/Cuiaba')
AUTHORIZATION = (
    "PUBLIC_STOREFRONT: cardápio público da Pizzaria D'Tália Sorriso; "
    "preços sujeitos a confirmação na loja"
)
PRICE = re.compile(r'R\$\s*([\d.]+),(\d{2})')


class SameHostRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        old, new = urllib.parse.urlsplit(req.full_url), urllib.parse.urlsplit(newurl)
        if new.scheme != 'https' or new.hostname != old.hostname:
            raise ValueError('redirect para domínio não autorizado')
        return super().redirect_request(req, fp, code, msg, headers, newurl)


def fetch(url, max_bytes=MAX_HTML):
    request = urllib.request.Request(
        url, headers={'User-Agent': 'SorriPrecos/1.0 (catalogo publico Sorriso)'}
    )
    with urllib.request.build_opener(SameHostRedirect()).open(request, timeout=35) as response:
        final = urllib.parse.urlsplit(response.url)
        if final.scheme != 'https' or final.hostname != urllib.parse.urlsplit(url).hostname:
            raise ValueError('redirect para domínio não autorizado')
        if response.status != 200:
            raise ValueError(f'HTTP {response.status}')
        body = response.read(max_bytes + 1)
        if len(body) > max_bytes:
            raise ValueError('resposta grande demais')
        return body.decode('utf-8', 'replace')


def text(value):
    return ' '.join(html.unescape(re.sub(r'<[^>]+>', ' ', value)).split())


def cents(value):
    match = PRICE.search(value or '')
    if not match:
        return None
    return int(match.group(1).replace('.', '')) * 100 + int(match.group(2))


def category_name(button):
    name = re.sub(r'\s*\d+\s*(?:itens?|sabores?)\b.*$', '', text(button))
    return name.removesuffix('+ Detalhes').strip()


def parse(page, observed):
    rows = []
    skipped = []
    categories = selected = 0
    for card in re.split(r'<div class="card">', page)[1:]:
        header = re.search(r'data-target="#(collapse-[^"]+)"[^>]*>(.*?)</button>', card, re.S)
        if not header:
            continue
        categories += 1
        category = category_name(header.group(2))
        priced = 0
        for item in re.findall(r'<li class="list-group-item">(.*?)</li>', card, re.S):
            fields = re.findall(r'<div class="col-12">(.*?)</div>', item, re.S)
            fields = [text(field) for field in fields]
            if not fields:
                continue
            name = fields[0]
            price = cents(next((field for field in fields[1:] if PRICE.search(field)), ''))
            if not name or price is None:
                continue
            if not 1 <= len(name) <= 180 or not 0 < price <= 100_000_000:
                continue
            priced += 1
            key = hashlib.sha1(f'{category}|{name}'.encode('utf-8')).hexdigest()[:16]
            rows.append({
                'source_id': f'dtalia-{key}',
                'branch_id': BRANCH,
                'name': name,
                'brand': "D'Tália",
                'quantity': 1,
                'unit': 'un',
                'presentation': 'Unidade de venda',
                'category': f'Restaurante · {category}'[:80],
                'price_cents': price,
                'offer_type': 'regular',
                'conditions': 'Preço do cardápio online. Confirme disponibilidade, entrega e retirada.',
                'available': True,
                'source_url': MENU,
                'collected_at': int(observed.timestamp()),
                'valid_until': int((observed + dt.timedelta(days=7)).timestamp()),
                'source_city': 'Sorriso',
                'source_state': 'MT',
            })
        if priced:
            selected += 1
        elif category:
            skipped.append(category)
    if not rows:
        raise ValueError('nenhum preço encontrado no cardápio')
    return rows, {'categories': categories, 'selected_categories': selected, 'skipped': skipped}


def import_rows(binary, database_url, rows):
    rejected = []
    with tempfile.TemporaryDirectory(prefix='sorri-dtalia-') as temp:
        path = Path(temp) / 'rows.json'
        path.write_text(json.dumps(rows, ensure_ascii=False), encoding='utf-8')
        def run():
            return subprocess.run(
                [binary, '--database-url', database_url, 'import', '--source', SUPERMARKET,
                 '--file', str(path), '--authorization', AUTHORIZATION],
                capture_output=True, text=True, timeout=120,
            )
        result = run()
        if result.returncode == 0:
            return len(rows), rejected
        imported = 0
        for row in rows:
            path.write_text(json.dumps([row], ensure_ascii=False), encoding='utf-8')
            one = run()
            if one.returncode == 0:
                imported += 1
            else:
                rejected.append({
                    'source_id': row['source_id'],
                    'name': row['name'],
                    'error': (one.stderr or one.stdout).strip()[-500:],
                })
        return imported, rejected


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--url', default=MENU)
    parser.add_argument('--database-url', default='sqlite://data/sorriprecos.db')
    parser.add_argument('--binary', default='target/release/sorriprecos')
    parser.add_argument('--watch', action='store_true')
    args = parser.parse_args()
    database = args.database_url.removeprefix('sqlite://')
    while True:
        started = int(time.time())
        observed = dt.datetime.now(TZ)
        requests = 0
        try:
            page = fetch(args.url)
            requests = 1
            rows, coverage = parse(page, observed)
            imported, rejected = import_rows(args.binary, args.database_url, rows)
            with sqlite3.connect(database) as db:
                db.execute('CREATE TEMP TABLE IF NOT EXISTS dtalia_seen(source_id TEXT PRIMARY KEY)')
                db.execute('DELETE FROM dtalia_seen')
                db.executemany(
                    'INSERT OR IGNORE INTO dtalia_seen(source_id) VALUES(?)',
                    [(row['source_id'],) for row in rows],
                )
                db.execute(
                    "UPDATE offers SET available=0 WHERE supermarket_id=? AND branch_id=? "
                    "AND source_product_id IN (SELECT id FROM source_products WHERE supermarket_id=? AND source_id LIKE 'dtalia-%') "
                    "AND source_product_id NOT IN (SELECT sp.id FROM source_products sp JOIN dtalia_seen s ON s.source_id=sp.source_id WHERE sp.supermarket_id=?)",
                    (SUPERMARKET, BRANCH, SUPERMARKET, SUPERMARKET),
                )
                db.execute(
                    "INSERT INTO source_coverage(supermarket_id,branch_id,adapter,observed_at,total_categories,selected_categories,total_items,selected_items,total_variants,selected_variants,skipped_categories) VALUES(?,?,'dtalia',?,?,?,?,?,0,0,?) ON CONFLICT(supermarket_id,branch_id) DO UPDATE SET observed_at=excluded.observed_at,total_categories=excluded.total_categories,selected_categories=excluded.selected_categories,total_items=excluded.total_items,selected_items=excluded.selected_items,total_variants=0,selected_variants=0,skipped_categories=excluded.skipped_categories",
                    (SUPERMARKET, BRANCH, int(observed.timestamp()), coverage['categories'],
                     coverage['selected_categories'], len(rows), imported,
                     json.dumps(coverage['skipped'], ensure_ascii=False)),
                )
                db.execute(
                    "INSERT INTO catalogue_collectors(supermarket_id,adapter,interval_seconds,status,last_started_at,last_finished_at,last_success_at,last_error,requests,raw_items,normalized_items) VALUES(?,'dtalia',?,'success',?,?,?,?,?,?,?) ON CONFLICT(supermarket_id) DO UPDATE SET status='success',last_started_at=excluded.last_started_at,last_finished_at=excluded.last_finished_at,last_success_at=excluded.last_success_at,last_error=NULL,requests=excluded.requests,raw_items=excluded.raw_items,normalized_items=excluded.normalized_items",
                    (SUPERMARKET, INTERVAL, started, int(time.time()), int(time.time()),
                     None, requests, len(rows), imported),
                )
                db.execute("UPDATE supermarkets SET integration_status='authorized' WHERE id=?", (SUPERMARKET,))
                db.commit()
            print(json.dumps({
                'items': len(rows), 'imported': imported, 'rejected': rejected,
                'skipped_categories': coverage['skipped'],
            }, ensure_ascii=False), flush=True)
        except Exception as error:
            with sqlite3.connect(database) as db:
                db.execute(
                    "INSERT INTO catalogue_collectors(supermarket_id,adapter,interval_seconds,status,last_started_at,last_finished_at,last_error,requests) VALUES(?,'dtalia',?,'failed',?,?,?,?) ON CONFLICT(supermarket_id) DO UPDATE SET status='failed',last_started_at=excluded.last_started_at,last_finished_at=excluded.last_finished_at,last_error=excluded.last_error,requests=excluded.requests",
                    (SUPERMARKET, INTERVAL, started, int(time.time()), str(error)[:500], requests),
                )
                db.commit()
            print(json.dumps({'error': str(error)}, ensure_ascii=False), flush=True)
            if not args.watch:
                raise SystemExit(1)
        if not args.watch:
            break
        time.sleep(INTERVAL)


if __name__ == '__main__':
    main()
