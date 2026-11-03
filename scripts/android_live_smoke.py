#!/usr/bin/env python3
"""Validate current public storefront data through localhost on a physical phone."""
import argparse,datetime,json,time,urllib.request
from android_smoke import Phone,ROOT

def api(path):
    with urllib.request.urlopen('http://127.0.0.1:8080'+path,timeout=10) as r:return json.load(r)
def money(cents):return ('R$ '+f'{cents/100:,.2f}').replace(',','_').replace('.',',').replace('_','.')
def scroll(phone):phone.command('shell','input','swipe',str(phone.width//2),str(int(phone.height*.8)),str(phone.width//2),str(int(phone.height*.35)),'350')
def main():
    parser=argparse.ArgumentParser();parser.add_argument('serial');args=parser.parse_args();phone=Phone(args.serial)
    output=ROOT/'artifacts/android-live';output.mkdir(parents=True,exist_ok=True)
    status=api('/v1/catalog/status');assert status['automatic_integrations']>=3 and status['recent_products']>0,status
    products=api('/v1/products/search?q=arroz%205kg&limit=50')['items']
    product=next(p for p in products if p.get('gtin') and p['branch_count']>=2 and p['quantity']==5000 and p['unit']=='g')
    comparison=api(f"/v1/products/{product['id']}/prices")
    recent=[o for o in comparison['items'] if o['status']=='recent']
    assert len({o['supermarket_id'] for o in recent})>=2
    assert all(o['city']=='Sorriso' and o['state']=='MT' for o in comparison['items'])
    expected=money(product['min_price_cents']);report={'tested_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'device_model':phone.command('shell','getprop','ro.product.model').decode().strip(),'android':phone.command('shell','getprop','ro.build.version.release').decode().strip(),'scope':{'city':'Sorriso','state':'MT'},'kind':'current_public_storefront','product':product,'recent_offers':recent,'tests':[]}
    def passed(name):report['tests'].append({'name':name,'status':'passed'});print('PASS:',name,flush=True)
    phone.command('reverse','tcp:8080','tcp:8080')
    try:
        root=phone.launch()
        texts=phone.texts(root)
        if any('DADOS REAIS HISTÓRICOS' in t or 'MODO DEMO' in t for t in texts):
            label='Pesquisa histórica do Procon' if any('DADOS REAIS HISTÓRICOS' in t for t in texts) else 'Modo DEMO'
            phone.tap('Ajustes');root=phone.wait('Do seu jeito')
            for _ in range(5):
                if label in phone.texts(root):break
                scroll(phone);root=phone.dump()
            phone.tap(label);phone.tap('Início')
        phone.launch();root=phone.wait('Mercados de Sorriso');assert not any('MODO DEMO' in t or 'DADOS REAIS HISTÓRICOS' in t for t in phone.texts(root))
        phone.screenshot(output/'home.png');passed('Catálogo atual é separado do Procon e da demonstração')
        phone.tap('Buscar');phone.wait('Encontre o produto');phone.command('shell','input','text',product['gtin']);root=phone.wait(expected)
        assert product['name'] in phone.texts(root);assert 'Embalagem: 5 kg' in phone.texts(root)
        phone.screenshot(output/'search.png');passed('Pesquisa pelo GTIN retorna produto real, embalagem e preço iguais à API local')
        phone.tap(product['name']);phone.wait('Preços por mercado');root=phone.wait(expected)
        seen=set();wanted={o['supermarket_name'] for o in recent}
        for _ in range(5):
            root=phone.dump();seen.update(wanted.intersection(phone.texts(root)))
            if seen==wanted:break
            scroll(phone)
        assert seen==wanted,(seen,wanted)
        phone.screenshot(output/'comparison.png');passed('Mesmo GTIN e embalagem com preços recentes de duas ou mais redes de Sorriso')
        # Return to the top of the comparison without touching other applications.
        for _ in range(4):
            texts=phone.texts(phone.dump())
            if 'Atualizar' in texts:break
            phone.command('shell','input','swipe',str(phone.width//2),str(int(phone.height*.35)),str(phone.width//2),str(int(phone.height*.8)),'350')
        phone.tap('Detalhes');phone.wait('A mesma embalagem, sempre');phone.wait(product['gtin']);phone.screenshot(output/'details.png')
        passed('Detalhes conservam GTIN, embalagem e origem do produto real')
        phone.tap('Preços');phone.command('reverse','--remove','tcp:8080');phone.tap('Atualizar');phone.wait('Offline · Comparação salva',timeout=25)
        phone.screenshot(output/'offline.png');passed('Comparação real disponível no cache com identificação offline')
        phone.command('reverse','tcp:8080','tcp:8080');phone.tap('Atualizar');phone.wait(expected)
        deadline=time.monotonic()+15
        while any('Offline · Comparação salva' in t for t in phone.texts(phone.dump())):
            assert time.monotonic()<deadline,'Reconexão não concluiu'
        phone.screenshot(output/'reconnected.png');passed('Reconexão ao localhost e aplicativo deixado nos preços atuais')
        report['status']='passed'
    finally:
        phone.command('reverse','tcp:8080','tcp:8080')
        (output/'report.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
if __name__=='__main__':main()
