#!/usr/bin/env python3
"""Online SQLite backup; restore into a NEW path, preserving the original database."""
import argparse,pathlib,sqlite3

def copy_database(source,target):
    source=pathlib.Path(source).resolve();target=pathlib.Path(target).resolve()
    if not source.is_file():raise SystemExit('Banco de origem inexistente')
    if target.exists():raise SystemExit('Destino já existe. Use outro caminho para preservar o banco anterior.')
    target.parent.mkdir(parents=True,exist_ok=True)
    with sqlite3.connect(source.as_uri()+'?mode=ro',uri=True) as src,sqlite3.connect(target) as dst:
        if src.execute('PRAGMA integrity_check').fetchone()[0]!='ok':raise SystemExit('Origem inválida')
        src.backup(dst,pages=256)
        if dst.execute('PRAGMA integrity_check').fetchone()[0]!='ok':raise SystemExit('Cópia inválida')
    print(f'Cópia verificada: {target}')

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('mode',choices=['backup','restore']);p.add_argument('source');p.add_argument('destination');a=p.parse_args();copy_database(a.source,a.destination)
