import datetime as dt
import json
import unittest
from collect_atacadao_flyers import package, store_flyers, rows_from_metadata, valid_pdf_text


class FlyerTests(unittest.TestCase):
    def setUp(self):
        self.start = dt.date(2026, 11, 3)
        self.end = dt.date(2026, 11, 9)
        self.flyer = {'id':'test', 'exclude':False, 'validity':{'initial':'2026-11-03T00:00:00Z','final':'2026-11-09T00:00:00Z'}, 'urlFinalDocument':'https://apigw.cloud.carrefour.com.br/api-middleware-flyer-services/api/v2/Flyer/?id=test'}
        self.store = {'storeId':899, 'cidade':'Sorriso', 'estado':'MT', 'endereco':'Rua Idemar Riedi', 'numero':'8150', 'flyers':[self.flyer]}
    def html(self):
        return '<script id="__NEXT_DATA__" type="application/json">'+json.dumps({'props':{'pageProps':{'storeInfo':self.store}}})+'</script>'
    def test_rejects_wrong_location_and_expired_flyer(self):
        self.assertEqual(len(store_flyers(self.html(), self.start)), 1)
        self.assertEqual(store_flyers(self.html(), dt.date(2026,11,10)), [])
        self.store['cidade'] = 'Sinop'
        with self.assertRaises(ValueError): store_flyers(self.html(), self.start)
    def test_rejects_ambiguous_packages(self):
        self.assertEqual(package(['Arroz','Pacote com 5kg']), (5000,'g','Pacote com 5kg'))
        self.assertEqual(package(['Queijo','Preço por quilo na peça']), (1,'kg','Preço por quilo na peça'))
        for p in (['Hambúrguer','Caixeta com 36x56g'], ['Chocolate','Vários sabores','Pacote com 150g'], ['Kit','300ml + 150ml'], ['Atum','Natural/ Óleo','Lata com 120g'], ['Cachaça','Com casco','Garrafa com 965ml']):
            self.assertIsNone(package(p))
    def test_requires_location_dates_and_three_tier_legend(self):
        text='SORRISO/MT: Rua Idemar Riedi, 8150 PREÇO VAREJO/ATACADO PREÇO COM OFERTA ATIVADA NO CARTÃO ATACADÃO OFERTAS VÁLIDAS DE 03/11 A 09/11/2026'
        self.assertTrue(valid_pdf_text(text,self.start,self.end))
        self.assertFalse(valid_pdf_text(text.replace('SORRISO/MT','SINOP/MT'),self.start,self.end))
        self.assertFalse(valid_pdf_text(text.replace('ATIVADA NO',''),self.start,self.end))
    def test_conditional_prices_are_not_unconditional_availability(self):
        f={'P':{'texto':'R$18,50'},'PSEG':{'texto':'R$17,50'},'PTER':{'texto':'R$14,90'},'QUNTOFERTAO':{'texto':'Máximo de 6 un. por cliente no APP e cartão'}}
        meta={'paginas':[{'pagina':1,'itens':[{'numero':1,'descricao':{'texto':'Arroz Rei\nPacote com 5kg'},'campos':f}]}]}
        rows=rows_from_metadata(meta,self.flyer,self.end,1793770000)
        self.assertEqual(len(rows),1);row=rows[0]
        self.assertEqual((row['price_cents'],row['promotional_price_cents'],row['offer_type']), (1850,1750,'club'))
        self.assertIsNone(row['available']);self.assertIn('R$ 14,90',row['conditions'])
        f.pop('PTER');self.assertEqual(rows_from_metadata(meta,self.flyer,self.end,1793770000),[])

if __name__ == '__main__': unittest.main()
