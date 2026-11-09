# Aplicativos de Sorriso descompilados

Revisão de 9 de novembro de 2026. Os APKs foram obtidos do aparelho físico por ADB e
inspecionados com JADX/REA. A descompilação serviu para localizar a origem pública dos
dados. Nenhum coletor contorna CAPTCHA, 403, login ou medida de segurança.

## Farma & Farma e FarMelhor (Febrafar eDelivery)

- Pacotes: `br.com.febrafar.edelivery.farmaefarma` e
  `br.com.febrafar.edelivery.farmelhor`.
- Os aplicativos são invólucros WebView. O endereço carregado é
  `https://farmaefarma.edelivery.febrafar.com.br` e
  `https://farmelhor.edelivery.febrafar.com.br`.
- A API pública usada pela própria vitrine é same-origin:
  `/api/stores/search`, `/api/stores/{id}`, `/api/stores/open/{id}`,
  `/api/products/search`, `/api/categories` e `/api/session`.
- **Farma & Farma.** A busca pública devolveu a loja `Farma Sorriso`,
  ID `54a3138b-3fd2-49a5-8320-2d898a16712a`, CNPJ 26.616.675/0001-75,
  Avenida Natalino João Brescansin, 451. A loja publica
  `modules.sales=false`, `modules.offers=false` e `available=false`.
  Não há catálogo de venda nessa unidade.
- **FarMelhor.** A rede respondeu com 58 lojas no mecanismo de busca
  regional. Nenhuma fica em Sorriso ou no Mato Grosso. A unidade de
  Avenida Tancredo Neves, 2302 não está cadastrada na vitrine eDelivery.
- A busca de lojas exige verificação Turnstile. O projeto não implementou
  contorno; por isso os dois aplicativos continuam sem coletor automático.
- Conclusão: não há produto/preço local atribuível a Sorriso por esses canais.

## Farmácia Madeira (Remédio Brasil / RetailOn)

- Pacote: `com.inteliger.madeira.app`.
- O aplicativo é um cliente Kotlin Multiplatform/RetailOn que consome a
  vitrine `https://app.farmaciamadeira.com.br` e o marketplace
  `https://app.remediobrasil.com.br`.
- Rotas públicas identificadas: `POST /core/produto/buscar/precos`,
  `/layout/menu`, `/produto/validar/entrega/cep` e o HTML de listagem e
  detalhe de produto.
- O endpoint de preços recebe `produtos`, `filial_id` e `estado`, além do
  token CSRF da sessão anônima. A página `/nossas-lojas` lista Sinop/MT; a
  validação de entrega para o CEP 78890-000 devolveu a loja de Sinop.
- Conclusão: a vitrine é pública, mas não publica preço/estoque vinculado à
  unidade de Sorriso. Não foi criado coletor para uma filial que a origem
  não identifica.

## Machado Supermercados (Agile E-commerce)

- Pacote: `br.com.redemachado`, versão 1.2, instalado no aparelho físico em
  09/11/2026 a partir da Play Store. O APK foi puxado por ADB e o bundle
  React Native aponta para `https://www.redemachado.com.br`.
- O aplicativo carrega o mesmo catálogo da web, sem seletor de filial. A
  vitrine e o app identificam a filial 3, CNPJ 03.790.896/0003-00, em
  Sinop. O app `Clube + Machado Supermercados`
  (`br.com.gsgroup.zoombox.machadoprimaveras`) é o programa de
  relacionamento de Sinop.
- A consulta de CEP 78893-110 para a unidade Tancredo Neves continua
  recusada. Não há preço público atual atribuível a Sorriso.
- Conclusão: nenhum coletor foi criado; a filial permanece como coleta
  pendente, sem reutilizar os preços de Sinop.

## Pizzaria D'Tália Sorriso

- Pacote: `com.pizzariadtalia.sorriso` (Expo/React Native).
- O código do aplicativo aponta para `sorriso.pizzariadtalia.com`; a vitrine
  pública `https://sorriso.pizzariadtalia.com/cardapio` responde sem login,
  CAPTCHA ou 403.
- A página publica categorias, itens e preços em HTML server-side. A lista
  de sabores não tem preço e é descartada. O cardápio real rendeu 50 itens
  com preço em 8 categorias; a categoria "Sabores" ficou fora.
- Coletor novo: `scripts/collect_dtalia_menu.py --watch`. Fonte
  `pizzaria-dtalia`, filial `pizzaria-dtalia-sorriso`, intervalo de seis
  horas. O coletor marca como indisponível o item que sumiu do cardápio.
- O preço é uma observação datada do cardápio online. Disponibilidade,
  entrega e retirada precisam de confirmação na loja.

## Aplicativos sem catálogo utilizável

- `com.zefominha.customer` (FoodKing): o código Flutter instalado aponta para
  `http://10.0.2.2:8090`, endereço de desenvolvimento. Não foi localizado um
  servidor de produção público no APK.
- `com.vamu.app` (Expo): aplicativo de entregador, sem catálogo de produtos.
- `com.sorrimobi`, `com.cidadeinteligente.app` e `com.guiacidade.app`:
  aplicativos de serviços/cidade, sem vitrine de produto/preço por filial
  validada nesta revisão.

## Regra de integração

Uma fonte só entra no ingestor quando a origem pública devolve identidade da
filial de Sorriso e produto/preço sem contornar proteção. A ausência desses
elementos fica registrada como cobertura pendente, não como oferta.
