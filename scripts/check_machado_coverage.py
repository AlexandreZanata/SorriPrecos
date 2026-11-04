#!/usr/bin/env python3
"""Check the public Agile delivery selector and Clube Mais price scope."""
import http.cookiejar
import json
import re
from urllib.parse import urlencode
from urllib.request import HTTPRedirectHandler, HTTPCookieProcessor, Request, build_opener

ORIGIN = "https://www.redemachado.com.br"
SORRISO_STORE_POSTCODE = "78893110"
SORRISO_STORE_CNPJ = "03790896001110"
CLUB_PATH = "/c/1864462699260793/clube-mais"
MAX_CATALOGUE_BYTES = 2 * 1024 * 1024


class NoRedirect(HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


def json_request(session, path, *, form=None, visitor=None):
    headers = {"Accept": "application/json", "X-Requested-With": "XMLHttpRequest"}
    body = None
    if form is not None:
        headers["Content-Type"] = "application/x-www-form-urlencoded"
        headers["Referer"] = ORIGIN + "/"
        headers["X-Agile-Token"] = visitor
        body = urlencode(form).encode("ascii")
    with session.open(Request(ORIGIN + path, data=body, headers=headers), timeout=20) as response:
        payload = response.read(64 * 1024 + 1)
    if len(payload) > 64 * 1024:
        raise ValueError("Resposta acima do limite de 64 KiB")
    return json.loads(payload)


def club_catalogue_scope(session):
    with session.open(ORIGIN + CLUB_PATH, timeout=20) as response:
        payload = response.read(MAX_CATALOGUE_BYTES + 1)
    if len(payload) > MAX_CATALOGUE_BYTES:
        raise ValueError("Catálogo acima do limite de 2 MiB")
    html = payload.decode("utf-8", errors="replace")
    cnpjs = sorted({
        re.sub(r"\D", "", match)
        for match in re.findall(r"CNPJ\s*:?\s*(\d{2}\.\d{3}\.\d{3}[./]\d{4}-\d{2})", html, re.I)
    })
    price_count = len(re.findall(r'product-card__new-price[^>]*>\s*R\$', html))
    scope = "needs_branch_verification"
    if not price_count:
        scope = "no_prices"
    elif cnpjs and SORRISO_STORE_CNPJ not in cnpjs:
        scope = "other_store"
    return {
        "url": ORIGIN + CLUB_PATH,
        "visible_prices": price_count,
        "observed_cnpjs": cnpjs,
        "agile_price_branch_ids": sorted(set(re.findall(r'data-id-filial="(\d+)"', html))),
        "scope": scope,
    }


def main():
    session = build_opener(NoRedirect(), HTTPCookieProcessor(http.cookiejar.CookieJar()))
    issued = json_request(session, "/controllers/api-controller.php?acao=tokens")
    controller = issued.get("controller")
    if not isinstance(controller, str) or not controller:
        raise ValueError("Sessão anônima indisponível")
    response = json_request(
        session,
        "/controllers/order-controller.php",
        form={"acao": "consultarLocalizacoes", "cep": SORRISO_STORE_POSTCODE},
        visitor=controller,
    )
    accepted = response.get("type") == "success"
    print(json.dumps({
        "source": ORIGIN,
        "postcode": SORRISO_STORE_POSTCODE,
        "city": "Sorriso",
        "state": "MT",
        "expected_cnpj": SORRISO_STORE_CNPJ,
        "status": "needs_branch_verification" if accepted else "postcode_not_confirmed",
        "message": (
            "A vitrine aceitou o CEP. Ainda é preciso confirmar o CNPJ da filial do catálogo."
            if accepted else "A vitrine não confirmou atendimento para o CEP da filial."
        ),
        "club_catalogue": club_catalogue_scope(session),
    }, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
