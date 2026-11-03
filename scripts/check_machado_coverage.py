#!/usr/bin/env python3
"""Check store coverage for the public business postcode of Machado Sorriso."""
import http.cookiejar
import json
from urllib.parse import urlencode
from urllib.request import HTTPRedirectHandler, HTTPCookieProcessor, Request, build_opener

ORIGIN = "https://www.redemachado.com.br"
SORRISO_STORE_POSTCODE = "78893110"


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
        "status": "needs_branch_verification" if accepted else "postcode_not_confirmed",
        "message": (
            "A vitrine aceitou o CEP. Ainda é preciso confirmar o CNPJ da filial do catálogo."
            if accepted else "A vitrine não confirmou atendimento para o CEP da filial."
        ),
    }, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
