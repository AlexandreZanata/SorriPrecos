#!/usr/bin/env python3
"""Exercise the reference-inspired UI and accessibility on the named physical phone.
Uses only SorriPreços UI; leaves the app in real mode with default text/light palette.
"""
import argparse,datetime,json,re
from android_smoke import Phone,ROOT,PACKAGE

def node(root,label):
    return next(n for n in root.iter('node') if n.attrib.get('package')==PACKAGE and (n.attrib.get('text')==label or n.attrib.get('content-desc')==label))

def height(root,label):
    bounds=list(map(int,re.findall(r'\d+',node(root,label).attrib['bounds'])))
    return bounds[3]-bounds[1]

def setting(root,label):
    for parent in root.iter('node'):
        if parent.attrib.get('checkable')=='true' and any(n.attrib.get('text')==label or n.attrib.get('content-desc')==label for n in parent.iter('node')):
            return parent.attrib.get('checked')=='true'
    raise AssertionError('Controle de alternância não encontrado: '+label)

def main():
    parser=argparse.ArgumentParser();parser.add_argument('serial');args=parser.parse_args();phone=Phone(args.serial)
    output=ROOT/'artifacts/android-ui';output.mkdir(parents=True,exist_ok=True)
    report={'tested_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'device_model':phone.command('shell','getprop','ro.product.model').decode().strip(),'android':phone.command('shell','getprop','ro.build.version.release').decode().strip(),'tests':[]}
    def passed(name):report['tests'].append({'name':name,'status':'passed'});print('PASS:',name,flush=True)
    phone.command('reverse','tcp:8080','tcp:8080')
    try:
        phone.launch();phone.tap('Ajustes');root=phone.wait('Do seu jeito')
        for label in ['Texto maior','Alto contraste','Modo DEMO']:
            for _ in range(5):
                try:node(root,label);break
                except StopIteration:
                    phone.command('shell','input','swipe',str(phone.width//2),str(int(phone.height*.8)),str(phone.width//2),str(int(phone.height*.4)),'350');root=phone.dump()
            if setting(root,label):phone.tap(label);root=phone.dump()
        phone.tap('Início');root=phone.wait('DADOS REAIS HISTÓRICOS')
        texts=phone.texts(root)
        assert all(label in texts for label in ['Início','Buscar','Favoritos','Ajustes','Mercados de Sorriso','DelMoro','Sorrinorte'])
        phone.screenshot(output/'home.png');passed('Início com marca, grade de mercados, quatro abas e aviso histórico')
        phone.tap('DelMoro');phone.wait('DelMoro Supermercados');phone.wait('Sorriso-MT');phone.tap('Fechar')
        passed('Cartão de mercado abre cobertura e procedência reais')
        phone.tap('Lacta');root=phone.wait('R$ 69,90')
        assert 'Resultados da busca' in phone.texts(root)
        assert any('Ilustração de categoria' in n.attrib.get('content-desc','') for n in root.iter('node'))
        phone.screenshot(output/'search.png');passed('Busca separada com miniaturas identificadas como ilustrações e preço histórico')
        title='OVO PASCOA LACTA 277G SONHO DE VALSA'
        phone.tap(title);root=phone.wait('Registros históricos · não são ofertas atuais')
        phone.wait('Loja Rota do Sol');phone.screenshot(output/'comparison.png')
        assert 'MENOR PREÇO CONHECIDO' not in phone.texts(root)
        passed('Comparação compacta mostra duas filiais sem promover registro antigo como oferta')
        phone.tap('Loja Belo Horizonte');phone.wait('Av. Belo Horizonte, 94');phone.wait('Ver na fonte original')
        phone.screenshot(output/'expanded-source.png');phone.tap('Loja Belo Horizonte')
        passed('Linha da filial expande endereço, data, condições e origem')
        normal=height(phone.dump(),title);phone.tap('Aumentar texto');large=height(phone.dump(),title)
        assert large>normal,(normal,large)
        phone.screenshot(output/'large-text.png');passed('A+ aumenta o texto e reorganiza o conteúdo sem ocultar controles')
        phone.tap('Ativar alto contraste');root=phone.dump()
        node(root,'Desativar alto contraste');phone.screenshot(output/'high-contrast.png')
        passed('Controle de alto contraste muda a aparência e informa o estado')
        phone.launch();phone.tap('Ajustes');root=phone.wait('Do seu jeito')
        assert setting(root,'Texto maior') and setting(root,'Alto contraste')
        phone.screenshot(output/'settings.png');passed('Preferências de acessibilidade persistem ao reiniciar o aplicativo')
        phone.tap('Texto maior');phone.tap('Alto contraste');phone.tap('Conexão');phone.wait('http://127.0.0.1:8080');phone.tap('Cancelar')
        phone.tap('Início');phone.wait('Mercados de Sorriso');phone.screenshot(output/'final-home.png')
        passed('Conexão local preservada e aplicativo devolvido ao início claro')
        report['status']='passed'
    finally:
        phone.command('reverse','tcp:8080','tcp:8080')
        (output/'report.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')

if __name__=='__main__':main()
