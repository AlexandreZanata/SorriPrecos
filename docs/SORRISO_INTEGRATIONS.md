# Separação dos dados de Sorriso-MT, mercado por mercado

Atualização: 03/11/2026. Escopo único: **Sorriso, MT, IBGE 5107925**, confirmado no [IBGE](https://www.ibge.gov.br/cidades-e-estados/mt/sorriso.html). Não se usa o nome “Sorriso” isoladamente como filtro: ele também é marca de produto e nome de supermercados de outros municípios.

O código e as correções foram enviados a [AlexandreZanata/SorriPrecos](https://github.com/AlexandreZanata/SorriPrecos). A primeira execução de CI passou no backend e falhou no setup Android por pacote obsoleto `tools`; a correção para pacotes atuais passou nos dois jobs.

## O que está realmente consumido no localhost

**27 produtos e 31 observações históricas reais, em sete redes e oito filiais de Sorriso**, extraídos do [relatório público Procon-MT / Páscoa 2026](https://www.procon.mt.gov.br/documents/d/procon/tabela-produtos-analisados-pascoa-final). Observações são de março de 2026. **Zero preços atuais confirmados; zero feeds comerciais varejistas autorizados ativos.** Importação de histórico público não significa integração automática com o supermercado.

O coletor aceita exclusivamente a coluna Município = Sorriso, endereço com Sorriso-MT, nome de origem mapeado e logradouro/número correspondente. O mapeamento está em `config/procon_sorriso_branches.json`. A presença de uma filial da rede na cidade não é suficiente para atribuir a ela preços nacionais ou de outra unidade.

Foram encontradas 75 linhas de Sorriso na extração por células. Aceitaram-se 31 observações finais: 42 linhas não têm estabelecimento mapeado, marca confirmada ou embalagem utilizável; duas observações repetidas/anteriores foram consolidadas por SKU/filial. Americanas, Havan, postos, lojas de chocolates e outros negócios fora do escopo não foram incorporados. Linhas de outros municípios nunca são exportadas/importadas. GTINs não aparecem no relatório; a normalização preserva o título exato e a embalagem para evitar agrupamentos por semelhança.

O snapshot verificável está em `datasets/procon-2026-sorriso/`. Cada registro preserva a data da nota, a pesquisa, URL/página, embalagem e filial. Estoque atual permanece desconhecido (`available: null`). Os preços históricos não entram no mínimo recente e não recebem destaque de oferta atual. O PDF original consultado tem SHA-256 `db9d50a347f161bd3b9b7ce0569137410bdb895c944a8d0c38b03271bb80f49c`.

## Análise individual

1. **DelMoro.** O institucional aponta para [delmorosupermercados.com.br](https://www.delmorosupermercados.com.br/). A vitrine consultada não exibiu produtos, preços ou seletor de filial. Separação histórica validada: loja 04/Belo Horizonte 94 e loja 18/Blumenau Sul 754, com nomes/endereço do relatório. Sete observações importadas. Os IDs do comércio eletrônico dessas filiais não foram comprovados; não se usa um catálogo nacional como se fosse de Sorriso.
2. **Sorriso Supermercados.** Identidade local: Brescansin 220, CNPJ 14.953.277/0001-43. Também foi localizado o [app `br.com.sorrisosuper.omnichannel`](https://play.google.com/store/apps/details?id=br.com.sorrisosuper.omnichannel), distinto do app homônimo de Uberlândia. Quatro observações históricas importadas por nome e endereço. Domínios citados e app não forneceram um feed atual local acessível nesta execução.
3. **Sorrinorte.** [Vitrine](https://www.sorrinorte.com.br/) e app VipCommerce confirmados, mas a página mostrou estrutura genérica sem catálogo. `robots.txt` restringe busca/filtros e diversas rotas legadas; essas rotas não foram automatizadas. A análise de módulos estáticos públicos confirmou que o frontend trabalha com filial selecionada, mas não forneceu um identificador validado da unidade de Sorriso nem um feed documentado. Cadastro recente aponta Palmares 489 e fontes anteriores Tangará 549: resolver a divergência antes de habilitar preços. Nenhum preço importado.
4. **Machado.** A unidade denominada Machado Tancredo Neves no relatório está em Noêmia Tonello Dalmolin 2501. As duas denominações do documento são mapeadas à mesma filial somente porque endereço e número coincidem. Sete observações históricas importadas. O domínio `redemachado.com.br` apresentou bloqueio de acesso; resultados do buscador sem filial não foram usados como oferta atual. Próximo contrato precisa fornecer o código de loja local, não somente a marca da rede.
5. **Casa Aurora.** [Unidade oficial](https://casaaurora.com.br/lojas/) em Otávio de Souza Cruz 441; o institucional vincula [casaaurorasupermercados.com.br](https://www.casaaurorasupermercados.com.br/). A vitrine consultada não mostrou itens/filial. A regra de futura coleta é exigir o identificador da unidade 441 e confirmação da cidade; não confundir redes homônimas. Sem preços importados.
6. **DelNorte.** Página oficial reúne Sorriso e Nova Mutum. O buscador mostrou uma lista de ofertas que **não apareceu na página atual do navegador**; os valores indexados não foram atribuídos a Sorriso ou apresentados como vigentes. O acesso automatizado direto recebeu 403 e não foi contornado. O relatório histórico identifica loja 13, Perimetral Sudeste 8157; oito observações importadas nessa filial. Coleta atual depende de origem que identifique explicitamente Sorriso e validade.
7. **Babi.** Sorriso: Tancredo Neves 2773/Girassol Supermercado. O CNPJ e a filial de União do Sul são distintos e ficam excluídos. A presença municipal/social não entregou catálogo atual estruturado. Nenhum preço importado; futura integração deve vincular CNPJ/filial da unidade de Sorriso.
8. **Novos Campos.** Existem referências a Alencar Bortolanza 949, Blumenau Sul 1185 e São Francisco de Assis 1675. Cadastros mais recentes também associam o número 949 à bandeira ZL/empresa do Filezão, enquanto referências antigas indicam Novos Campos. Não se somam preços dessas identidades sem conferir CNPJ e período. Sem catálogo/preços atuais importados.
9. **Real.** Unidade Tancredo Neves 104, Irmãos Maldaner, CNPJ 01.020.470/0001-80. Perfil comercial consultado não publica produtos. Não se usa uma rede homônima de outro estado. Sem preço importado; referência futura deve corresponder ao endereço 104.
10. **Mercado Amazonas.** Foi encontrada a [página específica aiqfome em MT/Sorriso](https://aiqfome.com/MT/sorriso/mercado-amazonas), que confirma Blumenau 1011. A página atual não apresentou itens/preços. O mínimo de pedido de R$ 50,00 e valores de exemplo de checkout **não** são preços de produtos e foram descartados. Futura integração deve fixar esse vendedor/endereço, observando os termos da plataforma.
11. **Atacadão.** A página `/loja/sorriso` identifica Idemar Riedi 8150. Contexto de CEP de outra cidade apareceu nos resultados de pesquisa; não se consumiu esse catálogo como local. O navegador apresentou `ERR_CERT_AUTHORITY_INVALID`, sem desvio do aviso. Uma observação histórica foi importada pela unidade exata. Para catálogo atual, exige-se seleção de loja e evidência do seu identificador.
12. **Londrina.** Referência local Londrina 477/Jardim Carolina. Pesquisa individual não confirmou catálogo com preços. Importação futura somente por arquivo/contrato com identificação da unidade; sem preços importados.
13. **Vipi.** O [histórico do site homônimo](https://www.supervipi.com.br/institucional) menciona operação antiga em Sorriso, mas a relação atual apresentada é de lojas de SC/PR. Não se usaram esses preços. Referência local Rui Barbosa 351 precisa de confirmação cadastral e código de filial atual; sem preços importados.
14. **Sol.** São Francisco de Assis 2073, CNPJ 07.919.261/0001-59. Duas observações históricas importadas. Ofertas do Supermercado Sol de Ponta Porã-MS apareceram na busca e foram excluídas por município/UF, mesmo contendo produtos da marca Sorriso.
15. **Realeza.** Referências divergem entre Perimetral Nordeste e Noroeste 2258. Não é o Real da Tancredo Neves 104. Sem catálogo atual; não habilitar até resolver a identidade/endereço e origem do preço.
16. **Docelar.** Iraí 957/Boa Esperança no cadastro estadual. Pesquisa individual não encontrou catálogo estruturado com preço. Uma fonte futura deve informar endereço e CNPJ; sem preços importados.
17. **União.** Cadastro consultado distingue Noêmia Tonello Dalmolin 2285, Blumenau 4392 e Tancredo Neves 2122; a matriz é de outro estado. O relatório histórico corresponde **somente a Tancredo Neves 2122**, registrada como `uniao-tancredo`, CNPJ 91.232.843/0005-00. Duas observações importadas nessa unidade, sem atribuí-las à filial 2285. [Evidência cadastral](https://www.econodata.com.br/consulta-empresa/91232843000330-falabrette-supermercados-ltda).
18. **Ferreirão.** Passo Fundo 1894/São Mateus, empresa M. R. Ferreira. Diretório/cadastro confirmado, catálogo atual não encontrado. Sem preços importados; não confundir unidades ou empresas com nomes semelhantes.
19. **Amorim.** Há referências a Perimetral Sudoeste 3195 e Claudino Frâncio. A rede portuguesa Amorim foi excluída. A razão social no relatório não foi automaticamente ligada à bandeira sem validação adicional. Sem preços importados.
20. **O Filezão.** Unidade São Francisco de Assis 576. Fonte cadastral relaciona filial ZL em Alencar Bortolanza 949; isso exige atualização temporal e confirmação antes de atribuir produtos. Sem catálogo/preços atuais validados.
21. **Nortão.** Caminho do Sol 518/Rota do Sol. Perfil de açougue/diretório possui categorias genéricas, sem prova de SKUs e preços da unidade. Nenhum preço importado.
22. **Berté.** [Cadastro consultado](https://cnpj.biz/03825925000160) indica baixa do CNPJ 03.825.925/0001-60 em 21/01/2025. A referência é histórica; a filial foi desativada no registro do MVP até nova confirmação. Não conta como loja integrada nem recebe preços.
23. **Líder.** Referências Lions Club 729 e 590 não foram fundidas. Catálogo homônimo de Lucas do Rio Verde foi excluído. O registro Procon em 590 não foi importado por faltar vínculo inequívoco com a filial já cadastrada em 729. Sem preço importado.
24. **Chico.** Panambi 350/Industrial no cadastro consultado. Nenhum catálogo estruturado atual validado. Futura importação requer identificação dessa unidade de Sorriso; sem preços importados.
25. **São Lourenço.** Referência de BR-163/km 742 é antiga; não comprova catálogo ou operação atual. Sem filial ativa confirmada/preço importado. Não vincular supermercados São Lourenço de outros municípios.
26. **Extra Sorriso.** Apenas referência de diretório, sem identidade/filial/catálogo atual suficientemente verificáveis. Não foi associado à rede nacional Extra. Sem preços importados.

As referências acima são um levantamento, não uma alegação de que todos os 26 negócios continuam ativos. A nova unidade do União leva o registro a **29 referências de filiais**, uma delas inativa. Não se contam 29 lojas operantes ou integradas.

## Outras fontes verificadas

O [Procon municipal](https://site.sorriso.mt.gov.br/procon) exibe pesquisa recente de combustíveis/GLP, sem catálogo de supermercado por filial; não foi importada. O [Índice Custo por Prato](https://indice.custoporprato.com.br/cidades/sorriso-mt) menciona preços de outubro de 2026 em três supermercados da cidade, mas deliberadamente identifica as lojas por pseudônimos. Como não informa qual rede/filial praticou cada preço, não serve para atribuir ofertas aos mercados cadastrados e não foi importado.

## Regras implementadas e reprodução

- Banco impõe cidade Sorriso e UF MT nas filiais; FKs ligam rede e filial.
- Importação real exige `source_city=Sorriso` e `source_state=MT`. Município ausente/divergente aborta o lote; autorização textual não remove esse filtro.
- O coletor de relatório usa nome exato + logradouro + número + município/UF. Uma marca de produto ou nome de loja contendo “Sorriso” não satisfaz a regra.
- Estoque desconhecido permanece nulo. Datas históricas não são substituídas pela data de execução. `reference_price_cents` é referência antiga; `min_price_cents` fica nulo sem oferta recente válida.
- Buscas dos consumidores continuam consultando apenas a API local/SQLite, sem requisições às lojas.

```sh
cargo build --release --locked
python3 scripts/load_public_history.py
./target/release/sorriprecos serve
curl 'http://127.0.0.1:8080/v1/products/search?q=lacta%20357g'
scripts/android-install.sh SERIAL
python3 scripts/android_real_smoke.py SERIAL
```

Para coleta atual automática, ainda é preciso uma origem pública utilizável com filial e condições verificadas ou um feed/arquivo autorizado da loja. Login do Nota MT não foi contornado, e credenciais de terceiros não foram utilizadas. O resultado entregue permite testar o fluxo com dados reais **históricos**; não equivale a encontrar preços atuais de todas as lojas.
