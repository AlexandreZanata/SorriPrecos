import datetime as dt
import hashlib
import json
import unittest

from collect_casa_aurora_flyer import TZ, verified_rows


class CasaAuroraFlyerTest(unittest.TestCase):
    image = b'official image fixture'
    url = 'https://casaaurora.com.br/wp-content/uploads/2026/11/quarta.jpg'
    observed = dt.datetime(2026, 11, 4, 9, tzinfo=TZ)

    def download(self, url, _limit):
        if url == self.url:
            return self.image
        return json.dumps({'id': 3864, 'source_url': self.url}).encode()

    def manifest(self):
        return {
            'media_id': 3864, 'image_sha256': hashlib.sha256(self.image).hexdigest(),
            'valid_from': '2026-11-04', 'valid_through': '2026-11-04',
            'items': [{'id': 'coxao-duro', 'name': 'Coxão Duro', 'price_cents': 4490}],
        }

    def test_imports_reviewed_price_per_kg_without_stock(self):
        row, = verified_rows(self.manifest(), self.observed, self.download)
        self.assertEqual(row['branch_id'], 'casa-aurora-centro')
        self.assertEqual((row['quantity'], row['unit'], row['price_cents']), (1, 'kg', 4490))
        self.assertEqual(row['source_url'], self.url)
        self.assertIsNone(row['available'])
        valid_until = dt.datetime.fromtimestamp(row['valid_until'], TZ)
        self.assertEqual((valid_until.date(), valid_until.hour), (self.observed.date(), 23))

    def test_rejects_changed_art_or_expired_review(self):
        manifest = self.manifest()
        manifest['image_sha256'] = '0' * 64
        with self.assertRaisesRegex(ValueError, 'arte oficial mudou'):
            verified_rows(manifest, self.observed, self.download)
        with self.assertRaisesRegex(ValueError, 'fora da validade'):
            verified_rows(manifest, self.observed + dt.timedelta(days=1), self.download)
