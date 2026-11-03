#!/usr/bin/env python3
"""List the public storefront candidates linked from the Sorriso directory."""
import json
import re
from html.parser import HTMLParser
from urllib.parse import urljoin, urlsplit
from urllib.request import urlopen

DIRECTORY = "https://aiqfome.com/restaurantes/sorriso-MT"
MAX_BYTES = 32 * 1024 * 1024


class StorefrontLinks(HTMLParser):
    def __init__(self):
        super().__init__()
        self.paths = set()

    def handle_starttag(self, tag, attrs):
        href = dict(attrs).get("href")
        if tag != "a" or not href:
            return
        url = urlsplit(urljoin(DIRECTORY, href))
        if (url.scheme == "https" and url.netloc in {"aiqfome.com", "www.aiqfome.com"}
                and re.fullmatch(r"/MT/sorriso/[a-z0-9-]+/?", url.path)):
            self.paths.add(url.path.rstrip("/"))


def main():
    with urlopen(DIRECTORY, timeout=25) as response:
        body = response.read(MAX_BYTES + 1)
    if len(body) > MAX_BYTES:
        raise ValueError("Diretório excede 32 MiB")
    parser = StorefrontLinks()
    parser.feed(body.decode("utf-8"))
    print(json.dumps({
        "directory": DIRECTORY,
        "city": "Sorriso",
        "state": "MT",
        "candidates": ["https://aiqfome.com" + path for path in sorted(parser.paths)],
    }, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
