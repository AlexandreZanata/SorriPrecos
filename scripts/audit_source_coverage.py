#!/usr/bin/env python3
"""Audit aiqfome category coverage and optionally align config with every published category."""
import argparse
import gzip
import json
import re
import time
import unicodedata
import urllib.error
import urllib.request
from pathlib import Path

USER_AGENT = "SorriPrecos/0.1 (public-storefront-coverage-audit)"
MAX_BYTES = 16 * 1024 * 1024
EXCLUDED = {
    "baralho",
    "cigarros",
    "cigarros e paieiros",
    "cigarro",
    "complementos e utensilios",
    "copos",
    "couvert artistico",
    "fumo",
    "isqueiro",
    "isqueiro e baralho",
    "narguile",
    "originais do aiqfome",
    "paieiro",
    "polenteiras eletricas bivolt",
    "presentes",
    "sabores",
    "tabacaria",
    "utensilios",
}


def norm(value):
    value = unicodedata.normalize("NFD", value)
    value = "".join(char for char in value if not unicodedata.combining(char))
    return " ".join("".join(char if char.isalnum() else " " for char in value.lower()).split())


def entries(value):
    if isinstance(value, list):
        return value
    if isinstance(value, dict):
        return list(value.values())
    return []


def fetch(url):
    for attempt in range(4):
        request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT, "Accept-Encoding": "gzip"})
        try:
            with urllib.request.urlopen(request, timeout=60) as response:
                body = response.read(MAX_BYTES + 1)
                if response.headers.get("Content-Encoding") == "gzip":
                    body = gzip.decompress(body)
                if len(body) > MAX_BYTES:
                    raise ValueError("página acima de 16 MiB")
                return body.decode("utf-8", "replace")
        except urllib.error.HTTPError as error:
            if error.code not in (429, 503) or attempt == 3:
                raise
            retry = error.headers.get("Retry-After")
            time.sleep(max(10, int(retry)) if retry and retry.isdigit() else 10 * (attempt + 1))
    raise RuntimeError("limite de tentativas excedido")


def catalogue(html):
    marker = "const restaurantenovo = "
    start = html.find(marker)
    if start < 0:
        raise ValueError("catálogo público ausente")
    return json.JSONDecoder().raw_decode(html[start + len(marker) :])[0]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--config", type=Path, default=Path("config/aiqfome.json"))
    parser.add_argument("--report", type=Path, default=Path("artifacts/source-coverage.json"))
    parser.add_argument("--update", action="store_true", help="grava todas as categorias não excluídas")
    parser.add_argument("--delay", type=float, default=1.2)
    args = parser.parse_args()
    sources = json.loads(args.config.read_text(encoding="utf-8"))
    report = {"checked_at": int(time.time()), "excluded_normalized": sorted(EXCLUDED), "sources": []}
    failed = 0
    for source in sources:
        if source.get("provider", "aiqfome") != "aiqfome":
            continue
        url = f"https://aiqfome.com/MT/sorriso/{source['slug']}"
        try:
            page = catalogue(fetch(url))
            seller = page.get("id")
            city = (page.get("cidade") or {}).get("nome")
            if seller != source["seller_id"] or city != "Sorriso":
                raise ValueError(f"identidade divergente: seller={seller} city={city}")
            published = [entry.get("nome", "").strip() for entry in entries(page["menu"]["cardapios_categorias"])]
            selected = {norm(name) for name in source.get("categories", [])}
            selected_names = [name for name in published if not selected or norm(name) in selected]
            excluded = [name for name in published if norm(name) in EXCLUDED]
            missing = [
                name
                for name in published
                if (not selected or norm(name) not in selected) and norm(name) not in EXCLUDED
            ]
            all_allowed = [name for name in published if norm(name) not in EXCLUDED]
            items = sum(len(entries(category.get("itens", []))) for category in entries(page["menu"]["cardapios_categorias"]))
            variants = sum(
                len(entries(item.get("itens_tamanhos", [])))
                for category in entries(page["menu"]["cardapios_categorias"])
                for item in entries(category.get("itens", []))
            )
            report["sources"].append(
                {
                    "supermarket_id": source["supermarket_id"],
                    "seller_id": seller,
                    "url": url,
                    "published_categories": len(published),
                    "configured_categories": len(selected_names),
                    "missing_consumable_categories": missing,
                    "excluded_categories": excluded,
                    "items": items,
                    "variants": variants,
                }
            )
            if missing:
                print(f"{source['supermarket_id']}: {len(missing)} categorias publicadas fora da configuração")
            if args.update:
                source["categories"] = [norm(name) for name in all_allowed]
        except Exception as error:
            failed += 1
            report["sources"].append({"supermarket_id": source.get("supermarket_id"), "url": url, "error": str(error)})
            print(f"{source.get('supermarket_id')}: FALHA {error}")
        time.sleep(args.delay)
    report["failed"] = failed
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    if args.update:
        args.config.write_text(json.dumps(sources, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"fontes auditadas: {len(report['sources'])}; falhas: {failed}; relatório: {args.report}")
    raise SystemExit(1 if failed else 0)


if __name__ == "__main__":
    main()
