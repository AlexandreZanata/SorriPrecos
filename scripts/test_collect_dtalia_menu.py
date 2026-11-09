import datetime as dt
import unittest

from collect_dtalia_menu import parse

PAGE = """
<div class="card">
  <button data-target="#collapse-1">Sabores <span>71 sabores</span> + Detalhes</button>
  <div id="collapse-1"><ul>
    <li class="list-group-item"><div class="col-12">Calabresa</div><div class="col-12"><small>Molho, mussarela e calabresa</small></div></li>
  </ul></div>
</div>
<div class="card">
  <button data-target="#collapse-2">Pizzas <span>2 itens</span> + Detalhes</button>
  <div id="collapse-2"><ul>
    <li class="list-group-item"><div class="col-12">Pizza Grande</div><div class="col-12"><small>R$ 110,00</small></div></li>
    <li class="list-group-item"><div class="col-12">Pizza Pequena</div><div class="col-12"><small>R$ 75,00</small></div></li>
  </ul></div>
</div>
<div class="card">
  <button data-target="#collapse-3">Macarrão <span>1 item</span> + Detalhes</button>
  <div id="collapse-3"><ul>
    <li class="list-group-item"><div class="col-12">Macarrão</div><div class="col-12"><small>acompanha arroz</small></div><div class="col-12"><small>R$ 99,90</small></div></li>
  </ul></div>
</div>
"""


class DtaliaMenuTest(unittest.TestCase):
    observed = dt.datetime(2026, 11, 9, 9, tzinfo=dt.timezone(dt.timedelta(hours=-4)))

    def test_parses_priced_items_and_skips_flavours(self):
        rows, coverage = parse(PAGE, self.observed)
        self.assertEqual([row['name'] for row in rows], ['Pizza Grande', 'Pizza Pequena', 'Macarrão'])
        self.assertEqual([row['price_cents'] for row in rows], [11000, 7500, 9990])
        self.assertEqual(coverage['categories'], 3)
        self.assertEqual(coverage['selected_categories'], 2)
        self.assertEqual(coverage['skipped'], ['Sabores'])
        for row in rows:
            self.assertEqual(row['branch_id'], 'pizzaria-dtalia-sorriso')
            self.assertEqual(row['category'].split(' · ')[0], 'Restaurante')
            self.assertEqual(row['source_city'], 'Sorriso')
            self.assertEqual(row['source_state'], 'MT')
            self.assertTrue(row['available'])
            self.assertTrue(row['source_id'].startswith('dtalia-'))

    def test_source_id_is_stable_for_same_category_and_name(self):
        first, _ = parse(PAGE, self.observed)
        second, _ = parse(PAGE, self.observed + dt.timedelta(hours=1))
        self.assertEqual([row['source_id'] for row in first], [row['source_id'] for row in second])

    def test_rejects_menu_without_prices(self):
        with self.assertRaisesRegex(ValueError, 'nenhum preço'):
            parse(PAGE.split('<div class="card">')[1], self.observed)


if __name__ == '__main__':
    unittest.main()
