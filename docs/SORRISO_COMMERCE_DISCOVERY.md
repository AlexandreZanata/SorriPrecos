# Descoberta de comércio eletrônico em Sorriso-MT

O [OpenCNPJ](https://github.com/AlexandreZanata/OpenCNPJ) é o ponto de partida para conferir CNPJ e endereço. Com o snapshot 2026-10, o filtro de estabelecimentos ativos no município RFB 9907 devolveu 23.134 registros. As divisões 45 (veículos), 47 (varejo) e 56 (alimentação), no CNAE principal ou secundário, selecionaram 8.011 candidatos únicos, incluindo 60 com CNAEs 4771701, 4771702 ou 4771703 de farmácia humana. A triagem cadastral não comprova a existência de loja virtual.

```sh
python3 scripts/discover_opencnpj_sorriso.py /caminho/snapshot.ocnpj \
  --opencnpj /caminho/opencnpj > /tmp/sorriso-candidatos.json
```

1. Cruze o CNPJ completo com o rodapé, endereço de retirada ou identificador de filial exposto pela vitrine. Para redes, confira também a loja ou o CEP usado na resposta que contém a oferta.
2. Abra o catálogo público como visitante e confira a resposta usada pela vitrine para exibir produto, preço e disponibilidade. Use a navegação pública e a sessão anônima da própria loja quando aplicável.
3. Vincule o preço à identidade da loja virtual ou da filial verificada. Confira SKU, apresentação, quantidade, preço em reais, disponibilidade e URL. Valores sob consulta e opções obrigatórias sem preço final ficam fora. Estoque desconhecido não entra no ranking.
4. Registre a evidência por fonte, teste em SQLite isolado e compare o resultado com a vitrine antes de ativar a coleta principal.

## Farmácias verificadas

- [Therapeutica](https://www.therapeuticafarmacia.com.br/) tem uma loja virtual vinculada ao CNPJ ativo 01.780.384/0001-75, Avenida Curitiba, 2734, Sorriso-MT. O cadastro fiscal e a vitrine usam nomes diferentes para o bairro, com coincidência de CNPJ, logradouro, número e CEP. As páginas públicas têm preço e estoque em JSON-LD. Itens [sob encomenda](https://www.therapeuticafarmacia.com.br/actrisave) podem expor valores simbólicos de R$ 0,01 ou R$ 1,00 no HTML. O coletor descarta essa faixa de preços desta fonte.
- [Farma & Farma no aiqfome](https://aiqfome.com/MT/sorriso/farma-farma) identifica a unidade de Sorriso (26.661.675/0001-75), mas a vitrine consultada devolveu zero categorias.
- [FarMelhor Centro](https://unidades.franquiafarmelhor.com.br/farmelhor-centro-sorriso-mt-73568) tem páginas locais de produtos sem preço público verificável.
- [Drogaria Nebrasca](https://drogarianebrasca24horas.com.br/contato/), [Farmácia Aurora](https://www.farmaciaaurora.com.br/sorriso-mt/) e [Fórmula Certa](https://www.formulacertasorriso.com.br/) têm presença local. As páginas consultadas direcionam para atendimento ou orçamento.
- [Pague Menos](https://www.paguemenos.com.br/), [Drogaria São Paulo](https://www.drogariasaopaulo.com.br/) e [Drogasil](https://www.drogasil.com.br/) têm filiais ativas no OpenCNPJ e catálogos nacionais. Falta comprovar a identidade local na resposta pública que contém preço e estoque. Para Ultra Popular, o cadastro confirmou filiais locais e ainda falta comprovar uma vitrine local com preços.

## Coleta

```sh
cargo run -- collect-pharmacy --limit 40
cargo run -- collect-pharmacy --limit 40 --watch
```

O coletor lê o sitemap e visita um lote de até 40 páginas por ciclo de dez minutos. O cursor fica no SQLite. O ciclo seguinte continua a varredura e os preços coletados expiram em quatro horas. HTTP 429 interrompe a rodada e respeita pelo menos dez minutos ou um `Retry-After` maior. A periodicidade é por lote. A cobertura completa depende do tamanho e da disponibilidade da vitrine.

O app oferece Mercado e Farmácia como buscas separadas. A API de busca recebe `sector=pharmacy` para a farmácia. O padrão continua sendo Mercado.

## Validação individual por vitrine

A rodada de 2026-11-03 acrescentou a divisão 45 (comércio e reparação de veículos e motocicletas) à triagem, pois o CNAE varejista de motopeças 4541206 não aparece nas divisões 47 e 56. Com o mesmo snapshot, a fila passa a **8.011 CNPJs**, incluindo os mesmos 60 candidatos farmacêuticos. Atividades secundárias e a divisão 45 podem produzir candidatos sem venda ao consumidor. A fila indica o universo cadastral inicial, ainda em revisão individual.

### Vitrine Aqui — cinco lojas de Sorriso

O [diretório público](https://vitrineaqui.com/lojas) usa a cidade selecionada. O HTML de cada loja publica um objeto JSON-LD `Store` com o endereço em Sorriso-MT. Cada página de produto publica JSON-LD `Product` com SKU, preço, imagem, disponibilidade geral e a URL de `offers.seller`. As amostras abaixo tiveram resposta HTTP 200 na consulta direta, sem login.

- **Cris Moda Evangélica · 24.** Rua Vale Dourado, 545, Rota do Sol. O OpenCNPJ tem um registro ativo no endereço e CEP publicados, mas a vitrine não mostra CNPJ para concluir a atribuição. A amostra Vestido Feminino retornou SKU 133 e R$ 310,00. [Loja](https://vitrineaqui.com/sorriso/loja/24-cris-moda-evangelica) · [Produto](https://vitrineaqui.com/produto-vestido-feminino)
- **Seja Girassol · 25.** Rua Ametista, 100, Rota do Sol. O identificador 25 permanece no slug antigo `lua-doce-presentes`. O nome da loja atual e o vendedor do produto coincidem. CNPJ pendente. A amostra Conjunto retornou SKU 195 e R$ 139,90. [Loja](https://vitrineaqui.com/sorriso/loja/25-lua-doce-presentes) · [Produto](https://vitrineaqui.com/produto-conjunto-195)
- **PK Store · 26.** Rua São Silvestre, 1703, Flor do Cerrado. O registro cadastral encontrado no endereço é de outra marca, portanto a correspondência fiscal segue pendente. A amostra Camisa social quadriculada retornou SKU 128 e R$ 219,90. [Loja](https://vitrineaqui.com/sorriso/loja/26-pk-store) · [Produto](https://vitrineaqui.com/produto-camisa-social-quadriculada)
- **Carona Moda Fitness · 27.** Avenida Natalino João Brescansin, 915, Centro Sul. Nome empresarial, logradouro e número coincidem com 68.650.305/0001-82 no OpenCNPJ, com divergência de CEP. A amostra Conjunto retornou SKU 166 e R$ 65,00. O JSON-LD anuncia estoque geral, mas a página exibe variantes esgotadas. A disponibilidade por cor e tamanho exige validação própria. [Loja](https://vitrineaqui.com/sorriso/loja/27-carona-moda-fitness) · [Produto](https://vitrineaqui.com/produto-conjunto-166)
- **Personalizado Ferreira · 28.** Travessa São Vicente, 70, São Mateus. O cadastro localizado no endereço tem outro ramo e o CNPJ do vendedor segue pendente. A amostra Oratório retornou SKU 190 e R$ 40,00. [Loja](https://vitrineaqui.com/sorriso/loja/28-personalizado-ferreira) · [Produto](https://vitrineaqui.com/produto-oratorio-190)

O frontend usa o seguinte POST de leitura. O domínio respondeu JSON a cada um dos cinco IDs sem cookies ou token. A primeira página inicia em zero. Compare `total`, `totalPaginas` e `pagina`, confira `storeUrl` em cada item e valide também a página `Store` do vendedor. A paginação completa devolveu 92 SKUs únicos com preço positivo e URL da loja esperada. Foram 34 da Cris, 3 da Seja Girassol, 4 da PK Store, 26 da Carona e 25 da Personalizado Ferreira. O retorno oferece `price`, `promoPrice`, `finalPrice`, estoque agregado e imagem. Itens com variante obrigatória precisam do preço final e da disponibilidade da combinação.

```sh
id=27
page=0
curl -fsS -H 'X-Requested-With: XMLHttpRequest' \
  --data "busca=&buscar=&pagina=$page&id_loja=$id&loja=$id&categoria=&subcategoria=&format=json" \
  'https://vitrineaqui.com/ajax/listar_produtos.php?v=loja-busca-3'
```

### Lojas independentes e casos pendentes

- **Steak House 67 · 38.289.159/0001-40.** Avenida Jardim dos Imigrantes, 4363, no OpenCNPJ. O CNPJ consta no rodapé da loja Wix. O GET de `/shop` devolveu HTTP 200 e mostrou Bacon Fuego a R$ 24,90. O HTML e o estado público Wix concordaram quanto ao preço, e o estado trouxe `isInStock: true`. A loja de produtos e o cardápio do restaurante são fluxos distintos. [Loja 67](https://www.steakhouse67.com/shop)
- **Atacadão Moto Peças · 34.242.171/0001-49.** Avenida Perimetral Sudeste, 10355. CNPJ e endereço da matriz no rodapé e no OpenCNPJ. O GET do produto devolveu HTTP 200 e JSON-LD com SKU 3671. A vitrine apresenta a lente a R$ 3,15. A oferta pode ser atribuída à loja virtual da empresa de Sorriso, enquanto o estoque de balcão por filial da rede permanece pendente. [Rede e filiais](https://www.atacadaomotopeca.com.br/nossas-lojas) · [Produto](https://www.atacadaomotopeca.com.br/produtos/plasticos/lentes-do-pisca/lente-de-pisca-agrale-cristal-transparente-stlu-para-reposicao)
- **Eletromais · 23.232.816/0001-46.** Avenida Perimetral Sudeste, 11479. CNPJ e endereço coincidem com a vitrine, que anuncia retirada na loja. A página indexada exibe uma lâmpada Stella de 9 W a R$ 13,50. A consulta direta à página do item e às páginas iniciais retornou HTTP 403. A rota de leitura ao vivo permanece pendente. [Vitrine](https://eletromaismt.com.br/)
- **Bellissima · 30.051.974/0001-65.** Avenida Tancredo Neves, 244, confirmada no site, no OpenCNPJ e no diretório municipal. A página indexada de uma planta traz texto em inglês, crédito a uma loja estrangeira e preço de R$ 69,00. O catálogo respondeu HTTP 406. A oferta e a disponibilidade comercial seguem pendentes. [Diretório municipal](https://turismo.sorriso.mt.gov.br/produtos-servicos/bem-estar/floricultura-bellissima--flores-e-presentes) · [Página examinada](https://bellissimafloricultura.com.br/produto/neanthe-bella-palm/)
- **Mariná Flores · 16.563.869/0001-84.** Avenida Natalino João Brescansin, 2572, no OpenCNPJ e no diretório municipal. O botão comercial do diretório aponta para atendimento via WhatsApp. Catálogo público com preços pendente. [Diretório municipal](https://turismo.sorriso.mt.gov.br/produtos-servicos/flores-e-floricultura/marina-flores-e-presentes)
- **Floricultura na Web.** A vitrine regional anuncia preços e entrega em Sorriso por floristas parceiros, mas a página não identifica o estabelecimento executor. Classificação como oferta regional, com filial de Sorriso pendente. [Vitrine regional](https://floriculturanaweb.com.br/mt/floricultura-em-sorriso)

### Supermercados e farmácias de rede

Os quatro fluxos VipCommerce já configurados mantêm CNPJ e distribuição por unidade em [vipcommerce.json](../config/vipcommerce.json). A rota de identificação é `GET /api-admin/v1/organizacoes/filiais/dominio/{dominio}`, seguida pela sessão anônima da loja. A retirada é lida em `GET /api-admin/v1/org/{org}/filial/{filial}/loja/centros_distribuicoes/retiradas`. Depois de validar município, CNPJ e distribuição, os produtos vêm de `GET /api-admin/v1/org/{org}/filial/{filial}/centro_distribuicao/{distribuicao}/loja/classificacoes_mercadologicas/departamentos/{departamento}/produtos?page={pagina}&limit={limite}`.

- **Sorrinorte Industrial:** organização 365, vitrine 412, filial 1, distribuição 1, CNPJ 02.756.536/0001-67.
- **Sorriso Supermercados Centro:** organização 324, vitrine 369, filial 1, distribuição 1, CNPJ 14.953.277/0001-43.
- **DelMoro Belo Horizonte:** organização 287, vitrine 328, filial 1, distribuição 1, CNPJ 00.877.761/0004-79. As lojas Brescansin e Rota do Sol têm cadastro físico, com vínculo de preços ainda pendente.
- **Casa Aurora Centro:** organização 289, vitrine 330, filial 1, CNPJ 26.777.276/0005-06. O código de distribuição é resolvido em cada rodada pelo endereço e CNPJ da retirada.
- **DelNorte Sorriso:** CNPJ 00.877.761/0013-60, número 8157. O GET do localizador retornou 200. O vínculo entre preço de tabloide, vigência e esta filial segue pendente. [Localizador da rede](https://delnorteatacarejo.com.br/nossas-lojas)
- **Pague Menos Brescansin:** CNPJ 06.626.253/0980-20, número 1314, CEP 78890-134. **Pague Menos Tancredo:** CNPJ 06.626.253/1406-70, número 2198, CEP 78890-143. Para cada loja, o caminho a validar é informar o CEP no site oficial e verificar o identificador que chega ao preço ou ao estoque na retirada. [Catálogo da rede](https://www.paguemenos.com.br/)
- **Drogaria São Paulo Brescansin:** CNPJ 61.412.110/1204-87, número 1722, CEP 78890-179. O [LogMed](https://logmed.org.br/onde-descartar/pdf) também lista esse CNPJ local. A atribuição do catálogo à unidade permanece pendente. [Catálogo da rede](https://www.drogariasaopaulo.com.br/)
- **Drogasil Blumenau:** CNPJ 61.585.865/2284-11, número 2995, CEP 78890-168. **Drogasil Brescansin:** CNPJ 61.585.865/2873-45, número 488, CEP 78896-072. **Drogasil Tancredo:** CNPJ 61.585.865/4134-05, número 2942, CEP 78890-143. O CEP e a seleção de retirada precisam ser testados separadamente para confirmar a loja responsável pelo preço e pelo estoque. [Catálogo da rede](https://www.drogasil.com.br/)

As evidências anteriores de Machado, Atacadão, aiqfome, Ola, Egao e D'Macena continuam na [auditoria dos catálogos públicos](PUBLIC_CATALOGUES.md). Os caminhos novos desta seção documentam a fonte e a validação disponível. O coletor ainda não publica as novas vitrines de varejo.

### Rodada de limpeza, artigos infantis, agro, pet e cosméticos

- **MM Química Produtos de Limpeza.** O rodapé do site publica Avenida Luiz Amadeu Lodi, 361, Sorriso-MT. Nome fantasia e endereço coincidem com o CNPJ ativo 15.191.132/0001-15 no OpenCNPJ. Há outros CNPJs ativos no mesmo endereço e o site consultado não explicita o CNPJ da venda, então o vínculo fiscal da vitrine permanece pendente. `GET /produtos`, `GET /produtos?cat=alcool` e `GET /produtos/agua-sanitaria-5-litros` responderam HTTP 200 sem sessão. A página de produto mostrou SKU 2, R$ 18,00 e estoque de 1.000 unidades na leitura direta. O valor de estoque é uma observação pontual. A listagem inicial mostrou 24 links e 55 páginas. A interface usa uma consulta client-side para avançar e a primeira página HTML por categoria é reproduzível sem chave. A paginação completa sem chave ainda está pendente. [Catálogo](https://mmprodutosdelimpeza.com.br/produtos) · [Produto](https://mmprodutosdelimpeza.com.br/produtos/agua-sanitaria-5-litros)
- **Baby Chic · 19.391.084/0001-60.** O CNPJ e a Avenida Curitiba, 2645, sala 02, CEP 78896-003, coincidem no rodapé e no OpenCNPJ. O domínio indexado `bazarbaby.com.br` falhou na resolução DNS direta. O domínio `babychicsorriso.com.br` retornou HTTP 200 no catálogo e nos produtos. Os links de paginação usam `GET /catalogo?page={n}`, e os itens, `GET /p/{sku}`. As páginas 1 e 2 retornaram 12 SKUs distintos cada, sem sobreposição. Em `/p/01226`, a página e o estado público de variantes mostraram R$ 36,54 para a variante 2360, rosa, 80 × 100 cm, com quatro unidades. O preço e o estoque devem ser lidos por variante. [Catálogo ativo](https://babychicsorriso.com.br/catalogo) · [Produto](https://babychicsorriso.com.br/p/01226)
- **Terra Agri Shop · 54.459.329/0001-44.** CNPJ ativo e endereço Rua Lions Clube, 110, Sorriso-MT, coincidem entre o rodapé e o OpenCNPJ. O CNAE principal é 4661300 e o registro aparece na triagem por atividade secundária. O mapa HTML público `GET /catalog/seo_sitemap/category/` retornou HTTP 200. O caminho `/sitemap.xml` retornou HTTP 404. O GET do produto publicou JSON-LD com SKU 14344, R$ 682,07 e `InStock`. A evidência vale para a loja virtual identificada pelo rodapé. [Mapa de categorias](https://www.terraagrishop.com.br/catalog/seo_sitemap/category/) · [Produto](https://www.terraagrishop.com.br/regulador-de-pressao)
- **Zebu Pet Center Sorriso · 01.034.894/0002-84.** A Avenida Tancredo Neves, 3718, coincide entre o localizador da rede e o cadastro ativo. O GET da página pública do produto K Otrine retornou HTTP 200 e mostrou R$ 15,00 e estoque. O rodapé da loja virtual, porém, indica o CNPJ 01.034.894/0001-01, matriz em Sinop. O preço e o estoque da filial de Sorriso seguem pendentes de um seletor de unidade confirmado. [Produto e localizador](https://www.zebupetcenter.com.br/review/product/list/id/1149/)
- **Inovar Cosméticos Sorriso · 17.226.232/0051-24.** A página oficial da unidade retornou HTTP 200. O OpenCNPJ identifica a mesma marca, Avenida Natalino João Brescansin, 2473, e CEP 78890-135. A página descreve retirada de pedidos do televendas ou WhatsApp. Preço e estoque públicos por filial seguem pendentes. [Página da unidade](https://inovarcosmeticos.com.br/lojas/sorriso)
- **Jr_Personalizados no Elo7.** A página indexada atribui ao vendedor a cidade de Sorriso-MT e anuncia um arquivo digital com preço por forma de pagamento. O mesmo URL retornou HTTP 404 na leitura direta. Vendedor fiscal e disponibilidade atual seguem pendentes. [Página examinada](https://www.elo7.com.br/arquivo-de-topo-corinthians/dp/21612F0)

### Redes de móveis, eletrodomésticos e perfumaria

**Martinello.** O JavaScript `fbits.lojas.js` do [localizador](https://www.martinello.com.br/lojas/busca) usa um POST público de leitura. Ambas as chamadas abaixo devolveram JSON sem sessão. O servidor retornou lojas de outras cidades mesmo com `cidade=Sorriso`, então a resposta precisa ser filtrada por cidade e UF antes de usar os IDs.

```sh
for retirada in false true; do
  curl -fsS -H 'X-Requested-With: XMLHttpRequest' \
    -H 'Referer: https://www.martinello.com.br/lojas/busca' \
    --data "somenteRetiradaNaLoja=$retirada&cidade=Sorriso&estado=MT" \
    'https://www.martinello.com.br/Lojas/GetLojas' |
    jq '.Lojas[] | select(.Estado.Sigla == "MT" and (.Cidade == "Sorriso" or .Cidade == "Boa Esperança (Sorriso)")) | {LojaId, Nome, Cidade, Logradouro, Numero, Cep, RetirarNaLoja}'
done
```

- **F18, LojaId 5.** Avenida Brescansin, 975, CEP 78890-000. Logradouro, número e CEP coincidem com 32.951.535/0019-63 no OpenCNPJ.
- **F50, LojaId 54.** Avenida Natalino João Brescansin, 84. Logradouro e número coincidem com 32.951.535/0051-01. O CEP 78896-072 da loja diverge do 78890-000 cadastral.
- **F64, LojaId 66.** Rua Panambi, 350. Logradouro e número coincidem com 32.951.535/0065-07. O CEP 78890-000 da loja diverge do 78898-095 cadastral.
- **F15, LojaId 22.** Avenida dos Flamboyants, 1303, CEP 78899-200. O localizador classifica a unidade como Boa Esperança (Sorriso). A correspondência fiscal permanece pendente.
- Os CNPJs **32.951.535/0112-50** (Avenida Blumenau Sul, 695) e **32.951.535/0113-30** (Avenida Tancredo Neves, 1999) estão ativos no snapshot, mas não apareceram entre as unidades locais nas respostas testadas. Seus IDs de vitrine continuam pendentes.

Os quatro IDs públicos retornaram `RetirarNaLoja: true`. O campo indica uma modalidade da unidade e não contém SKU, preço ou disponibilidade do produto. A [busca pública de geladeiras](https://www.martinello.com.br/Busca?busca=Geladeiras&pagina=1) devolveu HTML com preços de e-commerce, sem identidade de loja no resultado. O rodapé identifica 32.951.535/0001-34, de Lucas do Rio Verde. A atribuição das ofertas às unidades de Sorriso segue pendente.

- **Magalu Centro · 47.960.950/1208-87.** Avenida Natalino João Brescansin, 30. A ficha oficial `GET /filiais/1137/sorriso` respondeu HTTP 200 e confirma o endereço. O procedimento oficial de retirada consulta o CEP na página do produto. A documentação da rede descreve também envio posterior à loja escolhida, então a opção de retirada isolada não confirma estoque físico ou preço da filial 1137. [Unidade oficial](https://lojas.magazineluiza.com.br/filiais/1137/sorriso) · [Retira Loja](https://conteudo.magazineluiza.com.br/regulamentos/regulamento-retira-loja.html)
- **Havan Sorriso · 79.379.491/0150-24.** Rua Idemar Riedi, 7536, CEP 78895-350, no OpenCNPJ. A rede confirma a presença em Sorriso e a página `/retira-facil` retornou HTTP 200. A rede informa que o Retira Fácil depende do estoque da filial escolhida. Falta observar essa unidade no retorno da consulta de um SKU e confirmar a origem do preço. [Presença em Sorriso](https://ri.havan.com.br/noticia/pequena-helena-aproveita-artigos-natalinos-da-havan-para-decorar-sua-casa-em-itanhanga-mt/) · [Retira Fácil](https://www.havan.com.br/retira-facil)
- **Gazin.** A página inicial retornou HTTP 200 e encaminha ao diretório institucional de filiais. O OpenCNPJ lista 77.941.490/0061-96 (Brescansin, 145), 77.941.490/0258-16 (São Francisco de Assis, 793), 77.941.490/0268-98 (Aripuanã, sem número), 77.941.490/0302-24 (Brescansin, 1499) e 77.941.490/0410-05 (Avenida Brasil, 3013). Cada CNPJ ainda precisa de correspondência com o localizador e com um retorno de preço ou estoque por unidade. [Vitrine](https://www.gazin.com.br/) · [Diretório institucional](https://www.grupogazin.com.br/empresa#filiais)
- **O Boticário.** O OpenCNPJ lista 33.667.510/0010-67 (Brescansin, 2473), 33.667.510/0011-48 (Belo Horizonte, 94), 33.667.510/0012-29 (Roberto Carlos Braga, 51) e 33.667.510/0019-03 (Noêmia Tonello Dalmolin, 2501). A ajuda oficial documenta consultar o CEP na opção de retirada para ver quais lojas têm o produto. A tentativa direta na página de entrada do fluxo retornou HTTP 403. A correspondência das quatro lojas no retorno da consulta de um SKU segue pendente. [Ajuda oficial](https://www.boticario.com.br/atendimento-online/retirar-na-loja)
