#!/usr/bin/env python3
"""Verify public historical Sorriso observations through localhost on a physical phone."""
import argparse,datetime,json,pathlib,time
from android_smoke import Phone,ROOT

def main():
    p=argparse.ArgumentParser();p.add_argument('serial');a=p.parse_args();phone=Phone(a.serial)
    output=ROOT/'artifacts/android-real';output.mkdir(parents=True,exist_ok=True)
    report={'device_model':phone.command('shell','getprop','ro.product.model').decode().strip(),'android':phone.command('shell','getprop','ro.build.version.release').decode().strip(),'tested_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'scope':{'city':'Sorriso','state':'MT'},'dataset':'Procon-MT Páscoa 2026','kind':'historical','tests':[]}
    def passed(name):report['tests'].append({'name':name,'status':'passed'});print('PASS:',name,flush=True)
    phone.command('reverse','tcp:8080','tcp:8080')
    try:
        root=phone.launch()
        if any('MODO DEMO' in t for t in phone.texts(root)):
            phone.tap('Fontes')
            for _ in range(6):
                root=phone.dump()
                if 'Ver fontes reais' in phone.texts(root):break
                phone.command('shell','input','swipe',str(phone.width//2),str(int(phone.height*.78)),str(phone.width//2),str(int(phone.height*.35)),'350')
            phone.tap('Ver fontes reais');phone.wait('DADOS REAIS HISTÓRICOS');phone.launch()
        root=phone.wait('DADOS REAIS HISTÓRICOS')
        assert not any('MODO DEMO' in t for t in phone.texts(root))
        phone.screenshot(output/'home.png');passed('Modo real com aviso de referência histórica, sem DEMO')
        phone.tap('Lacta');root=phone.wait('R$ 69,90')
        assert 'REGISTRO HISTÓRICO · DESATUALIZADO' in phone.texts(root)
        phone.screenshot(output/'search.png');passed('Busca real no localhost retorna valor documentado de 6990 centavos')
        phone.tap('OVO PASCOA LACTA 277G SONHO DE VALSA');root=phone.wait('Registros históricos · não são ofertas atuais')
        phone.wait('R$ 69,90');phone.screenshot(output/'comparison.png')
        assert not any(t=='MENOR PREÇO CONHECIDO' for t in phone.texts(root))
        passed('Comparação mantém datas antigas e não destaca preço como oferta atual')
        found=set()
        for _ in range(6):
            root=phone.dump();texts=phone.texts(root)
            for branch in ['Loja Belo Horizonte','Loja Rota do Sol']:
                if branch in texts:found.add(branch)
            if len(found)==2:break
            phone.command('shell','input','swipe',str(phone.width//2),str(int(phone.height*.8)),str(phone.width//2),str(int(phone.height*.4)),'350')
        assert len(found)==2,found
        phone.screenshot(output/'branches.png');passed('Mesmo produto ligado às duas filiais DelMoro de Sorriso do relatório')
        phone.launch();phone.tap('Lacta');phone.wait('R$ 69,90');phone.tap('OVO PASCOA LACTA 277G SONHO DE VALSA');phone.wait('R$ 69,90')
        phone.command('reverse','--remove','tcp:8080');phone.tap('Atualizar');phone.wait('Offline · Comparação salva',timeout=25)
        phone.screenshot(output/'offline.png');passed('Dados reais históricos continuam legíveis no cache offline')
        phone.command('reverse','tcp:8080','tcp:8080');phone.tap('Atualizar');phone.wait('R$ 69,90')
        deadline=time.monotonic()+15
        while any('Offline · Comparação salva' in t for t in phone.texts(phone.dump())):
            if time.monotonic()>deadline:raise AssertionError('Reconexão falhou')
        phone.screenshot(output/'reconnected.png');passed('Reconexão restaurada e aplicativo deixado na comparação real histórica')
        phone.launch();phone.tap('Lacta');phone.wait('R$ 69,90')
        title='OVO PASCOA LACTA 357G SONHO DE VALSA'
        for _ in range(6):
            if title in phone.texts(phone.dump()):break
            phone.command('shell','input','swipe',str(phone.width//2),str(int(phone.height*.8)),str(phone.width//2),str(int(phone.height*.4)),'350')
        phone.tap(title);phone.wait('Registros históricos · não são ofertas atuais')
        found=set()
        expected={'DelMoro Supermercados','DelNorte Atacarejo','R$ 92,41','R$ 81,90'}
        for _ in range(8):
            root=phone.dump();texts=phone.texts(root);found.update(expected.intersection(texts))
            assert 'MENOR PREÇO CONHECIDO' not in texts
            if found==expected:break
            phone.command('shell','input','swipe',str(phone.width//2),str(int(phone.height*.8)),str(phone.width//2),str(int(phone.height*.4)),'350')
        assert found==expected,found
        phone.screenshot(output/'different-markets.png');passed('Mesmo produto de 357g compara registros reais DelMoro e DelNorte de Sorriso, sem oferta atual')
        report['status']='passed'
    finally:
        phone.command('reverse','tcp:8080','tcp:8080')
        (output/'report.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')

if __name__=='__main__':main()
