# Fontes e rastreabilidade

Data da pesquisa: 03/11/2026. A matriz inicial está em SUPERMARKETS_AUDIT.md; a revisão individual e os critérios de separação geográfica estão em SORRISO_INTEGRATIONS.md. O registro executável correspondente está em `config/supermarkets.json`; não contém contatos pessoais ou credenciais.

Fontes primárias utilizadas:

- [Diretório municipal de mercados](https://turismo.sorriso.mt.gov.br/produtos-servicos/mercado): confirmação inicial de Babi, DelMoro e Sorriso Supermercados.
- [DelMoro](https://delmoro.com.br/): unidades locais.
- [Casa Aurora](https://casaaurora.com.br/lojas/): unidade e vínculo do endereço de compra online.
- [DelNorte](https://delnorteatacarejo.com.br/nossas-lojas): unidade local.
- [Atacadão Sorriso](https://www.atacadao.com.br/loja/sorriso): unidade; contexto de CEP impede atribuir automaticamente o preço nacional à cidade.
- [Sorrinorte no Google Play](https://play.google.com/store/apps/details?id=br.com.sorrinorte.omnichannel): declaração do desenvolvedor VipCommerce sobre loja online em Sorriso.
- [Cadastros de credenciados SETASC](https://www.setasc.mt.gov.br/documents/364148/0/ESTABELECIMENTOS%2BCREDENCIADOS%2B-%2B08%2BJULHO/194aafab-a05c-d30f-d065-fe56c965e4dd): estabelecimentos adicionais. Registro em credenciamento não comprova operação atual.
- [Pesquisa Procon-MT Páscoa 2026](https://www.procon.mt.gov.br/documents/d/procon/tabela-produtos-analisados-pascoa-final): evidências de filiais, sem importação de valores históricos como atuais.

Diretórios secundários (Solutudo, GuiaPJ, Casa dos Dados, Inteligue, Nova Infortel, Página Amarela e WorldOrgs) servem como pistas documentadas, não como autorização de uso ou prova definitiva de catálogo. Links por estabelecimento constam da matriz.

Referências técnicas oficiais consultadas para a seleção de versões:

- [Compatibilidade Kotlin Multiplatform](https://kotlinlang.org/docs/multiplatform/multiplatform-compatibility-guide.html).
- [Compatibilidade Compose](https://kotlinlang.org/docs/multiplatform/compose-compatibility-and-versioning.html).
- [Compose 1.11.1](https://kotlinlang.org/docs/multiplatform/whats-new-compose-111.html).
- [Engines Ktor](https://ktor.io/docs/client-engines.html).

Não foram incorporadas imagens de folhetos, marcas gráficas ou bases proprietárias dos supermercados. A licença MIT cobre o código do projeto; não concede direitos sobre dados de terceiros.

Fontes adicionais da revisão individual:

- [IBGE — Sorriso](https://www.ibge.gov.br/cidades-e-estados/mt/sorriso.html): município/UF e código 5107925.
- [Loja vinculada pelo DelMoro](https://www.delmorosupermercados.com.br/): catálogo público e CD de Belo Horizonte/Sorriso posteriormente verificados; ver PUBLIC_CATALOGUES.md.
- [Sorriso Supermercados — app local](https://play.google.com/store/apps/details?id=br.com.sorrisosuper.omnichannel): distinto do Mais Sorriso de Uberlândia.
- [Mercado Amazonas no aiqfome](https://aiqfome.com/MT/sorriso/mercado-amazonas): vendedor/endereço locais confirmados, sem cardápio de produtos/preços observado.
- [Procon municipal](https://site.sorriso.mt.gov.br/procon): pesquisa recente exibida é de combustíveis/GLP, não catálogo de supermercado; não importada.
- [Índice Custo por Prato em Sorriso](https://indice.custoporprato.com.br/cidades/sorriso-mt): observações de outubro de 2026, mas supermercados deliberadamente pseudonimizados. Não foram atribuídos seus preços a nenhuma rede/filial identificada.

O subconjunto histórico Procon-MT foi extraído por células, restrito a Sorriso-MT e a filiais mapeadas: 27 produtos/31 observações de março de 2026. Snapshot, contadores de rejeição, origem/página e SHA-256 do documento em `datasets/procon-2026-sorriso/`. Não são preços atuais nem um feed contratado das redes.

Revisão de APIs públicas: [Sorrinorte](https://www.sorrinorte.com.br/), [Sorriso Supermercados](https://www.sorrisosuper.com.br/) e [DelMoro](https://www.delmorosupermercados.com.br/) usam a origem pública `services.vipcommerce.com.br`. Foram verificadas as respostas de identidade, sessão visitante, filiais/CDs, árvore e páginas de produtos. Casa Aurora retornou somente Sinop. O [Machado](https://www.redemachado.com.br/) publicou o bootstrap visitante Agile B2B; a vitrine identificou Sinop e não foi atribuída a Sorriso. Rotas, parâmetros e evidências sanitizadas em PUBLIC_CATALOGUES.md.
