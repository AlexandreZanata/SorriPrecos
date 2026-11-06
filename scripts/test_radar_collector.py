import importlib.util
import sqlite3
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("radar_collector", ROOT / "scripts" / "collect_radar_farmacias.py")
RADAR = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(RADAR)


class RadarCollectorTest(unittest.TestCase):
    def test_package_uses_the_last_explicit_pack(self):
        self.assertEqual(RADAR.package("Dipirona 500mg EMS 10 Comprimidos"), (10, "un"))
        self.assertEqual(RADAR.package("Fórmula Infantil Ninho 800g"), (800, "g"))
        self.assertEqual(RADAR.package("Soro Fisiológico 500ml"), (500, "ml"))
        self.assertEqual(RADAR.package("Atadura 10cm x 1,80m"), (1, "un"))

    def test_gtin_validation(self):
        self.assertTrue(RADAR.valid_gtin("7891000062661"))
        self.assertFalse(RADAR.valid_gtin("7891000062662"))
        self.assertFalse(RADAR.valid_gtin(""))

    def test_row_keeps_unavailable_and_unpriced_products(self):
        row, error = RADAR.row(
            {"id": "p", "name": "Produto 20g", "available": False, "displayPrice": None, "codigoBarras": None},
            "branch",
            100,
        )
        self.assertIsNone(error)
        self.assertEqual(row["available"], False)
        self.assertIsNone(row["price_cents"])

    def test_presentation_follows_existing_gtin_product(self):
        with tempfile.TemporaryDirectory() as temp:
            database = Path(temp) / "catalog.db"
            with sqlite3.connect(database) as db:
                db.execute("CREATE TABLE products(gtin TEXT,quantity INTEGER,unit TEXT,presentation TEXT,is_demo INTEGER)")
                db.execute("INSERT INTO products VALUES('07891000062661',800,'g','unidade de venda',0)")
            rows = [{"gtin": "7891000062661", "quantity": 800, "unit": "g", "presentation": "unidade de venda"}]
            RADAR.align_presentations(rows, str(database))
            self.assertEqual(rows[0]["presentation"], "unidade de venda")
            rows[0].update({"quantity": 900, "gtin": "7891000062661"})
            RADAR.align_presentations(rows, str(database))
            self.assertIsNone(rows[0]["gtin"])


if __name__ == "__main__":
    unittest.main()
