#!/usr/bin/env python3
"""Discover public Casa Aurora artwork and import a reviewed, current meat flyer."""
import argparse
import datetime as dt
import hashlib
import json
import os
import re
import subprocess
import tempfile
from pathlib import Path
from urllib.parse import urlsplit

from collect_atacadao_flyers import MAX_HTML, TZ, fetch

ORIGIN = 'https://casaaurora.com.br'
MEDIA = f'{ORIGIN}/wp-json/wp/v2/media'
FIELDS = 'id,date,title,source_url'


def verified_rows(manifest, observed, download=fetch):
    today = observed.date()
    start = dt.date.fromisoformat(manifest['valid_from'])
    end = dt.date.fromisoformat(manifest['valid_through'])
    if not start <= today <= end:
        raise ValueError('encarte fora da validade em Sorriso')
    media_id = manifest['media_id']
    if type(media_id) is not int or media_id <= 0:
        raise ValueError('identificador de mídia inválido')
    attachment = json.loads(download(f'{MEDIA}/{media_id}?_fields={FIELDS}', MAX_HTML))
    url = urlsplit(attachment['source_url'])
    if (attachment['id'] != media_id or url.scheme != 'https'
            or url.netloc != 'casaaurora.com.br' or url.query or url.fragment
            or not re.fullmatch(r'/wp-content/uploads/\d{4}/\d{2}/[\w.-]+\.(?:jpg|jpeg|png|webp)', url.path)):
        raise ValueError('mídia fora do endereço oficial esperado')
    image = download(url.geturl(), 8_000_000)
    if hashlib.sha256(image).hexdigest() != manifest['image_sha256']:
        raise ValueError('a arte oficial mudou e precisa de nova revisão')
    seen = set()
    rows = []
    for item in manifest['items']:
        key, name, price = item['id'], item['name'].strip(), item['price_cents']
        if (not re.fullmatch(r'[a-z0-9-]+', key) or key in seen
                or not 1 <= len(name) <= 180 or type(price) is not int or not 0 < price < 1_000_000):
            raise ValueError('item inválido ou duplicado no encarte revisado')
        seen.add(key)
        rows.append({
            'source_id': f'flyer-{media_id}-{key}', 'branch_id': 'casa-aurora-centro',
            'name': name, 'brand': 'Não informada', 'quantity': 1, 'unit': 'kg',
            'presentation': 'Preço por kg', 'category': 'Açougue', 'price_cents': price,
            'offer_type': 'regular', 'conditions': 'Preço do encarte oficial. Estoque não confirmado.',
            'available': None, 'source_url': url.geturl(),
            'collected_at': int(observed.timestamp()),
            'valid_until': int(dt.datetime.combine(end, dt.time.max, TZ).timestamp()),
            'source_city': 'Sorriso', 'source_state': 'MT',
        })
    if not rows:
        raise ValueError('nenhum item revisado para importar')
    return rows


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    action = parser.add_mutually_exclusive_group(required=True)
    action.add_argument('--list', action='store_true', help='lista as 20 mídias oficiais mais recentes')
    action.add_argument('--manifest', type=Path)
    parser.add_argument('--database-url', default=os.environ.get('DATABASE_URL', 'sqlite://data/sorriprecos.db'))
    parser.add_argument('--binary', default='target/debug/sorriprecos')
    args = parser.parse_args()
    if args.list:
        data = fetch(f'{MEDIA}?per_page=20&_fields={FIELDS}', MAX_HTML)
        print(json.dumps(json.loads(data), ensure_ascii=False, indent=2))
        return
    manifest = json.loads(args.manifest.read_text(encoding='utf-8'))
    rows = verified_rows(manifest, dt.datetime.now(TZ))
    with tempfile.TemporaryDirectory(prefix='sorri-aurora-flyer-') as temp:
        batch = Path(temp) / 'rows.json'
        batch.write_text(json.dumps(rows, ensure_ascii=False), encoding='utf-8')
        subprocess.run(
            [args.binary, '--database-url', args.database_url, 'import', '--source', 'casa-aurora',
             '--file', str(batch), '--authorization', 'PUBLIC_STOREFRONT: encarte Casa Aurora vigente em Sorriso'],
            check=True, timeout=60,
        )


if __name__ == '__main__':
    main()
