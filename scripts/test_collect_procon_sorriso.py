import json,pathlib,unittest
from collect_procon_sorriso import match_branch,convert,cents

ROOT=pathlib.Path(__file__).resolve().parents[1]
MAPPINGS=json.loads((ROOT/'config/procon_sorriso_branches.json').read_text())

class ScopeTests(unittest.TestCase):
    def test_municipality_is_not_inferred_from_store_name(self):
        self.assertIsNone(match_branch('Sinop','AV IDEMAR RIEDI, 8150, SORRISO - MT','ATACADÃO SORRISO',MAPPINGS))
    def test_same_chain_different_branch_rejected(self):
        self.assertIsNone(match_branch('Sorriso','AV NOEMIA TONELLO DALMOLIN, 2285, SORRISO - MT','SUPERMERCADO UNIAO',MAPPINGS))
        self.assertEqual(match_branch('Sorriso','AV TANCREDO NEVES, 2122, SORRISO - MT','SUPERMERCADO UNIAO',MAPPINGS)['branch_id'],'uniao-tancredo')
    def test_wrong_state_rejected(self):
        self.assertIsNone(match_branch('Sorriso','BELO HORIZONTE 94, SORRISO - MS','LOJA 04 - SORRISO BH',MAPPINGS))
    def test_decimal_cents_are_exact(self):
        self.assertEqual(cents('R$ 69,90'),6990)
        with self.assertRaises(ValueError):cents('0,00')
    def test_source_observation_and_historical_warning_preserved(self):
        cells=['26/03/2026','22/03/2026','Sorriso','Ovo de Chocolate - Clássico','OVO PASCOA LACTA 277G SONHO DE VALSA','Não','277','R$ 69,90','R$ 0,25','LOJA 04 - SORRISO BH','AVENIDA BELO HORIZONTE, 94, SORRISO - MT']
        result,reason=convert(cells,12,MAPPINGS)
        self.assertEqual(reason,'accepted');self.assertEqual(result[0],'delmoro')
        row=result[1];self.assertEqual(row['collected_at'],1774152000);self.assertEqual(row['quantity'],277)
        self.assertIn('HISTÓRICO',row['conditions']);self.assertEqual(row['source_city'],'Sorriso')

if __name__=='__main__':unittest.main()
