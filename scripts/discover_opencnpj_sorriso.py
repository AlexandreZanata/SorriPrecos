#!/usr/bin/env python3
"""Export local commerce candidates from a verified OpenCNPJ snapshot."""
import argparse
import json
import subprocess
from collections import Counter

FIELDS = (
    "cnpj", "nome_fantasia", "razao_social", "tipo_logradouro",
    "logradouro", "numero", "bairro", "cep", "cnae_fiscal_principal",
    "cnae_fiscal_secundaria",
)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("snapshot")
    parser.add_argument("--opencnpj", default="opencnpj")
    args = parser.parse_args()
    command = [args.opencnpj, "export", args.snapshot, "--filter", "uf=MT",
               "--filter", "municipio=9907", "--filter", "situacao=02"]
    candidates = {}
    with subprocess.Popen(command, stdout=subprocess.PIPE, text=True) as process:
        for line in process.stdout:
            row = json.loads(line)
            if (row.get("uf"), row.get("municipio"), row.get("situacao_cadastral")) != ("MT", "9907", "02"):
                raise ValueError("OpenCNPJ retornou uma filial fora do filtro local")
            codes = {row["cnae_fiscal_principal"], *row["cnae_fiscal_secundaria"].split(",")}
            if not any(code.startswith(("47", "56")) for code in codes):
                continue
            group = "pharmacy" if codes & {"4771701", "4771702", "4771703"} else "commerce"
            candidates[row["cnpj"]] = {
                **{field: row[field] for field in FIELDS},
                "group": group,
                "status": "website_unverified",
            }
        if process.wait():
            raise RuntimeError("A exportação OpenCNPJ falhou")
    print(json.dumps({
        "city": "Sorriso", "state": "MT", "municipio_rfb": "9907",
        "counts": Counter(row["group"] for row in candidates.values()),
        "candidates": list(candidates.values()),
    }, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
