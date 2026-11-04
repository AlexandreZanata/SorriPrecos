#!/usr/bin/env python3
"""Read the official Sorriso flyer; unknown stock must never be ranked as available.

Requires Poppler's pdfinfo and pdftotext. This intentionally fails closed for
unknown flyer formats, prices without the verified three-tier legend, missing
branch/validity evidence, and ambiguous packages.
"""
import argparse
import datetime as dt
import json
import os
import re
import subprocess
import tempfile
import time
import urllib.parse
import urllib.request
from decimal import Decimal, InvalidOperation
from pathlib import Path
from zoneinfo import ZoneInfo

STORE_URL = 'https://www.atacadao.com.br/loja/sorriso'
PDF_HOST = 'apigw.cloud.carrefour.com.br'
PDF_PATH = '/api-middleware-flyer-services/api/v2/Flyer/'
BRANCH = 'atacadao-sorriso'
MAX_HTML = 3_000_000
MAX_PDF = 25_000_000
TZ = ZoneInfo('America/Cuiaba')


class SameHostRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        old, new = urllib.parse.urlsplit(req.full_url), urllib.parse.urlsplit(newurl)
        if new.scheme != 'https' or new.hostname != old.hostname:
            raise ValueError('redirect para domínio não autorizado')
        return super().redirect_request(req, fp, code, msg, headers, newurl)


def fetch(url, max_bytes):
    request = urllib.request.Request(url, headers={'User-Agent': 'SorriPrecos/1.0 (catalogo publico Sorriso)'})
    with urllib.request.build_opener(SameHostRedirect()).open(request, timeout=35) as response:
        # Reject redirects away from the documented hosts, including local addresses.
        final = urllib.parse.urlsplit(response.url)
        original = urllib.parse.urlsplit(url)
        if final.scheme != 'https' or final.hostname != original.hostname:
            raise ValueError('redirect para domínio não autorizado')
        body = response.read(max_bytes + 1)
        if len(body) > max_bytes:
            raise ValueError('resposta grande demais')
        return body


def store_flyers(html, today):
    found = re.search(r'<script\s+id="__NEXT_DATA__"\s+type="application/json">(.*?)</script>', html, re.S)
    if not found:
        raise ValueError('estado público Next.js ausente')
    store = json.loads(found.group(1))['props']['pageProps']['storeInfo']
    if not (store.get('storeId') == 899 and store.get('cidade', '').casefold() == 'sorriso'
            and store.get('estado') == 'MT' and 'idemar riedi' in store.get('endereco', '').casefold()
            and str(store.get('numero')) == '8150'):
        raise ValueError('identidade da filial não corresponde a Sorriso')
    flyers = []
    for flyer in store.get('flyers', []):
        try:
            start = dt.date.fromisoformat(flyer['validity']['initial'][:10])
            end = dt.date.fromisoformat(flyer['validity']['final'][:10])
            url = urllib.parse.urlsplit(flyer['urlFinalDocument'])
        except (ValueError, KeyError, TypeError):
            continue
        if (start <= today <= end and not flyer.get('exclude') and url.scheme == 'https'
                and url.username is None and url.password is None and url.port is None
                and url.hostname == PDF_HOST and url.path == PDF_PATH
                and urllib.parse.parse_qs(url.query).get('id') == [flyer.get('id')]):
            flyers.append((flyer, start, end))
    return flyers


def flyer_metadata(pdf_path):
    result = subprocess.run(['pdfinfo', str(pdf_path)], capture_output=True, text=True, check=True, timeout=30)
    subject = result.stdout.split('Subject:', 1)
    if len(subject) != 2 or not subject[1].lstrip().startswith('TABLOIDI-DADOS 2.0\n'):
        raise ValueError('metadados estruturados reconhecidos ausentes')
    source = subject[1].lstrip().split('\n', 1)[1]
    data, _ = json.JSONDecoder().raw_decode(source)
    if data.get('versao') != '2.0' or not isinstance(data.get('paginas'), list):
        raise ValueError('versão de dados não reconhecida')
    return data


def valid_pdf_text(text, start, end):
    text = ' '.join(text.upper().split())
    required = ['SORRISO/MT: RUA IDEMAR RIEDI, 8150', 'PREÇO VAREJO/ATACADO',
                'PREÇO COM OFERTA', 'ATIVADA NO', 'CARTÃO ATACADÃO']
    dates = f"OFERTAS VÁLIDAS DE {start:%d/%m} A {end:%d/%m/%Y}"
    return dates in text and all(x in text for x in required)


def package(lines):
    """Accept a single explicit pack, excluding multipacks and mixed variants."""
    description = ' '.join(lines)
    if re.search(r'v[aá]ri[oa]s?|divers[oa]s?|sortid[oa]s?|com casco|\d\s*[x×]\s*\d|/|\+', description, re.I):
        return None
    last = lines[-1].strip()
    if re.fullmatch(r'(?:preço por )?quilo(?: n[ao] (?:peça|pacote))?', last, re.I):
        return 1, 'kg', last
    match = re.search(r'\bcom\s+(\d+(?:[.,]\d+)?)\s*(kg|ml|g|l|unidades?|rolos?)\s*$', last, re.I)
    if not match:
        return None
    value, unit = match.groups(); unit = unit.lower()
    try:
        number = Decimal(value.replace(',', '.'))
    except InvalidOperation:
        return None
    if unit == 'kg': number, unit = number * 1000, 'g'
    elif unit == 'l': number, unit = number * 1000, 'ml'
    elif unit.startswith(('unidade', 'rolo')): unit = 'un'
    if number != number.to_integral() or not 1 <= number <= 1_000_000:
        return None
    return int(number), unit, last


def cents(value):
    match = re.fullmatch(r'R\$\s*(\d{1,6}),(\d{2})', value.strip())
    if not match: raise ValueError('preço inválido')
    return int(match.group(1)) * 100 + int(match.group(2))


def rows_from_metadata(meta, flyer, end, observed):
    rows = []
    valid_until = int(dt.datetime.combine(end, dt.time.max, TZ).timestamp())
    for page in meta['paginas']:
        for item in page.get('itens', []):
            lines = [x.strip() for x in item.get('descricao', {}).get('texto', '').splitlines() if x.strip()]
            if not lines or len(' '.join(lines)) > 180: continue
            parsed = package(lines)
            if not parsed: continue
            fields = item.get('campos', {})
            if set(fields) != {'P', 'PSEG', 'PTER', 'QUNTOFERTAO'}: continue
            try:
                regular, app, card = (cents(fields[k]['texto']) for k in ('P', 'PSEG', 'PTER'))
            except (KeyError, ValueError): continue
            if not 0 < card <= app <= regular: continue
            condition = ' '.join(fields['QUNTOFERTAO']['texto'].split()).lstrip('▪').strip()
            if not condition or len(condition) > 250: continue
            quantity, unit, presentation = parsed
            app_text, card_text = (f'{amount // 100},{amount % 100:02d}' for amount in (app, card))
            conditions = f'Preço com oferta ativada no aplicativo: R$ {app_text}; com aplicativo e cartão: R$ {card_text}. {condition}'
            rows.append({'source_id': f"flyer-{flyer['id']}-{page['pagina']}-{item['numero']}",
                         'branch_id': BRANCH, 'name': ' '.join(lines), 'brand': 'Não informada',
                         'quantity': quantity, 'unit': unit, 'presentation': presentation,
                         'category': 'Ofertas de encarte', 'price_cents': regular,
                         'promotional_price_cents': app, 'offer_type': 'club',
                         'conditions': conditions, 'available': None,
                         'source_url': flyer['urlFinalDocument'], 'photo_source_url': None,
                         'collected_at': observed, 'valid_until': valid_until,
                         'source_city': 'Sorriso', 'source_state': 'MT'})
    return rows


def collect(database_url, binary):
    today = dt.datetime.now(TZ).date()
    flyers = store_flyers(fetch(STORE_URL, MAX_HTML).decode('utf-8'), today)
    if not flyers:
        print('Nenhum encarte vigente validado para Sorriso.', flush=True); return
    for flyer, start, end in flyers:
        with tempfile.TemporaryDirectory(prefix='sorri-flyer-') as temp:
            pdf = Path(temp) / 'flyer.pdf'; pdf.write_bytes(fetch(flyer['urlFinalDocument'], MAX_PDF))
            plain = subprocess.run(['pdftotext', str(pdf), '-'], capture_output=True, text=True, check=True, timeout=40).stdout
            if not valid_pdf_text(plain, start, end):
                print('Formato ou contexto não reconhecido; ignorado:', flyer['id'], flush=True); continue
            try: metadata = flyer_metadata(pdf)
            except (ValueError, KeyError, json.JSONDecodeError):
                print('Sem dados estruturados reconhecidos:', flyer['id'], flush=True); continue
            rows = rows_from_metadata(metadata, flyer, end, int(time.time()))
            if not rows:
                print('Nenhum produto seguro:', flyer['id'], flush=True); continue
            batch = Path(temp) / 'rows.json'; batch.write_text(json.dumps(rows, ensure_ascii=False))
            subprocess.run([binary, '--database-url', database_url, 'import', '--source', 'atacadao',
                            '--file', str(batch), '--authorization', 'PUBLIC_STOREFRONT: encarte oficial da filial Sorriso'], check=True, timeout=60)
            print(f"{flyer['id']}: {len(rows)} registros; estoque desconhecido; validade {end}", flush=True)


def main():
    p = argparse.ArgumentParser(); p.add_argument('--database-url', default=os.environ.get('DATABASE_URL', 'sqlite://data/sorriprecos.db'))
    p.add_argument('--binary', default='artifacts/sorriprecos-retomada-four'); p.add_argument('--watch', action='store_true')
    args = p.parse_args()
    while True:
        try: collect(args.database_url, args.binary)
        except Exception as exc:
            print(f'Coleta Atacadão falhou: {type(exc).__name__}: {exc}', flush=True)
            if not args.watch: raise
        if not args.watch: break
        time.sleep(600)


if __name__ == '__main__': main()
