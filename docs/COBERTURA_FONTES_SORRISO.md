# Cobertura dos catálogos de Sorriso

Verificações realizadas em 4 de novembro de 2026.

## Casa Aurora

O [localizador oficial](https://casaaurora.com.br/lojas/sorriso-mt/) confirma a filial da Av. Otávio de Souza Cruz, 441, com CNPJ 26.777.276/0005-06. O [encarte Quarta da Carne](https://casaaurora.com.br/wp-content/uploads/2026/11/quarta-da-carne-casa-aurora.jpg) informa que as ofertas valem nas lojas Casa Aurora em 04/11/2026. A revisão separou os 12 preços individuais por kg do programa de selos e excluiu o anúncio com mais de um corte sem variante identificada. Os registros vencem ao fim do dia em Sorriso. O aplicativo exibe o preço do encarte, com estoque não confirmado e link para o original. Esses preços ficam fora do cálculo do carrinho.

Para repetir o processo:

1. Execute `python3 scripts/collect_casa_aurora_flyer.py --list`. O comando lê as 20 mídias recentes na API pública do WordPress.
2. Confira na arte a validade, o escopo das lojas, as condições, o produto e a unidade de medida. Crie um manifesto em `config/flyers/` no formato do exemplo `casa-aurora-3864.json`. Grave o SHA-256 do arquivo oficial efetivamente revisado. Separe artes de campanhas condicionais e descarte variantes ambíguas.
3. Execute `python3 scripts/collect_casa_aurora_flyer.py --manifest config/flyers/<arquivo>.json --binary /caminho/para/sorriprecos`. O coletor verifica a mídia na API, a origem HTTPS, o hash e a validade local antes de importar. A chave da mídia e do item permite repetir a importação com a mesma identidade.

A rota descoberta cobre encartes oficiais em imagem. A publicação de um catálogo completo da filial continua pendente.

Revalidação de 09/11/2026: o e-commerce `casaaurorasupermercados.com.br` continua identificando a loja de Sinop (CNPJ 26.777.276/0003-36); o `pickups` público só lista a unidade Sinop 503. O tipo de conteúdo `ofertas-aurora` está vazio e a mídia mais recente é o encarte Quarta da Carne de 04/11/2026, já fora da validade. A loja de Sorriso segue apenas com os encartes oficiais revisados, sem catálogo de produto por SKU.

## Drogasil

As [páginas públicas de categoria](https://www.drogasil.com.br/saude/saude-sexual/preservativos.html) expõem SKUs próprios, embalagem, marca e imagem no estado do frontend. A página 2 por `?p=2` retornou deslocamento 48 e SKUs distintos da página 1. O teste também confirmou produtos na [categoria Medicamentos](https://www.drogasil.com.br/medicamentos.html).

A operação pública `GET_STOCK` em `/api/next/product-hub/graphql` retornou estoque para as filiais 3034, 3852 e 5635, cruzadas com endereço e município. O teste conjunto com 48 SKUs devolveu “Filial próxima sem estoque”, embora SKUs do mesmo conjunto tivessem saldo em Sorriso. Por isso o coletor consulta cada SKU separadamente. As observações usam o CEP 78890-168 e o identificador de filial devolvido pela fonte. O [Compre e Retire](https://www.drogasil.com.br/compre-retire) descreve a compra pelo site com retirada. A rota de estoque não fornece valor por filial. Os resultados ficam marcados como estoque de retirada sem preço.

O processo `collect-pharmacy-vtex --watch` inclui a Drogasil. A cada ciclo, a Drogasil lê uma página de uma das dez categorias configuradas e faz até oito consultas de estoque, com intervalo mínimo de 1,5 segundo entre elas. O SQLite retém a página e a posição de cada categoria. O próximo ciclo seleciona a categoria há mais tempo sem tentativa. A posição avança junto com a gravação dos dados de cada item. HTTP 429 ou 503 interrompe a rodada e adia nova consulta ao domínio por pelo menos dez minutos, respeitando `Retry-After` quando maior. Kits, itens de terceiros e embalagens ambíguas ficam fora da normalização. As confirmações de estoque expiram em 30 minutos. A varredura percorre o catálogo progressivamente e a disponibilidade exibida corresponde aos SKUs consultados nessa janela. O primeiro ciclo em produção local verificou oito preservativos e gravou 24 observações nas três filiais.

## Atacadão e Therapeutica

O estado público da [unidade Atacadão Sorriso](https://www.atacadao.com.br/loja/sorriso) verificado anteriormente identificou `storeId=899`, endereço Rua Idemar Riedi 8150 e `ecommerce=null`. A filial publica encartes. O SQLite local conservava 90 preços vigentes do encarte com validade até 09/11/2026, marcados com estoque não confirmado. O catálogo geral do site precisa demonstrar vínculo local antes de ser usado como fonte da unidade.

O [índice BR Catálogos da unidade 899](https://brcatalogos.com.br/atacadao/loja-899) também lista o Ofertão e o [folheto Boa do Dia](https://brcatalogos.com.br/atacadao/boa-do-dia-mt). A cópia do folheto diário foi conferida visualmente em 04/11/2026. A arte do Atacadão traz a data e lista Sorriso, Rua Idemar Riedi 8150, no rodapé. Foram validados quatro preços com estoque não confirmado. O índice é independente e fornece imagens, sem o PDF estruturado usado na coleta oficial. O modo abaixo lê a imagem pública referenciada pelo índice, verifica o hash do arquivo revisado e expira os preços no fim do dia em Sorriso. Para cada nova arte, revise as condições, a data, os produtos e o endereço antes de registrar outro manifesto.

```sh
python3 scripts/collect_atacadao_flyers.py --reviewed-copy config/flyers/atacadao-boa-do-dia-2026-11-04.json --binary /caminho/para/sorriprecos
```

A [Therapeutica matriz Sorriso](https://www.therapeuticafarmacia.com.br/lojas/8) confirma a Av. Curitiba, 2734 e CNPJ 01.780.384/0001-75. O coletor usa o sitemap e os dados públicos do produto, com checagem de endereço e CNPJ no site. A rede local apresentou certificados emitidos por uma CA Fortinet que a cadeia padrão deste host não pôde validar para os dois domínios.

Para retomar, execute os coletores em uma rede autorizada onde os certificados HTTPS sejam verificados normalmente. Em uma instalação com inspeção TLS aprovada, o administrador pode fornecer um PEM por coletor via `SORRIPRECOS_ATACADAO_CA_FILE` ou `SORRIPRECOS_THERAPEUTICA_CA_FILE`. O certificado deve vir do administrador da rede por canal autenticado. As variáveis são opcionais, adicionam confiança apenas no respectivo coletor e mantêm a validação de domínio. Verifique o HTTPS e o resultado de uma coleta antes de iniciar o modo contínuo.

```sh
python3 scripts/collect_atacadao_flyers.py --binary /caminho/para/sorriprecos
/caminho/para/sorriprecos collect-pharmacy --limit 12
```

## Machado Tancredo Neves

A consulta anônima de CEP foi repetida para 78893-110. O site recusou a cobertura e a primeira página do Clube Mais devolveu 50 preços associados à filial 3, CNPJ 03.790.896/0003-00, em Sinop. A loja de Sorriso usa o CNPJ 03.790.896/0011-10. O [portal de encartes vinculado pelo site institucional](https://jcm.net.br/ms/ofertas) só oferece Colíder no seletor de supermercados. O seletor de atacados oferece cinco outras cidades de MT e também não lista Sorriso. As verificações estão detalhadas em [Machado de Sorriso](MACHADO_SORRISO_PRICES.md).

Para avançar com preços atuais, falta uma fonte pública que vincule os produtos ao CNPJ de Sorriso, como um encarte vigente da unidade ou uma vitrine pública com identificação de filial. O Procon continua separado no histórico.

Revalidação de 09/11/2026: o app oficial `br.com.redemachado` (Agile E-commerce) foi instalado e aberto no aparelho físico. Ele carrega o mesmo catálogo de `www.redemachado.com.br`, sem seletor de loja, e mantém a filial 3 de Sinop; o app `Clube + Machado` é o programa de relacionamento de Sinop. O CEP 78893-110 continua recusado. Nenhum preço foi atribuído a Sorriso.
