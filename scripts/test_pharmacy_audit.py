import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
AUDIT = ROOT / "config" / "pharmacy_audit.json"
KNOWN = {
    "public_catalogue",
    "stock_only",
    "app_only",
    "app_only_local_unconfirmed",
    "public_program",
    "no_public_catalogue",
}


class PharmacyAuditTest(unittest.TestCase):
    def setUp(self):
        self.data = json.loads(AUDIT.read_text(encoding="utf-8"))
        self.rows = self.data["pharmacies"]

    def test_ids_are_unique_and_status_is_known(self):
        ids = [row["id"] for row in self.rows]
        self.assertEqual(len(ids), len(set(ids)))
        self.assertTrue({row["status"] for row in self.rows} <= KNOWN)

    def test_public_catalogue_rows_have_identity_and_page(self):
        for row in self.rows:
            if row["status"] == "public_catalogue":
                self.assertTrue(row.get("cnpj"), row["id"])
                self.assertTrue(row.get("public_page"), row["id"])

    def test_app_only_rows_have_app_link(self):
        for row in self.rows:
            if row["status"].startswith("app_only"):
                self.assertTrue(row.get("app", "").startswith("https://play.google.com/"), row["id"])


if __name__ == "__main__":
    unittest.main()
