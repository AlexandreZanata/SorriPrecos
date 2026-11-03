#!/usr/bin/env python3
"""Load the small verified public historical snapshot. Never changes its dates."""
import argparse,json,pathlib,subprocess
ROOT=pathlib.Path(__file__).resolve().parents[1]

def main():
    p=argparse.ArgumentParser();p.add_argument('--database-url',default='sqlite://data/sorriprecos.db');p.add_argument('--binary',type=pathlib.Path,default=ROOT/'target/release/sorriprecos');a=p.parse_args()
    folder=ROOT/'datasets/procon-2026-sorriso'
    report=json.loads((folder/'report.json').read_text())
    if report['scope']!={'city':'Sorriso','state':'MT'}:raise SystemExit('Escopo não autorizado')
    for source in report['markets']:
        subprocess.run([str(a.binary),'--database-url',a.database_url,'import-public-report','--source',source,'--file',str(folder/f'{source}.json')],cwd=ROOT,check=True)
    print('Registros históricos de Sorriso-MT carregados. Nenhuma oferta atual foi criada.')

if __name__=='__main__':main()
