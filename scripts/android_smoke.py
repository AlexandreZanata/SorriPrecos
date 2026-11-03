#!/usr/bin/env python3
"""Test the installed app on a named physical Android device, preserving app data.
The script temporarily removes ONLY the USB mapping for port 8080 to test offline.
"""
import argparse,json,os,pathlib,re,subprocess,time,xml.etree.ElementTree as ET

ROOT=pathlib.Path(__file__).resolve().parents[1]
PACKAGE='br.com.sorriprecos'

class Phone:
    def __init__(self,serial):
        self.adb=[os.environ.get('ADB','adb'),'-s',serial]
        self.width,self.height=map(int,re.findall(r'(\d+)x(\d+)',self.command('shell','wm','size').decode())[-1])
    def command(self,*args):return subprocess.check_output(self.adb+list(args),stderr=subprocess.STDOUT)
    def dump(self):
        self.command('shell','uiautomator','dump','/sdcard/sorriprecos-ui.xml')
        root=ET.fromstring(self.command('shell','cat','/sdcard/sorriprecos-ui.xml'))
        if not any(n.attrib.get('package')==PACKAGE for n in root.iter('node')):raise RuntimeError('SorriPreços não está visível; desbloqueie o celular e abra o app')
        return root
    @staticmethod
    def texts(root):return [n.attrib.get('text','') for n in root.iter('node') if n.attrib.get('package')==PACKAGE and n.attrib.get('text')]
    def wait(self,text,timeout=20):
        deadline=time.monotonic()+timeout
        while time.monotonic()<deadline:
            root=self.dump()
            if any(text in s for s in self.texts(root)):return root
            time.sleep(.3)
        raise AssertionError(f'Texto não apareceu: {text}; {self.texts(root)}')
    def tap(self,text):
        root=self.dump()
        nodes=[n for n in root.iter('node') if n.attrib.get('package')==PACKAGE and (n.attrib.get('text')==text or n.attrib.get('content-desc')==text)]
        if not nodes:raise AssertionError(f'Controle não encontrado: {text}')
        bounds=list(map(int,re.findall(r'\d+',nodes[0].attrib['bounds'])))
        x1,y1,x2,y2=bounds;self.command('shell','input','tap',str((x1+x2)//2),str((y1+y2)//2))
    def launch(self):
        self.command('shell','am','force-stop',PACKAGE)
        self.command('shell','am','start','-W','-n',PACKAGE+'/.MainActivity')
        return self.wait('SorriPreços')
    def screenshot(self,path):path.write_bytes(self.command('exec-out','screencap','-p'))

def main():
    parser=argparse.ArgumentParser();parser.add_argument('serial');parser.add_argument('--output',type=pathlib.Path,default=ROOT/'artifacts/android-smoke');args=parser.parse_args()
    args.output.mkdir(parents=True,exist_ok=True);phone=Phone(args.serial)
    report={'device_model':phone.command('shell','getprop','ro.product.model').decode().strip(),'android':phone.command('shell','getprop','ro.build.version.release').decode().strip(),'tests':[]}
    def passed(name):report['tests'].append({'name':name,'status':'passed'});print('PASS:',name,flush=True)
    phone.command('reverse','tcp:8080','tcp:8080')
    try:
        root=phone.launch()
        if not any('MODO DEMO' in t for t in phone.texts(root)):
            phone.tap('Ajustes');root=phone.wait('Do seu jeito')
            phone.command('shell','input','swipe',str(phone.width//2),str(int(phone.height*.75)),str(phone.width//2),str(int(phone.height*.45)),'350');root=phone.dump()
            for _ in range(5):
                if 'Modo DEMO' in phone.texts(root):break
                phone.command('shell','input','swipe',str(phone.width//2),str(int(phone.height*.8)),str(phone.width//2),str(int(phone.height*.4)),'350');root=phone.dump()
            phone.tap('Modo DEMO');phone.tap('Início');phone.wait('MODO DEMO');phone.launch()
        phone.screenshot(args.output/'home.png');passed('Inicialização e identificação explícita de DEMO')
        phone.tap('Arroz');root=phone.wait('R$ 22,70')
        assert not any('1 kg' in t for t in phone.texts(root))
        phone.screenshot(args.output/'search.png');passed('Busca API por arroz 5kg sem misturar embalagem de 1kg')
        phone.tap('Comparar ›');root=phone.wait('Mercado Cerrado · DEMO')
        texts=phone.texts(root)
        assert texts.index('R$ 22,70')<texts.index('R$ 23,90')
        assert 'Filial de demonstração' in texts
        phone.screenshot(args.output/'comparison.png');passed('Comparação ordenada com filial e data')
        if '♡ Salvar nos favoritos' in texts:phone.tap('♡ Salvar nos favoritos')
        phone.wait('♥ Salvo nos favoritos');phone.tap('Voltar');phone.tap('Favoritos');phone.wait('Arroz branco DEMO')
        phone.screenshot(args.output/'favorites.png');passed('Favorito salvo localmente')
        phone.launch();phone.tap('Favoritos');phone.wait('Arroz branco DEMO');passed('Favorito persiste após reiniciar o processo')
        phone.tap('Arroz branco DEMO');phone.wait('Mercado Cerrado · DEMO')
        phone.tap('Detalhes');phone.wait('A mesma embalagem, sempre');phone.screenshot(args.output/'details.png');passed('Detalhes e natureza sintética do produto')
        phone.tap('Preços')
        phone.command('reverse','--remove','tcp:8080')
        phone.tap('Atualizar');phone.wait('Offline · Comparação salva',timeout=25)
        phone.screenshot(args.output/'offline.png');passed('Comparação em cache identificada offline sem alterar Wi-Fi/dados do aparelho')
        phone.command('reverse','tcp:8080','tcp:8080');phone.tap('Atualizar');phone.wait('Mercado Cerrado · DEMO')
        # Give the request an opportunity to replace the cached response, checking only our app.
        deadline=time.monotonic()+15
        while any('Offline · Comparação salva' in t for t in phone.texts(phone.dump())):
            if time.monotonic()>deadline:raise AssertionError('App não recuperou conexão')
        phone.screenshot(args.output/'reconnected.png');passed('Reconexão ao backend após restaurar USB')
        # Verify Android system Back returns from comparison to the favorites screen.
        phone.command('shell','input','keyevent','4');phone.wait('Sua lista de favoritos');passed('Botão Voltar do Android')
        phone.tap('Início');phone.tap('Arroz');phone.wait('R$ 22,70');phone.tap('Comparar ›');phone.wait('Mercado Cerrado · DEMO')
        report['status']='passed'
    finally:
        phone.command('reverse','tcp:8080','tcp:8080')
        (args.output/'report.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps(report,ensure_ascii=False),flush=True)

if __name__=='__main__':main()
