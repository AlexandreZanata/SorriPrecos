#!/usr/bin/env python3
"""Import real HISTORICAL observations, never current offers, from a public report.
Only municipality Sorriso + state MT + an explicit retailer/address mapping is accepted.
No authenticated Nota MT endpoints or retailer APIs are called.
"""
import argparse,collections,datetime,decimal,hashlib,json,pathlib,re,subprocess,unicodedata,urllib.request

ROOT=pathlib.Path(__file__).resolve().parents[1]
SOURCE='https://www.procon.mt.gov.br/documents/d/procon/tabela-produtos-analisados-pascoa-final'
BRANDS=['LACTA','NESTLE','GAROTO','FERRERO','KINDER','ARCOR','CACAU SHOW','DELICE','CONTENTTO','BRASMAR','SADIA','SEARA','PERDIGAO','LA SUISSA','LINDT']

def normalize(value):return ' '.join(''.join(c for c in unicodedata.normalize('NFD',value) if not unicodedata.combining(c)).upper().split())

def match_branch(city,address,name,mappings):
    # Municipality is its own source column; a name containing Sorriso is not sufficient.
    if normalize(city)!='SORRISO':return None
    address=normalize(address)
    if not re.search(r'\bSORRISO\s*-\s*MT\b',address):return None
    for m in mappings:
        if normalize(name) in [normalize(s) for s in m['source_names']] and normalize(m['street_hint']) in address and re.search(r'\b'+re.escape(m['street_number'])+r'\b',address):return m
    return None

def cents(value):
    text=value.replace('R$','').replace(' ','').replace('.','').replace(',','.')
    number=decimal.Decimal(text)*100
    if number!=number.to_integral_value() or number<=0:raise ValueError('preço inválido')
    return int(number)

def convert(cells,page,mappings):
    if len(cells)!=11:return None,'formato'
    survey,note,city,category,name,licensed,weight,price,per_weight,market,address=cells
    mapping=match_branch(city,address,market,mappings)
    if not mapping:return None,'filial_fora_do_mapeamento'
    try:
        observed=datetime.datetime.strptime(note,'%d/%m/%Y').replace(tzinfo=datetime.timezone(datetime.timedelta(hours=-4)))
        # The report is dated March 2026. Never replace its observation with today's date.
        if not (datetime.datetime(2026,3,1,tzinfo=observed.tzinfo)<=observed<datetime.datetime(2026,4,1,tzinfo=observed.tzinfo)):return None,'data'
        amount=decimal.Decimal(weight.replace('.','').replace(',','.'))
        if amount<=0 or amount!=amount.to_integral_value():return None,'embalagem_ausente_ou_fracionaria'
        brand=next((b for b in BRANDS if b in normalize(name)),None)
        if brand is None:return None,'marca_nao_confirmada'
        value=cents(price)
    except (ValueError,decimal.InvalidOperation):return None,'campos_incompletos'
    quantity=int(amount)
    # The public report does not carry GTINs. Preserve exact variant names; do not infer aliases.
    name=' '.join(name.split())
    sku=hashlib.sha256(f'{normalize(name)}|{brand}|{quantity}g'.encode()).hexdigest()[:20]
    row=dict(source_id='procon-2026-'+sku,branch_id=mapping['branch_id'],name=name,brand=brand,
        quantity=quantity,unit='g',presentation='unidade conforme relatório',category=category,
        gtin=None,price_cents=value,promotional_price_cents=None,offer_type='regular',
        conditions=f'HISTÓRICO · Procon-MT / Páscoa 2026. Nota de {note}; pesquisa de {survey}. Não é oferta atual nem confirmação de estoque.',
        available=None,source_url=SOURCE+f'#page={page}',collected_at=int(observed.timestamp()),valid_until=None,
        source_city='Sorriso',source_state='MT')
    return (mapping['supermarket_id'],row),'accepted'

def extract(pdf):
    import pdfplumber
    text=subprocess.check_output(['pdftotext','-layout',str(pdf),'-']).decode('utf-8')
    indices=[i for i,p in enumerate(text.split('\f')) if re.search(r'\bSorriso\b',p)]
    with pdfplumber.open(pdf) as doc:
        for index in indices:
            page=doc.pages[index]
            for table in page.find_tables():
                if len(table.columns)!=11:continue
                left,right=table.columns[0].bbox[0],table.columns[0].bbox[2]
                for cell in table.cells:
                    if abs(cell[0]-left)>.2 or abs(cell[2]-right)>.2:continue
                    value=(page.crop(cell).extract_text() or '').strip()
                    if not re.fullmatch(r'\d{2}/\d{2}/\d{4}',value):continue
                    cells=[' '.join((page.crop((c.bbox[0],cell[1],c.bbox[2],cell[3])).extract_text() or '').split()) for c in table.columns]
                    if normalize(cells[2])=='SORRISO':yield index+1,cells

def main():
    p=argparse.ArgumentParser();p.add_argument('--pdf',type=pathlib.Path);p.add_argument('--output',type=pathlib.Path,default=ROOT/'artifacts/procon-sorriso');p.add_argument('--import-db',action='store_true');p.add_argument('--binary',type=pathlib.Path,default=ROOT/'target/release/sorriprecos');p.add_argument('--database-url',default='sqlite://data/sorriprecos.db');a=p.parse_args()
    a.output.mkdir(parents=True,exist_ok=True)
    pdf=a.pdf or a.output/'procon-pascoa-2026.pdf'
    if a.pdf is None:
        with urllib.request.urlopen(urllib.request.Request(SOURCE,headers={'User-Agent':'SorriPrecos/0.1 public-report-local-test'}),timeout=30) as response:
            data=response.read(15*1024*1024+1)
            if len(data)>15*1024*1024 or not data.startswith(b'%PDF'):raise SystemExit('Relatório inválido')
            pdf.write_bytes(data)
    mappings=json.loads((ROOT/'config/procon_sorriso_branches.json').read_text())
    rejected=collections.Counter();candidates=collections.defaultdict(list);source_rows=0
    for page,cells in extract(pdf):
        source_rows+=1;converted,reason=convert(cells,page,mappings)
        if converted is None:rejected[reason]+=1;continue
        market,row=converted;candidates[(market,row['source_id'],row['branch_id'])].append(row)
    grouped=collections.defaultdict(list)
    for (market,sku,branch),rows in candidates.items():
        latest=max(r['collected_at'] for r in rows);current=[r for r in rows if r['collected_at']==latest]
        if len({r['price_cents'] for r in current})!=1:rejected['precos_conflitantes_na_mesma_data']+=1;continue
        grouped[market].append(current[0])
    for market,rows in grouped.items():
        path=a.output/f'{market}.json';path.write_text(json.dumps(rows,ensure_ascii=False,indent=2)+'\n')
        if a.import_db:subprocess.run([str(a.binary),'--database-url',a.database_url,'import-public-report','--source',market,'--file',str(path)],check=True,cwd=ROOT)
    report=dict(source_url=SOURCE,pdf_sha256=hashlib.sha256(pdf.read_bytes()).hexdigest(),scope={'city':'Sorriso','state':'MT'},observation_period='2026-03',kind='historical_public_report',source_rows_sorriso=source_rows,accepted_offers=sum(map(len,grouped.values())),mapped_candidate_rows=sum(map(len,candidates.values())),collapsed_or_conflicting_rows=sum(map(len,candidates.values()))-sum(map(len,grouped.values())),markets={k:len(v) for k,v in grouped.items()},rejected=dict(rejected),note='Preços históricos reais, sem catálogo atual ou autorização comercial de integração com varejistas.')
    (a.output/'report.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n');print(json.dumps(report,ensure_ascii=False),flush=True)

if __name__=='__main__':main()
