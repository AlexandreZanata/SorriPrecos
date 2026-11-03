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

### Localizador da Drogaria São Paulo e farmácias de manipulação

**Drogaria São Paulo Brescansin.** O [localizador oficial](https://www.drogariasaopaulo.com.br/institucional/nossas-lojas) carrega o arquivo público de filiais pelo script `00-rnk-saopaulo-script-institucional-nossas-lojas.min.js`. A leitura direta retornou uma unidade de Sorriso-MT. O registro informa `id=4389`, `codigo=2436`, `contaVtex=drogariaspsorriso`, `indicadorOmnichannel=true` e CEP 78890-179. O texto da rua contém erro de grafia, mas o número 1722 e o CEP coincidem com o CNPJ local 61.412.110/1204-87. O arquivo contém dados de outras cidades e deve ser filtrado antes de aproveitar os IDs.

```sh
curl -fsSL 'https://www.drogariasaopaulo.com.br/api/dataentities/PR/documents/f52e9e7f-a02c-11ea-8337-0a8ac637298d/arquivo/attachments/nossas-lojas.js' |
  jq '.retorno[] | select(.uf == "MT" and .cidade == "SORRISO") | {id, codigo, contaVtex, nome, endereco, cep, indicadorOmnichannel}'
```

O indicador omnichannel descreve a unidade, sem SKU, preço ou quantidade. A rede diferencia explicitamente os preços do site, aplicativo, televendas e lojas físicas. O vínculo entre a unidade 2436 e o retorno de um produto no fluxo de retirada segue pendente. [Ajuda oficial](https://www.drogariasaopaulo.com.br/ajuda)

- **Biomagistral Sorriso / Biomanipulare · 46.455.133/0001-53.** O seletor da rede aponta para uma página específica de Sorriso que retornou HTTP 200. O HTML contém o endereço Avenida Blumenau, 2234, também registrado no OpenCNPJ. A página oferece atendimento e orçamento, sem SKU ou preço em reais na leitura direta. [Seletor da rede](https://www.biomanipula.com.br/) · [Página de Sorriso](https://lp.biomanipula.com.br/sorriso-1-cm-capt)
- **Arte Farma / AuraFarma · 39.538.037/0002-94.** A página retornou HTTP 200 e lista Sorriso e Sinop separadamente. Avenida Tancredo Neves, 1075, coincide com o OpenCNPJ para Sorriso. O site informa CEP 78890-000 e o cadastro informa 78894-097. A página descreve serviços e contato, sem preço público de item na leitura direta. [Página da rede](https://artefarmafarmacia.com.br/)
- **Therapeutica Blumenau · 65.057.774/0001-02.** A ficha `/lojas/7` retornou HTTP 200 e confirma Avenida Blumenau, 2265. O CTA específico da unidade direciona para atendimento. O rodapé mantém o CNPJ da loja virtual da Avenida Curitiba, 01.780.384/0001-75, já integrada. Preço e estoque por SKU da unidade Blumenau seguem pendentes. [Ficha oficial](https://www.therapeuticafarmacia.com.br/lojas/7)
- **DrogaVET · 40.816.299/0001-71.** O OpenCNPJ registra Avenida Santa Maria, 845, Sorriso. A versão indexada do localizador lista a cidade e a leitura direta carregou a aplicação do localizador sem a ficha da unidade no HTML inicial. A confirmação no retorno dinâmico segue pendente. É uma fonte veterinária. [Localizador oficial](https://spotshop.unidades.drogavet.com.br/)
- **Farmacam.** A página regional de entrega em Sorriso respondeu HTTP 200 e aponta para a mesma vitrine `loja=1063102` em suas buscas de produtos. O rodapé do catálogo lista 02.648.081/0001-66 e 24.111.813/0001-17, ambos de São Gonçalo-RJ no OpenCNPJ. A fonte comprova entrega anunciada em Sorriso, com vendedor fora do município. [Página regional](https://entrega.farmacam.com.br/farmacia_de_manipulacao_mato_grosso_sorriso.html) · [Busca vinculada](https://www.farmacam.com.br/loja/busca.php?loja=1063102&palavra_busca=Coenzima+Q10)
- **Pague Menos e Drogasil.** O guia Pague Menos respondeu HTTP 200 e situa a escolha da loja após selecionar os itens e informar o CEP. O guia Drogasil documenta escolha na cesta, mas a leitura direta respondeu HTTP 403. A validação de preço e disponibilidade nas duas filiais Pague Menos e nas três Drogasil permanece pendente. [Clique e Retire Pague Menos](https://www.paguemenos.com.br/clique-e-retire) · [Compre e Retire Drogasil](https://www.drogasil.com.br/compre-retire)

### Cardápios InstaDelivery, DataCaixa e GynFood

O [diretório InstaDelivery de Sorriso](https://portal.instadelivery.com.br/sorriso) retornou HTTP 200. Seu HTML e o estado público `window.__NUXT__` identificam a cidade pelo código IBGE 5107925. Foram examinados os 13 links abaixo. Todos responderam HTTP 200, mas o HTML inicial só carregou a aplicação, sem preço de item. Os valores de pedido mínimo e frete do diretório não são preços de produto. O acesso reproduzível aos itens após a renderização segue pendente.

- [Rudy Delivery e Açaí Prime](https://instadelivery.com.br/rudydeliveryacaiprime), loja 117979. Foz do Iguaçu, 1111. Há mais de um CNPJ ativo nesse endereço e a vitrine consultada não esclarece o vendedor fiscal.
- [Restaurante Bazzã](https://instadelivery.com.br/restaurantedantas), loja 166992. Rua Passo Fundo, 1699. Nome fantasia e endereço coincidem com 46.016.662/0001-50 no OpenCNPJ, apesar do slug legado.
- [Iarla Ravana Confeitaria](https://instadelivery.com.br/iarlaravanaconfeitaria), loja 228903. Endereço de exemplo no diretório.
- [Moodie Cookie](https://instadelivery.com.br/moodiecookie), loja 207116. Avenida Santa Maria, 1511, Jardim Santa Bárbara. CNPJ pendente.
- [Trufados na Garrafa](https://instadelivery.com.br/trufadosnagarrafa), loja 215947. Município indicado, sem logradouro.
- [GS Hamburgueria e Doceria](https://instadelivery.com.br/gshamburgueriaedoceria), loja 205324. Endereço de exemplo no diretório.
- [Açaí da Xefinha](https://instadelivery.com.br/acaidaxefinha). Rua Joinville, 97A, Bela Vista. CNPJ pendente.
- [Gold Açaíteria](https://instadelivery.com.br/goldacaiteriamt), loja 222981. Município indicado, sem logradouro.
- [Univerço Açaí](https://instadelivery.com.br/universoacaiMT). Rua Cristal, 100, Rota do Sol. CNPJ pendente.
- [Barbieris Pizzaria](https://instadelivery.com.br/barbierispizzaria), loja 229562. Município indicado, sem logradouro.
- [Gourmet By Voltanis](https://instadelivery.com.br/gourmetbyvoltanis). Município indicado, sem logradouro.
- [Sabores de Sorriso](https://instadelivery.com.br/saboresdesorriso), loja 192518. Avenida Los Angeles, 660, Jardim Califórnia. CNPJ pendente.
- [Receitas da Vovó](https://instadelivery.com.br/receitasdavovo1), loja 227364. Município indicado, sem logradouro.

A página indexada de [Piratas Burger e Açaiteria](https://instadelivery.com.br/piratasburgereacaiteria), fora dos 13 links acima, apresenta o Copo de Açaí Puro 300 ml a R$ 13,00. O GET inicial também entregou só a aplicação. O preço de R$ 0,01 no item personalizável depende da escolha de complementos e não representa o preço final. Identidade fiscal e leitura pública dinâmica pendentes.

- **O Açaizeiro, DataCaixa.** A página indexada traz Rua Nogueiras, 310, e CNPJ 31.180.602/0001-00. O OpenCNPJ registra situação 08 (baixada) e endereço Perimetral Sudoeste, 2345. A página indexada mostra R$ 22,00 para o açaí de 300 ml e valores zero em partes do HTML. Atribuição fiscal atual e preço final pendentes. [Cardápio](https://www.cardapio.datacaixa.com.br/oacaizeiro)
- **Estaçãoo Burger, DataCaixa.** O CNPJ 32.085.256/0001-35 está ativo em Sorriso. O cardápio indexado traz Rua Lupicines Rodrigues, 1039, enquanto o OpenCNPJ indica Rua Emílio Garrastazu Médici, 1164. A vitrine publica descrições de produtos, mas o endereço atual de operação precisa de confirmação. [Cardápio](https://cardapio.datacaixa.com.br/loja%3D12917)

Os dois GETs DataCaixa retornaram a aplicação sem itens no HTML inicial. O bundle carrega estabelecimentos, grupos e produtos por chamadas que exigem credenciais. A auditoria limitou-se à leitura pública inicial e às páginas indexadas.

O [diretório GynFood de Sorriso](https://www.gynfood.com.br/lista/sorriso-mt) retornou HTML com os nomes dos vendedores e, em alguns cartões, preço e ID de produto. As duas vitrines consultadas retornaram uma página genérica sem cardápio, mesmo com HTTP 200. Assim, o preço da listagem fica documentado como evidência do diretório, com confirmação na vitrine pendente.

- **Vip Açaiteria.** O diretório mostrou o copo de 300 ml a R$ 14,00, produto 533192. O OpenCNPJ tem o CNPJ ativo 23.822.491/0001-51 com o mesmo nome, sem confirmação de endereço na vitrine. [Produto](https://vipacaiteria-mkt6.gynfood.com.br/produtos/visualizar/533192)
- **Império Lanches Delivery.** A página da cidade listou o vendedor e condições de entrega, sem preço de item confirmado. A semelhança com o nome cadastral de 59.573.636/0001-75 não resolve a identidade sem confirmação de endereço. [Vitrine](https://imperiolanches-mkt6.gynfood.com.br/)

#### Revalidação da leitura pública InstaDelivery

Em navegador anônimo, o cardápio do Bazzã chamou `GET https://app.instadelivery.com.br/api/stores/by-slug/restaurantedantas`. A mesma rota funcionou com GET direto, sem login, cookie ou token, nos **14 slugs** examinados acima. Todos retornaram `city=Sorriso`, `state=MT` e `city_id=2402` (identificador interno da plataforma). O ID de cada grupo e item foi conferido contra a loja e o grupo que o contêm. O resultado substitui a pendência de leitura dinâmica registrada na primeira passagem.

```sh
curl -fsS 'https://app.instadelivery.com.br/api/stores/by-slug/restaurantedantas' |
  jq 'select(.city == "Sorriso" and .state == "MT" and .id == 166992)
      | {id, name, city, state, groups: [.groups[] |
          {id, name, itens: [.itens[] | {id, name, price1, price2, stock_control, stock, is_invisible}]}]}'
```

O JSON inclui configurações além do necessário para a auditoria. Preserve apenas os campos de identidade comercial e catálogo pertinentes. O contador abaixo considera grupos e itens visíveis e itens sem exclusão, antes de validar preço, complementos, modalidade de venda e horário. Dos **280 itens**, 27 têm controle de estoque declarado. `stock_control=0` indica estoque desconhecido mesmo quando `stock` é negativo. `is_open` descreve a loja, não a disponibilidade individual. Para o Bazzã, o navegador exibiu Marmita P a R$ 22,00, Marmita M a R$ 28,00 e Marmita G a R$ 35,00, iguais a `price1` nos itens 4926580, 4926660 e 4926673. A loja se apresentava fechada temporariamente na leitura. Variantes, pizzas, adicionais obrigatórios e promoções precisam de validação antes de publicar um preço final.

- **Bazzã · 166992, 14 itens.** Rua Passo Fundo, 1699 e CNPJ 46.016.662/0001-50 coincidem com o OpenCNPJ.
- **Piratas Burger · 24062, 45 itens.** Rua das Palmeiras, 285 B, coincide com o CNPJ ativo 47.499.388/0001-80.
- **Barbieris Pizzaria · 229562, 21 itens.** Rua Guilherme Battaglini, 975, coincide com 64.781.288/0001-61. Validar sabores e tamanhos antes de usar os preços.
- **Gold Açaiteria · 222981, 10 itens.** Travessa da Felicidade, 23, coincide com 66.398.878/0001-35.
- **Trufados na Garrafa · 215947, 13 itens.** A rua Alves de Azevedo coincide com 68.351.432/0001-80. O catálogo omite o número 1875 presente no cadastro.
- **Iarla Ravana · 228903, 8 itens.** O CNPJ 45.714.088/0001-41 está ativo em Sorriso, na Rua Santa Rosa, 14. O endereço de exemplo da vitrine impede confirmar o ponto de operação.
- **Rudy Delivery · 117979, 56 itens.** Rua Foz do Iguaçu, 1111. CNPJ da operação pendente por haver mais de um candidato no endereço.
- **Moodie Cookie · 207116, 17 itens.** Avenida Santa Maria, 1511. CNPJ pendente.
- **GS Hamburgueria e Doceria · 205324, 26 itens.** Endereço de exemplo. CNPJ e ponto de operação pendentes.
- **Açaí da Xefinha · 225271, 16 itens.** Rua Joinville, 97A. CNPJ pendente.
- **Univerço Açaí · 214810, 12 itens.** Rua Cristal, 100. CNPJ pendente.
- **Gourmet By Voltanis · 229662, 11 itens.** Rua Caminho do Sol, 1024. CNPJ pendente.
- **Sabores de Sorriso · 192518, 24 itens.** Avenida Los Angeles, 660. CNPJ pendente.
- **Receitas da Vovó · 227364, 7 itens.** Rua São Bernardo, 318, Serra Dourada. CNPJ pendente.

#### Estoque por item e identidade entre canais

Três cardápios aiqfome de Sorriso responderam HTTP 200. A identidade vem do objeto `const restaurantenovo =` no HTML público. Confira `id`, `slug`, `cnpj`, `logradouro`, `numero`, `cidade_id=4430` e a identidade interna do menu antes de cruzar com o catálogo de outra plataforma.

- **Trufados na Garrafa.** O vendedor aiqfome **143727** confirma Rua Alves de Azevedo, **1875**, CNPJ **68.351.432/0001-80**. O número completa o endereço abreviado da loja InstaDelivery **215947**. É o mesmo estabelecimento, com preços observados por canal. A resposta InstaDelivery para o item **6327343**, açaí com mousse de cupuaçu de 300 ml, informou **R$ 17,99**, estoque controlado de 10 unidades, venda em delivery e nenhum grupo de complementos na amostra da rodada anterior. A nova leitura em navegador anônimo confirmou R$ 17,99 e o botão de pedir para o mesmo item. [Aiqfome](https://aiqfome.com/MT/sorriso/trufados-na-garrafa) · [InstaDelivery](https://instadelivery.com.br/trufadosnagarrafa)
- **Sabores de Sorriso.** O vendedor aiqfome **140961** informa Avenida Los Angeles, 660 e CNPJ **66.307.066/0001-37**, que coincidem com o OpenCNPJ. Nome e endereço completo também coincidem com a loja InstaDelivery **192518**, cuja resposta não comprovou diretamente o CNPJ. Registre a correspondência como corroborada por nome e endereço e mantenha os IDs e preços separados por plataforma. [Aiqfome](https://aiqfome.com/MT/sorriso/sabores-de-sorriso) · [InstaDelivery](https://instadelivery.com.br/saboresdesorriso)
- **Piratas Burger.** A vitrine aiqfome **146974** informa CNPJ **41.450.477/0001-56**, diferente do **47.499.388/0001-80** da InstaDelivery **24062**. Os dois CNPJs constam ativos em Sorriso e os cardápios indicam Rua das Palmeiras, 285 B. A semelhança de marca e endereço não resolve a divergência fiscal. Mantenha os canais separados até confirmar o responsável por cada operação. [Aiqfome](https://aiqfome.com/MT/sorriso/piratas-burger) · [InstaDelivery](https://instadelivery.com.br/piratasburgereacaiteria)

Na amostra de 280 itens InstaDelivery, o filtro conservador reteve **11 candidatos** com estoque controlado positivo, delivery habilitado, preço-base positivo e sem promoção, preço riscado, desconto de item, segundo preço, complemento, indicação de preço inicial, agendamento, janela de horário no item ou grupo marcado como pizza. São quatro de Trufados, três de Gourmet By Voltanis, dois de Iarla Ravana, um de Açaí da Xefinha e um de Sabores de Sorriso. O filtro exclui ofertas que ainda precisam de interpretação comercial e não valida horários da loja ou do grupo. Exemplo de descarte por estoque: o Mix bites (6414611) da Moodie Cookie tinha R$ 14,00 e estoque controlado igual a zero.

O cruzamento comprova que preço e identidade devem ser registrados em níveis diferentes. Um CNPJ local pode ter vários canais com condições próprias. Um mesmo endereço pode exibir CNPJs diferentes e exige revisão antes de consolidar.

### Vitrines de moda, floricultura, decoração e piscina

Os sete sites desta rodada responderam HTTP 200. Nas páginas de produto, compare o preço estruturado com o preço visível e sua forma de pagamento. Em vitrines com recomendações, selecione o objeto de produto cuja URL da oferta corresponde à página solicitada. Para itens com tamanhos ou cores, a indicação de estoque geral ainda precisa ser conferida por variante.

- **Bella Loft · 13.342.691/0001-53.** O rodapé e os termos da loja identificam o CNPJ ativo de Sorriso, registrado na Avenida Brasil, 1680. O GET da página da camisola Laces Prata retornou microdados HTML com SKU **64033**, preço **R$ 859,00**, moeda BRL e `InStock`. A confirmação vale para a loja virtual. A própria loja informa que a disponibilidade pode variar entre canais. [Termos](https://www.bellaloft.com.br/pagina/termos-de-uso.html) · [Produto](https://www.bellaloft.com.br/camisola-curta-alca-laces-prata)
- **Lojas Josie · 53.961.238/0001-40.** CNPJ e Avenida Natalino João Brescansin, 2441, coincidem com o OpenCNPJ. O catálogo Nuvemshop usa `GET /produtos/` e páginas públicas por slug. O JSON-LD do vestido azul publicou SKU **160629**, **R$ 399,90** e duas unidades na vitrine. A rede também opera em Sinop e a amostra não separa estoque físico por loja ou tamanho. [Vitrine](https://www.lojasjosie.com.br/) · [Produto](https://www.lojasjosie.com.br/produtos/vestido-midi-lastex-busto-mv26-ve125-azul-45921/)
- **Mariná Flores e Presentes · 17.710.099/0001-18.** A vitrine localizada nesta rodada publica este CNPJ, ativo no OpenCNPJ como Marisa Baggio Decor. O CNPJ 16.563.869/0001-84, registrado na primeira passagem, também está ativo na Avenida Natalino João Brescansin, 2572. Para atribuir a loja virtual, use o CNPJ exibido no próprio site. O GET de `/botao-sem-embalagem` publicou disponibilidade e **R$ 15,20** em JSON-LD. O HTML identifica esse valor como pagamento à vista no Pix e mostra **R$ 16,00** como preço regular. A página pede a escolha da data de entrega. [Loja virtual](https://marinafloresepresentes.com.br/) · [Produto](https://marinafloresepresentes.com.br/botao-sem-embalagem)
- **Aguapé Piscinas · 07.479.006/0001-32.** A página de contato confirma o CNPJ e a Avenida Blumenau, 2070, que coincidem com o OpenCNPJ. A vitrine entrega links por categoria. A página pública do clorador flutuante Genco publicou SKU **129**, **R$ 84,00** e `InStock` em JSON-LD. A listagem distingue o preço com desconto à vista. [Contato](https://www.aguapepiscinas.com.br/contato) · [Produto](https://www.aguapepiscinas.com.br/acessorios-para-piscina/limpeza-quimica/clorador-flutuante-genco-modelo-cisne)
- **Portal Íntimo · 62.929.105/0001-87.** O rodapé identifica o CNPJ ativo de Sorriso. A listagem Nuvemshop abre produtos por slug em `/produtos/`. Na amostra de perfumaria, o JSON-LD da página informou **R$ 99,69** e cinco unidades para a loja virtual. A amostra não traz SKU estruturado e o endereço de retirada continua pendente. [Vitrine](https://portalintimo.com.br/) · [Produto](https://portalintimo.com.br/produtos/body-splash-cheiro-dela-com-aroma-sedutor-1qdi4/)
- **Revoada.** A vitrine e a página de contato identificam Avenida Tancredo Neves, 1209, Sorriso. A página pública de uma camiseta apresentou **R$ 150,00** e `InStock` no JSON-LD. CNPJ do vendedor e estoque por variante continuam pendentes. [Contato](https://revoada10.lojavirtualnuvem.com.br/contato/) · [Produto](https://revoada10.lojavirtualnuvem.com.br/produtos/camiseta-importada-nhr9z/)
- **Aluminium Móveis · 29.749.866/0001-81.** Nome fantasia e Avenida Curitiba, 3180, coincidem com o OpenCNPJ. O site mostra linhas de móveis e encaminha a orçamento. A página inicial consultada não publicou preço por item. [Site](https://moveisaluminium.com.br/)

### Informática, refrigeração, peças e bicicletas

- **Casa do Computador · 21.588.737/0001-47.** Nome e Avenida Tancredo Neves, 1269, CEP 78894-097, coincidem com o OpenCNPJ. O GET da página inicial retornou HTTP 200 e descreve os produtos, com venda sob orçamento. Sem preço público no HTML consultado. [Site](https://www.casacomputador.com.br/)
- **Premium Computadores · 11.098.692/0001-79.** Rua dos Pioneiros, 221, coincide com o OpenCNPJ, onde o nome fantasia aparece como Premuim Computadores. A página indexada identifica o endereço local. O GET atual respondeu HTTP 200 com uma aplicação Angular sem itens ou preços no HTML inicial. Catálogo dinâmico pendente. [Site](https://premiumcomputadores.online/)
- **ADS Informática.** O GET retornou HTTP 200 e mostra Rua Genésio Roberto Baggio, 510. O OpenCNPJ contém um candidato ativo nesse endereço (33.355.304/0001-20), mas a página não apresenta CNPJ para confirmar a atribuição. A grade nasce vazia no HTML e o JavaScript tenta preencher os produtos com uma consulta Supabase que usa chave de aplicação. Em navegador anônimo, a grade mostrou “Nenhum produto encontrado”. Como o código exibe a mesma grade em caso de falha da consulta, a mensagem não comprova catálogo vazio. [Site](https://adsinformatica-sorriso.com/)
- **Xing Ling Presentes · 63.692.128/0001-83.** Marca, Avenida Natalino João Brescansin, 261 e CEP 78896-071 coincidem com o OpenCNPJ. O GET respondeu HTTP 200 e mostra categorias e informações da loja física, sem oferta com preço por item no HTML. [Site](https://lojaxingling.com.br/)
- **UltraFrio Sorriso · 26.265.418/0002-03.** A unidade da Perimetral Sudeste, 10511, CEP 78896-052 coincide com o OpenCNPJ. O GET inicial carregou a aplicação React. Seu módulo público inclui a rota `/produtos` e os endereços de Sorriso e Sinop. Os valores em reais identificados no módulo referem-se a uma promoção de cupons, sem preço ou estoque por SKU de Sorriso. [Site](https://ultrafrio.net/)
- **AC Acessórios · 07.351.198/0001-05.** O site respondeu HTTP 200 e identifica Avenida Perimetral Sudeste, 10187. Logradouro e número coincidem com o OpenCNPJ, com divergência de CEP. O cadastro da mesma raiz também tem 07.351.198/0002-88 na Rua São José, 1894, cuja identidade no site segue pendente. O site descreve categorias e atendimento no balcão, sem preços de item no HTML consultado. [Site](https://www.acacessorios.com.br/)
- **Hipersom · 03.476.691/0001-92.** Marca, Rua Genésio Roberto Baggio, 545, e município coincidem com o OpenCNPJ, com divergência de CEP. O GET retornou HTTP 200 e apresenta tipos de acessórios automotivos, sem preços de item no HTML consultado. [Site](https://hipersomsorriso.com.br/)
- **Extra Bike e Ninu’s Bike.** O localizador oficial OGGI ainda aponta Extra Bike na Avenida Blumenau, 2895. No OpenCNPJ, **22.301.018/0001-66** (Extra Bike) consta baixado e **51.364.603/0001-03** (Ninu’s Bike) consta ativo no mesmo endereço. A ficha da revendedora não demonstra vínculo de sucessão ou catálogo atual da Ninu’s Bike. Preço e estoque locais seguem pendentes. [Ficha OGGI](https://oggibikes.com.br/placemarks/extra-bike/)
- **Loja das Abraçadeiras · 07.797.567/0001-80.** O nome aparece em documento de compra de Sorriso, e a vitrine retornou HTTP 200 com preços. O CNPJ exibido no rodapé consta no município do Rio de Janeiro-RJ no OpenCNPJ. Classificada como fornecedor externo. [Vitrine](https://www.abracadeiras.store/) · [Documento municipal](https://site.sorriso.mt.gov.br/dl/436817)

### Combustíveis e gás de cozinha — preços observados pela ANP

O GET da [planilha semanal por revendedor](https://www.gov.br/anp/pt-br/assuntos/precos-e-defesa-da-concorrencia/precos/arquivos-lpc/2026/revendas_lpc_2026_10_25-2026_10_31.xlsx), publicada em 03/11/2026, funcionou sem conta. Na aba `POSTOS REVENDEDORES`, filtre as colunas I e J por `SORRISO` e `MATO GROSSO`. Leia produto, unidade, valor e data nas colunas L a O. Complete os CNPJs numéricos da coluna A com zeros à esquerda até 14 dígitos.

A seleção contém **41 observações de 27/10/2026** para **17 CNPJs**. Todos estavam ativos em Sorriso no snapshot 2026-10 do OpenCNPJ, com endereço correspondente após normalizar abreviações e pontuação. Favretto apresenta a grafia Moraes na ANP e Morais no cadastro. Há divergências de CEP entre as fontes. A amostra abrange oito postos e nove revendas de GLP. Os valores são de uma pesquisa de campo datada e ficam classificados como referência histórica. A planilha não informa disponibilidade ou estoque atual. [Publicação semanal](https://www.gov.br/anp/pt-br/assuntos/precos-e-defesa-da-concorrencia/precos/levantamento-de-precos-de-combustiveis-ultimas-semanas-pesquisadas)

Postos da amostra, com gasolina comum em reais por litro:

- Praça das Fontes · 08.609.198/0001-17 · Brescansin, 2440 · R$ 6,92.
- Dallas · 06.047.857/0001-43 · Idemar Riedi, 9164 · R$ 6,39.
- Smile · 02.412.522/0001-26 · Tancredo Neves, 2219 · R$ 6,59.
- Paraíso · 04.208.206/0001-62 · Brescansin, 1375 · R$ 6,79.
- Cidade · 02.583.241/0001-36 · Brescansin, 1691 · R$ 6,89.
- Redentor · 00.345.307/0001-24 · Perimetral Sudeste, 8245 · R$ 6,89.
- Nossa Senhora do Sorriso · 09.060.760/0001-69 · Tancredo Neves, 2943 · R$ 6,59.
- Motter · 09.074.874/0001-68 · Blumenau, 1140 · R$ 6,68.

Revendas da amostra, com GLP em reais por botijão de 13 kg:

- Pirapó · 01.872.282/0001-80 · Brescansin, 2102 · R$ 145,00.
- Lorenz e Martini · 04.419.333/0001-00 · Perimetral Sudoeste, 2199 · R$ 140,00.
- Peters Gás Porto Alegre · 07.693.574/0001-31 · Porto Alegre, 2164 · R$ 144,00.
- V. J. Barbieri · 05.526.729/0001-10 · Carazinho, 415 · R$ 145,00.
- Favretto e Favretto · 12.600.767/0001-30 · Vinicius de Moraes, 818 · R$ 135,00.
- Gaz Centro Oeste · 01.977.586/0001-01 · La Paz, 169 · R$ 145,00.
- Disk Água Gás e Carvão Nilson · 17.992.813/0001-08 · Las Vegas, 959 · R$ 135,00.
- Peters Gás Horácio Lafer · 07.693.574/0003-01 · Horácio Lafer, 1033 · R$ 144,00.
- Chico do Gás · 52.779.175/0001-42 · Iguaçu, 230A · R$ 135,00.

A tabela municipal de 02/04/2025 também separa retirada e entrega de GLP. Para Chico do Gás, ela identifica **34.308.629/0001-15**, Rua Iguaçu, 230, enquanto a amostra ANP identifica **52.779.175/0001-42**, número 230A. A semelhança de nome e endereço não autoriza unir os dois CNPJs. [Tabela do Procon Sorriso](https://site.sorriso.mt.gov.br/noticia/procon-disponibiliza-tabela-de-precos-do-gas-de-cozinha-67eeb3e38916c)

### Cardápios online de gás em Sorriso

Seis páginas aiqfome responderam HTTP 200. Em cada GET, o objeto público `const restaurantenovo =` identifica o vendedor e traz `menu.cardapios_categorias[].itens[].itens_tamanhos[]`. A leitura confirmou `cidade_id=4430` e a correspondência do ID do menu com o vendedor. Confira também `restaurante_id` e `item_id` em cada variante, bem como disponibilidade da categoria, item e variante. Os preços abaixo são valores de cardápio antes de eventual taxa de entrega. A consulta não validou a tarifa final ou um preço específico de retirada. Separe recarga, botijão com vasilhame e combos.

- **Divino Gás · 52637 · 18.807.655/0001-31.** Avenida Mário Raiter, 256, coincide com o OpenCNPJ. O CEP genérico do cadastro diverge do da vitrine. O gás de 13 kg (variante 14278097) aparecia disponível a **R$ 158,00**, e o gás completo de 13 kg a R$ 428,00. O site Multigás confirma o mesmo endereço e descreve atendimento para botijões de 5, 8 e 13 kg, sem preço próprio publicado. A coincidência de endereço é evidência de correspondência, sem CNPJ publicado no site Multigás. [Cardápio](https://aiqfome.com/MT/sorriso/divino-gas) · [Site Multigás](https://multigassorriso.com.br/)
- **Peters Gás · 29328 · 07.693.574/0001-31.** Avenida Porto Alegre, 2164, coincide com OpenCNPJ e ANP. O gás P13 (variante 6578586) aparecia disponível a **R$ 160,00**. A unidade da Rua Horácio Lafer tem outro CNPJ e não foi vinculada a esse preço. [Cardápio](https://aiqfome.com/MT/sorriso/peters-gas)
- **Pirapó Gás · 30618 · 01.872.282/0001-80.** Avenida Natalino João Brescansin, 2102, coincide com OpenCNPJ e ANP. A recarga P13 (variante 44111992) trazia R$ 170,00 e preço promocional de R$ 160,00, mas a categoria, o item e a variante estavam indisponíveis. [Cardápio](https://aiqfome.com/MT/sorriso/pirapo-gas)
- **Lírio Gás · 79261 · 13.159.661/0001-06.** A recarga P13 (variante 24640273) aparecia disponível a **R$ 170,00**. O cardápio indica Rua Feliz Natal, 85B, e a pesquisa municipal de 2025 traz número 85. O OpenCNPJ situa o CNPJ na Rua Solimões, 548. A atribuição física atual permanece pendente. [Cardápio](https://aiqfome.com/MT/sorriso/lirio-gas)
- **Rico Gás · 106105 · 14.924.879/0001-72.** A recarga P13 (variante 59103054) aparecia disponível a **R$ 155,00**. A vitrine indica Avenida Porto Alegre, 1580, e o OpenCNPJ indica Avenida Perimetral Sudoeste, 1691. O vínculo físico segue pendente. [Cardápio](https://aiqfome.com/MT/sorriso/rico-gas)
- **Sol Gás · 93545 · 47.252.185/0001-95.** Rua Manoel da Nóbrega, 989, coincide com o OpenCNPJ, com diferença de CEP. A recarga 13 kg anunciada a R$ 150,00 traz `restaurante_id=61380` nas cinco variantes, diferente da loja 93545. A amostra não permite atribuir o preço da recarga à unidade. [Cardápio](https://aiqfome.com/MT/sorriso/sol-gas)
- **Sorrigas.** O diretório Supergasbras confirma Rua Carazinho, 415, o mesmo endereço da revenda V. J. Barbieri (05.526.729/0001-10) na ANP e no OpenCNPJ. A ficha consultada fornece canais de pedido sem preço de item. [Ficha oficial](https://revendas.supergasbras.com.br/revendas/sorrigas-sorriso-mt)

### Produtos naturais, pet e Sorrigas no aiqfome

Os seis GETs públicos abaixo retornaram HTTP 200. A leitura usou `const restaurantenovo =`, conferindo município, CNPJ, vendedor do menu e IDs do vendedor e item em cada variante. O total por vitrine conta variantes com valor positivo e IDs correspondentes, antes de interpretar unidades, complementos ou promoções. Nenhuma dessas variantes estava marcada como disponível nos três níveis do catálogo na consulta. Os preços são valores publicados no cardápio, sem confirmação de oferta ativa.

- **Empório Dom Manoel · 35099 · 30.398.947/0001-63.** Avenida Blumenau, 2694, coincide com o OpenCNPJ. Foram encontradas 452 variantes com preço. Exemplo: lúpulo 100 g, variante 8506860, R$ 7,90. [Cardápio](https://aiqfome.com/MT/sorriso/emporio-dom-manoel)
- **Nutrys Produtos Naturais · 141596 · 66.927.065/0001-95.** Avenida Tancredo Neves, 2302, coincide com o OpenCNPJ. Foram encontradas 545 variantes com preço. Exemplo: mistura para bolo 450 g, variante 60006254, R$ 49,90. O catálogo também mistura suplementos e produtos de higiene, que precisam de classificação separada. [Cardápio](https://aiqfome.com/MT/sorriso/nutrys-produtos-naturais)
- **Puppy PetsMT · 59548 · 40.436.597/0001-36.** A vitrine aponta Rua Caminho do Sol, 1008. O CNPJ ativo de Sorriso está cadastrado na Rua Inivaldo Bedin, 1233. Há 80 variantes com preço, mas o vínculo com o ponto físico segue pendente. [Cardápio](https://aiqfome.com/MT/sorriso/puppy-petsmt)
- **Veterinaria Sorriso São Francisco · 35391 · 35.848.278/0001-06.** Rua São Francisco de Assis, 433, coincide com o OpenCNPJ. Foram encontradas 289 variantes com preço. Exemplo: ração Thor para cão adulto 3 kg, variante 8571288, R$ 76,00. [Cardápio](https://aiqfome.com/MT/sorriso/veterinaria-sorriso)
- **Veterinaria Sorriso Tancredo Neves · 101747 · 35.848.278/0002-89.** Avenida Tancredo Neves, 320, coincide com o OpenCNPJ. O cardápio traz 31 variantes com preço e IDs próprios. Outras 134 variantes indicam o vendedor 35391 da unidade São Francisco e foram excluídas da atribuição à loja Tancredo. As duas filiais devem manter IDs e preços separados. [Cardápio](https://aiqfome.com/MT/sorriso/veterinaria-sorriso-tancredo-neves)
- **Sorrigas · 72785 · 05.526.729/0001-10.** Rua Carazinho, 415, coincide com o OpenCNPJ, a ficha Supergasbras e a ANP. O cardápio retorna oito variantes com preço, incluindo recarga P13 a R$ 170,00 (21574497) e vasilhame completo P13 a R$ 460,00 (21574502), ambos indisponíveis na consulta. [Cardápio](https://aiqfome.com/MT/sorriso/sorrigas)

#### Preço e retirada por SKU na simulação pública VTEX

A rota documentada pela VTEX respondeu HTTP 200 sem token, cookies ou dados pessoais. O POST é uma simulação de leitura. Os resultados abaixo são amostras de uma unidade de produto, sem pedido. [Documentação VTEX](https://developers.vtex.com/docs/guides/setting-up-white-label-seller-as-pickup-point)

```sh
curl -fsS -H 'Content-Type: application/json' \
  --data '{"items":[{"id":"49697","quantity":1,"seller":"1"}],"country":"BRA","postalCode":"78890-179"}' \
  'https://www.drogariasaopaulo.com.br/api/checkout/pub/orderForms/simulation' |
  jq '{items: [.items[] | {id, sellingPrice, availability, sellerChain}],
       retirada: [.logisticsInfo[].slas[] | select(.deliveryChannel=="pickup-in-point") |
         {pickupPointId, price, shippingEstimate}],
       pickupPoints: [.pickupPoints[] | {id, address}]}'
```

- **Drogaria São Paulo Brescansin · 61.412.110/1204-87.** O CEP 78890-179 foi confirmado pelo GET `/api/checkout/pub/postal-code/BRA/78890-179` como Sorriso-MT. O SKU **49697** apareceu como disponível a **R$ 5,59** e `sellerChain=["1","drogariaspsorriso"]`. O único ponto de retirada foi `drogariaspsorriso_2436`, com endereço Avenida Natalino João Brescansin, 1722, Sorriso-MT, tarifa zero e estimativa de 30 minutos. O controle com CEP 01310-100 devolveu vendedor e pontos de retirada de São Paulo, embora o preço do item fosse igual. Use o vendedor e o ponto da resposta para validar a filial. A igualdade de preço entre CEPs não comprova identidade. [Produto](https://www.drogariasaopaulo.com.br/creme-dental-colgate-90g-protecao-anticarie-/p)
- **Pague Menos Brescansin · 06.626.253/0980-20.** O mesmo caminho de simulação em `www.paguemenos.com.br`, com SKU **52572** e CEPs 78890-134 e 78890-143, devolveu **R$ 12,99**, `availability=available` e `sellerChain=["1","paguemenos00980"]` em ambas as consultas. O ponto de retirada da loja **980** informa Avenida Natalino João Brescansin, 1314, Sorriso-MT, e `warehouseId=980`. O CEP devolvido no ponto (78896-016) diverge do cadastral (78890-134).
- **Pague Menos Tancredo · 06.626.253/1406-70.** As mesmas simulações listaram também o ponto da loja **1406**, Avenida Tancredo Neves, 2198, Sorriso-MT, com `warehouseId=1406`, retirada grátis e estimativa de 60 minutos. O CEP devolvido no ponto (78890-000) diverge do cadastral (78890-143). A retirada do SKU foi observada, enquanto o preço retornado continua associado à loja 980 na cadeia de vendedores. O preço próprio da filial 1406 segue pendente. [Produto Pague Menos](https://www.paguemenos.com.br/creme-dental-colgate-natural-exracts-defesa-reforcada-90g/p)

Em cada coleta, valide SKU, disponibilidade e vendedor antes de atribuir o valor. Cruze a opção de retirada em `logisticsInfo[].slas[]` com o endereço em `pickupPoints[]` pelo identificador exato. O `price` da opção de retirada é a tarifa logística e `items[].sellingPrice` é o preço do produto, ambos em centavos. Essas amostras comprovam a oferta online no momento da consulta. O catálogo completo, a quantidade física em estoque e os preços de balcão ficam fora da evidência desta rodada.
