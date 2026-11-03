# Catálogos públicos por filial — 03/11/2026

O coletor `collect-vip` usa a mesma sessão **anônima da vitrine** que o navegador cria antes de qualquer login de cliente. A presença de `Authorization: Bearer` não significa, por si, que seja uma conta de comprador. Na resposta observada, os campos `client`, `operator` e `vipcommerceClienteId` estavam vazios. O coletor recusa sessões que tragam esses identificadores ou uma organização diferente.

A sessão autentica o contexto da loja na API; não deve ser descrita como uma API sem autenticação. Não se utiliza login de consumidor, senha pessoal, token de operador, carrinho, pedido ou checkout. Os parâmetros de bootstrap são lidos dos módulos JavaScript publicados pelo próprio site e permanecem em memória, assim como o token. Nenhuma chave ou sessão real integra a configuração, os logs ou o Git. Uma nova sessão é criada em cada ciclo, sem depender de um JWT permanente.

## Filiais habilitadas

- **Sorrinorte:** `sorrinorte.com.br`, organização 365, vitrine global 412, filial de API 1, centro de distribuição 1. CNPJ 02.756.536/0001-67, Rua Palmares 489, Industrial 2ª Etapa, Sorriso-MT.
- **Sorriso Supermercados:** `sorrisosuper.com.br`, organização 324, vitrine global 369, filial 1, CD 1. CNPJ 14.953.277/0001-43, Brescansin 220, Centro Sul, Sorriso-MT. O domínio antigo redireciona para essa vitrine.
- **DelMoro:** `delmorosupermercados.com.br`, organização 287, vitrine global 328, filial 1, CD 1. CNPJ 00.877.761/0004-79, Belo Horizonte 94, Sorriso-MT. A lista também devolve quatro CDs de outros municípios; nenhum deles recebe requisições de produtos. As outras filiais físicas de Sorriso não herdam esse catálogo.

A validação anterior à primeira página de produtos exige domínio/organização/vitrine, CD ativo, cidade Sorriso, UF MT/Mato Grosso, CNPJ, logradouro e número. As coordenadas geográficas devolvidas pela plataforma não são usadas como prova de município: foram encontradas coordenadas inconsistentes com o endereço textual.

Configuração reproduzível, sem credenciais: `config/vipcommerce.json`. O cadastro pesquisado continua em `config/supermarkets.json`.

## Protocolo observado no frontend VipCommerce

Origem fixa: `https://services.vipcommerce.com.br`; prefixo `/api-admin/v1`.

1. `GET /organizacoes/filiais/dominio/{dominio}`: identidade pública da vitrine, sem sessão.
2. Módulo público de configuração referenciado pelo site: campos `lojaUser` e `lojaAuthJWT`. O hash de módulo fica configurado; se removido com 404, a descoberta acompanha somente referências de scripts publicadas na página e no módulo principal, com limite de oito candidatos. Uma recusa 403 não dispara essa alternativa.
3. `POST /org/{org}/auth/loja/login`: objeto `domain`, `username` e `key`, obtidos no passo anterior. Retorna a sessão anônima da loja.
4. `GET /org/{org}/loja/filiais/{filial}`: vínculo com a vitrine global.
5. `GET /org/{org}/filial/{filial}/loja/centros_distribuicoes/retiradas`: CDs e endereços, filtrados antes de buscar produtos.
6. Sob `/org/{org}/filial/{filial}/centro_distribuicao/{cd}/loja`, `GET /classificacoes_mercadologicas/departamentos/arvore`.
7. No mesmo prefixo, `GET /classificacoes_mercadologicas/departamentos/{departamento}/produtos?page=N&limit=100`.

A árvore e o paginador delimitam a coleta completa de cada departamento. O coletor verifica página, número de páginas e total de itens, deduplica o mesmo SKU presente em departamentos diferentes e registra falha se os totais mudarem durante a paginação. Não limita a coleta às ofertas ou a uma consulta por “arroz”. Links de origem usam a rota publicada `/produto/{id}/{slug}`.

Fontes primárias: [Sorrinorte](https://www.sorrinorte.com.br/), [Sorriso Supermercados](https://www.sorrisosuper.com.br/), [DelMoro](https://www.delmorosupermercados.com.br/) e seus módulos públicos `chunk-IBICK36J.js`/`chunk-DP7IR7KI.js` observados nesta data. Não foi localizado um contrato público OpenAPI utilizável para esse catálogo. Esta documentação descreve o protocolo efetivamente verificado no frontend; não é documentação oficial nem garantia de estabilidade da fornecedora. APIs de integração ERP com credenciais comerciais são outro produto e não foram utilizadas.

## Periodicidade, volume e normalização

```sh
cargo build --release --locked
./target/release/sorriprecos init
# Uma execução (respeita o intervalo desde a última tentativa):
./target/release/sorriprecos collect-vip --config config/vipcommerce.json
# Serviço contínuo, em terminal separado da API:
flock -n data/vip-collector.lock ./target/release/sorriprecos collect-vip --config config/vipcommerce.json --watch
./target/release/sorriprecos serve
```

O agendador verifica vencimentos a cada 15 s, com intervalo de 600 s por fonte desde o início da tentativa. Há uma concessão exclusiva no SQLite por fonte e `flock` para a instância de serviço. As redes são coletadas sequencialmente: não há busca disparada por consumidor nem sobreposição. Em caso de atraso, o próximo ciclo aguarda a execução anterior. Isso é **polling a cada dez minutos**, não atualização instantânea garantida. Requer o computador/serviço ligado e com rede.

Pausa mínima configurada de 300 ms entre requisições, página de até 100 itens, timeout de 25 s, resposta de até 8 MiB e no máximo três tentativas em falhas transitórias. HTTP 429 respeita `Retry-After`; adiamentos maiores que 30 s encerram a tentativa. HTTP 401/403 encerra sem procurar credenciais de cliente. O orçamento de nove minutos por fonte é verificado entre páginas; uma requisição em andamento ainda pode consumir seu timeout. Sessões não são enviadas a outro host e redirects não são seguidos. TLS permanece validado.

Todos os itens públicos recebidos são persistidos em `source_catalog_items`, com SKU, filial, JSON original, observação e motivo de revisão. Somente os normalizados com segurança entram em comparação. Embalagem ausente/ambígua, multipack sem regra, quantidade fracionária não representável ou conflito de GTIN ficam pendentes; não se inventa tamanho ou marca. Marca ausente é “Não informada”. Preço decimal é convertido exatamente em centavos. Venda por peso conserva a porção indicada pela plataforma, sem confundir preço por kg com preço da porção.

Promoções de cliente/clube/fidelidade e quantidade mínima mantêm condições e preço comum. Não entram como menor preço irrestrito. Disponibilidade é a informada pelo catálogo, não garantia de estoque físico. Só após uma leitura completa da filial, SKUs ausentes deixam de ser elegíveis como oferta atual (`available=null`); falhas não apagam preços. Histórico de alterações continua no SQLite.

`GET /v1/collectors` informa estado, última tentativa/sucesso, requisições, itens recebidos, normalizados e CDs de outros municípios excluídos. `docs/testing/vipcommerce-2026-11-03.json` registra uma execução observada, sem tokens ou dados de cliente. Não confundir itens de origem, produtos canônicos e ofertas disponíveis.

Para instalação persistente em servidor, há `deploy/sorriprecos-vip.service`. Copie a configuração para `/etc/sorriprecos/vipcommerce.json` e habilite esse serviço junto da API. Nenhuma VPS foi implantada nesta entrega.

## Machado e demais fontes

**Machado / Agile B2B:** o JavaScript público `https://assets.agilecdn.com.br/js/scripts.min.js?v=10660` cria uma sessão visitante em `GET /controllers/api-controller.php?acao=tokens`. O retorno contém `component` e `controller`. A pesquisa pública `GET /components/products-assistent.php?s=arroz&page=1&agile_token={component}` foi verificada com HTTP 200, usando apenas o token anônimo recém-emitido e seu cookie de sessão; sem token ela retornara 404. O frontend também usa `products-container.php`, `products-carousel-column.php`, `product-info.php` e `product-description.php` para fragmentos HTML públicos. Nenhuma ação de cliente/pedido foi utilizada.

Apesar de a pesquisa funcionar, a [vitrine Machado consultada](https://www.redemachado.com.br/) identifica Sinop: Av. das Itaúbas 4001, CNPJ 03.790.896/0003-00, contexto público de filial 3. O seletor público de CEP recusou o endereço da unidade Noêmia Tonello Dalmolin 2501 de Sorriso, conforme a revalidação abaixo. Não se tenta trocar IDs por adivinhação, nem se considera o número comercial “loja 11” como parâmetro de API comprovado. **Acesso anônimo confirmado; ingestão local desabilitada por falta de filial de Sorriso validada.**

**Casa Aurora / VipCommerce:** organização 289, vitrine 330, filial 1, CD 1 devolveu somente Sinop, CNPJ 26.777.276/0003-36. Não houve chamada de produtos desse CD. A unidade física de Sorriso, Otávio de Souza Cruz 441, permanece sem catálogo online local confirmado.

**DelNorte:** página atual de ofertas/tabloide sem itens utilizáveis; script público observado contém carrosséis, sem endpoint de catálogo identificado. Preços antigos indexados não foram importados como atuais.

**Atacadão:** filial física de Sorriso identificada, mas não há contexto de catálogo local validado; a conexão apresentou erro de certificado. Não se desabilitou TLS.

**Mercado Amazonas / aiqfome:** endereço de Sorriso confirmado; objeto público `restaurantenovo.menu.cardapios_categorias` observado vazio; o script da página consome esse objeto, sem catálogo disponível. Mínimo de pedido e exemplos do checkout não são preços de produtos.

As outras referências do levantamento não entregaram API pública de produtos/filial verificável na pesquisa individual. Permanecem pendentes, documentadas em `SORRISO_INTEGRATIONS.md`. Três catálogos ativos não significam cobertura integral dos 26 estabelecimentos pesquisados.

## Procon exclusivamente histórico

As origens agora têm `source_kind=retail|public_report`. A busca padrão e a comparação retornam somente o catálogo varejista; `historical=true` seleciona exclusivamente o relatório, sem misturar mínimos ou filiais com preços de outra origem. Os 27 produtos/31 observações do Procon conservam março de 2026 e estoque desconhecido. No app, use **Pesquisa histórica do Procon**; **Voltar aos preços atuais** restaura o catálogo varejista. Cache e chamadas do cliente incluem esse modo.

O acesso público da vitrine e a autorização do usuário para testar esse mecanismo não constituem licença comercial de redistribuição concedida pelos supermercados. A classificação de pesquisa continua B, não A; o estado técnico legado `authorized` indica coletor habilitado, sem afirmar contrato. A licença MIT cobre o código. Condições para lançamento e atribuição continuam em `LEGAL_CONSIDERATIONS.md`.

## Ampliação da cobertura em Sorriso em 03/11/2026

A nova revisão cruzou o diretório municipal, cadastros locais e as vitrines acessíveis por nome e município. O registro em `config/public_catalogue_audit.json` inclui os resultados por comércio e distingue as fontes aptas, os sites institucionais, os homônimos de outras cidades e as vitrines com acesso pendente.

- Babi tem endereço local no diretório da prefeitura. O domínio indicado pela ficha municipal não resolveu na consulta direta. [Ficha municipal](https://turismo.sorriso.mt.gov.br/produtos-servicos/mercado/babi-supermercados)
- Casa Aurora tem página institucional da filial local. O catálogo identificado continua apontando para Sinop. [Casa Aurora Sorriso](https://casaaurora.com.br/sorriso/)
- Machado consta no diretório de Sorriso. A sessão da vitrine Agile B2B conhecida continua associada à filial de Sinop. [Diretório municipal](https://turismo.sorriso.mt.gov.br/produtos-servicos/mercado)
- DelNorte publica as duas unidades no site. A seção de ofertas consultada não entregou preços atribuíveis à unidade de Sorriso. [DelNorte](https://delnorteatacarejo.com.br/)
- Atacadão tem página institucional da unidade. A vitrine indexada estava no CEP 02170-901 e a consulta direta falhou na validação TLS. [Atacadão Sorriso](https://www.atacadao.com.br/loja/sorriso)
- Nanai Foods mostrou preços no navegador e retirada na Avenida Tancredo Neves, 2862. O GET direto do ingestor recebeu HTTP 403 e a fonte ficou pendente. [Nanai Foods](https://sorrisonanaifoods.appsimpliza.com.br/)
- Mercado Amazonas tem uma página local com zero categorias de produtos. [Mercado Amazonas](https://aiqfome.com/MT/sorriso/mercado-amazonas)
- O site União Supermercados pertence à rede com lojas em São Paulo. A rede Falabrette de Sorriso ficou sem vitrine confirmada. [União Supermercados](https://www.uniaosuper.com.br/)

### Coleta aiqfome

O comando abaixo consulta o HTML público do Açougue Portal e do Empório Dom Manoel a cada 600 segundos por vendedor. Use sem `--watch` para executar uma tentativa. O comando `collect-vip --watch` atende às fontes VipCommerce em processo separado.

```sh
./target/release/sorriprecos collect-aiqfome --watch
```

- Açougue Portal, vendedor 128045, CNPJ 10.199.413/0001-09, Perimetral Sudoeste 1087. [Catálogo](https://aiqfome.com/MT/sorriso/acougue-portal)
- Empório Dom Manoel, vendedor 35099, CNPJ 30.398.947/0001-63, Blumenau 2694. [Catálogo](https://aiqfome.com/MT/sorriso/emporio-dom-manoel)

O ingestor lê o JSON `restaurantenovo.menu.cardapios_categorias` embutido no HTML, que alimenta a própria vitrine. Valida vendedor, CNPJ, endereço, CEP e os identificadores de Sorriso-MT antes de importar. A origem é a página pública, acessada por GET sem sessão ou credenciais. Uma configuração por vendedor define as categorias de mercearia. Combos, complementos pagos obrigatórios e embalagens ambíguas ficam fora da comparação. A coleta usa o preço base da variante, sem frete, cupom ou campanha, e a disponibilidade informada pela vitrine. Os preços saem do ranking após 30 minutos sem atualização. A gravação de preços e da auditoria ocorre na mesma transação SQLite.

Na coleta validada, o Açougue Portal apresentou 221 variantes nas categorias selecionadas, com 154 normalizadas e nenhuma disponível naquele momento. O Empório apresentou 173 variantes, com 169 normalizadas e 109 disponíveis. Quatro sucos ficaram fora por divergência entre o volume do título e o tamanho da variante. A API de acompanhamento continua em `/v1/collectors`.

No Android físico 2311DRK48G com Android 16, via ADB e API local, a busca por `picanha gril 2kg` abriu a filial do Açougue Portal com “Indisponibilidade informada”. A busca por `demerara 100g` mostrou o açúcar do Empório Dom Manoel por R$ 3,50. O teste conferiu município, UF e URL de origem de ambas as ofertas pela API.

## Comércios de alimentos e cardápios de Sorriso

A revisão de 03/11/2026 habilitou nove novas fontes. São sete vitrines aiqfome de vendedores verificados (A Casa do Salame, Armazém Polpas Naturais, Buffet Sabor de Festa, Puro Sabor, Ronaldo’s Peixaria e Restaurante, Serra Gaúcha e S&P Frescor) e duas vitrines próprias (Egao Sushi e Pizzaria D’Macena). O ingestor normalizou 849 variantes adicionais na rodada de validação. A lista municipal da plataforma expôs 531 URLs candidatas. A descoberta lista links, e a habilitação exige verificar o vendedor e os dados de cada catálogo. [Diretório do aiqfome](https://aiqfome.com/restaurantes/sorriso-MT)

```sh
python3 scripts/discover_aiqfome.py
./target/release/sorriprecos collect-local-food --watch
```

O comando `collect-local-food` é um alias de `collect-aiqfome`. Ele usa a mesma configuração e programa uma tentativa a cada 600 segundos por estabelecimento. A auditoria por fonte aparece em `/v1/collectors`. As três vitrines VipCommerce continuam em `collect-vip`.

- **aiqfome.** O ingestor lê o catálogo estruturado no HTML e valida cidade, estado, endereço, CEP, CNPJ e ID. A Casa do Salame publicou 73 variantes com outro ID de vendedor. O coletor separou essas entradas e processou o restante do catálogo local. [A Casa do Salame](https://aiqfome.com/MT/sorriso/a-casa-do-salame)
- **Egao Sushi.** O endereço de Blumenau 3675 vem do GET `/api/config`. O GET `/api/cardapio` fornece itens e opções com preço. O coletor conserva cada opção e a disponibilidade publicada. [Vitrine de pedidos](https://pedido.egaosushi.com.br/) · [API de cardápio](https://pedido.egaosushi.com.br/api/cardapio)
- **D’Macena.** A página de pedidos publica o JSON `window.__INITIAL_DATA__`. O coletor valida ID, CNPJ, Sorriso-MT e Porto Alegre 1648 antes de ler os produtos. Variantes com preço dependente de complemento, resgate ou quantidade mínima ficam na auditoria. A disponibilidade reflete as condições publicadas para o item. [Vitrine de pedidos](https://pizzariadmacena.meia7solucoes.com.br/cardapio/pedidos)
- **Outras fontes.** A P2 publica uma linha de alimentos sob cotação. A Panificadora Pimpão aponta para um cardápio legado cuja fonte atual de preços ficou pendente de validação. Nanai permanece pendente de acesso ao ingestor. [P2 Distribuidor](https://p2distribuidor.com.br/) · [Panificadora Pimpão](https://www.panificadorapimpao.com.br/) · [Nanai](https://sorrisonanaifoods.appsimpliza.com.br/)

### Correspondência de nomes e preços

A busca expande um vocabulário explícito de grafias e nomes regionais, por exemplo aipim, mandioca e macaxeira. Uma medida no fim da consulta vira um filtro da embalagem cadastrada, com conversão decimal para g ou ml. Assim, `aipim 1 kg` pode encontrar mandioca de 1.000 g, inclusive quando o título usa outra grafia.

O agrupamento de preços conserva a identidade canônica por GTIN válido ou pela combinação exata de nome, marca, quantidade e apresentação normalizados. Os sinônimos só ampliam a busca. O catálogo local preserva o estabelecimento e o rótulo da porção, como P, G, meia ou inteira, para manter receitas e tamanhos distintos em resultados próprios. Medidas conflitantes e preços condicionados a escolhas pagas ficam fora do ranking. O teste de API confirma que sinônimos localizam produtos distintos sem agrupá-los e que uma caixa de 2.500 g fica fora de uma busca de 1 kg mesmo se o título mencionar 1 kg.

A validação no celular físico, pelo ADB, encontrou a mandioca amarela congelada de 1 kg da Casa do Salame ao pesquisar `aipim amarela 1kg` e mostrou a indisponibilidade informada pela fonte. A busca por `yakimeshi salmao` abriu a opção P do Egao por R$ 82,00. A API retornou as porções P e G como itens distintos.

## Revalidação de Machado e Casa Aurora

- **Machado.** O seletor público consulta `POST /controllers/order-controller.php` com `acao=consultarLocalizacoes`, CEP e sessão visitante. O CEP **78893-110**, da filial com CNPJ **03.790.896/0011-10**, recebeu HTTP 200 com a mensagem “No momento não atendemos o CEP informado.” Os CEPs 78894-280 e 78890-000 também foram recusados. Execute `python3 scripts/check_machado_coverage.py` para repetir a consulta do CEP da filial. Uma resposta diferente exige verificar a identidade da unidade antes de coletar preços. [Vitrine](https://www.redemachado.com.br/) · [Cadastro municipal da filial](https://site.sorriso.mt.gov.br/dl/523483)
- **Casa Aurora.** Na organização VipCommerce 289, o CEP **78890-000** da filial de CNPJ **26.777.276/0005-06** devolveu entrega vazia e somente retirada em Sinop. O ingestor aceita resolver o centro de distribuição pelas rotas de retirada e entrega do CEP quando o ID ainda é desconhecido. Antes de pedir produtos, exige centro ativo em Sorriso-MT com o CNPJ, a rua e o número cadastrados. Em teste isolado, a fonte fez sete requisições e gravou zero ofertas. A entrada em `config/vipcommerce.json` está desabilitada até haver um centro local verificável. [Filial de Sorriso](https://casaaurora.com.br/sorriso/) · [Vitrine](https://www.casaaurorasupermercados.com.br/)
- **Panificadora Doce Mel.** O ingestor confirmou o vendedor 45802, CNPJ **34.813.798/0001-02** e endereço Perimetral Sudoeste 3249, Sorriso-MT. A coleta a cada 600 segundos abrange todas as categorias publicadas. Na primeira rodada, recebeu 228 variantes e normalizou 60. [Catálogo](https://aiqfome.com/MT/sorriso/panificadora-doce-mel)

Atacadão, DelNorte, Babi e Mercado Amazonas seguem pendentes pelos motivos registrados acima. O diretório de Sorriso da Pede Agora foi consultado como pista adicional. A consulta não estabeleceu uma fonte de preços para esses mercados. [Pede Agora Sorriso](https://www.pedeagora.net/lista/sorriso-mt/lojas/promocoes)

## Descoberta de estabelecimentos e validação

1. Comece pelo [diretório municipal](https://turismo.sorriso.mt.gov.br/produtos-servicos/mercado) e por categorias como padaria, açougue, hortifruti, peixaria, empório e conveniência. Cruze cada nome com “Sorriso MT”, endereço e termos como “pedidos”, “catálogo” e “cardápio”. Compare o CNPJ e o endereço para separar filiais, homônimos e vitrines duplicadas.
2. Consulte vitrines de pedidos e o site próprio. O comando `python3 scripts/discover_aiqfome.py` extrai URLs candidatas do diretório público da cidade. Para cada candidata, abra a página individual e confirme que ela publica itens, variantes e valores. O resultado do buscador ou a lista de lojas serve para descoberta. No aiqfome, o JSON `restaurantenovo` precisa conter vendedor, CNPJ, cidade, estado, CEP, endereço e o menu daquele vendedor.
3. Para um provedor novo, use a aba Rede do navegador em sessão de visitante. Se houver seletor de filial, selecione Sorriso e confira o endereço ou CNPJ na resposta antes de ler produtos. Localize o GET, JSON embutido ou sessão anônima iniciada pelo próprio frontend. Registre método, URL, paginação, formato de preço, disponibilidade e rota de fotos. Trate 401, 403, catálogo vazio, CEP recusado e localização incerta como motivos distintos no registro de auditoria.
4. Cadastre os resultados em `config/public_catalogue_audit.json`. Para habilitar uma fonte, configure o estabelecimento e a filial em `config/supermarkets.json` e a identidade esperada no arquivo do adaptador. Rode uma tentativa com `collect-local-food --config /caminho/candidatas.json` ou `collect-vip --config /caminho/candidatas.json` usando um SQLite de teste via `DATABASE_URL`. Confira em `/v1/collectors` a quantidade recebida, normalizada e rejeitada. Compare SKU, embalagem, preço e foto de alguns itens com a vitrine, incluindo itens indisponíveis e variantes com complementos.
5. Habilite o agendamento após validar a fonte e o escopo. Antes de iniciar vários processos sobre um SQLite existente, execute `sorriprecos init` para aplicar migrações. Em seguida, rode os coletores com `--watch`. Confira a busca no celular físico via ADB e mantenha no registro os motivos das fontes que ficaram pendentes.

Na rodada de 03/11/2026, o aiqfome forneceu 1.067 variantes em cinco novas vitrines. A primeira coleta normalizou 418 variantes.

- **Pim Pão**, 86 variantes. A página de pedidos atual complementa o site institucional. [Vitrine](https://aiqfome.com/MT/sorriso/pim-pao)
- **Panificadora Sousa**, 65 variantes. [Vitrine](https://aiqfome.com/MT/sorriso/panificadora-sousa)
- **Panificadora Nossa Senhora Aparecida**, 84 variantes. [Vitrine](https://aiqfome.com/MT/sorriso/panificadora-nossa-senhora-aparecida)
- **Dona Diva Empório Artesanal**, 93 variantes. [Vitrine](https://aiqfome.com/MT/sorriso/dona-diva-emporio-artesanal)
- **Ponto do Churrassco**, 90 variantes. A vitrine declara vendedor e CNPJ próprios, com o mesmo número de endereço do Açougue Portal. Cada catálogo mantém sua identidade de origem. [Vitrine](https://aiqfome.com/MT/sorriso/ponto-do-churrassco)

A Padaria Popular apareceu indexada com preços, mas a consulta direta recebeu identidade e menu vazios. A Sol e Lua publica catálogo comercial de pães e o Mercado Itália divulga ofertas e pedidos por WhatsApp. O Mercado Triunfo apareceu em um diretório de açougues. O cardápio do Steakhouse 67 e a página da Marmitaria Kau Kau exibem valores, com vigência e disponibilidade ainda pendentes de confirmação. Um índice externo usa pseudônimos para os mercados de Sorriso, o que impede atribuir preços às filiais. [Padaria Popular](https://aiqfome.com/MT/sorriso/padaria-popular) · [Sol e Lua](https://panificadorasolelua.ind.br/) · [Mercado Triunfo](https://acougue.online/anuncio/mt/sorriso/mercado-triunfo/) · [Steakhouse 67](https://www.steakhouse67.com/menu?menu=steaks) · [Marmitaria Kau Kau](https://marmitaria-da-kau-kau-deliveri.ueniweb.com/) · [Índice Custo por Prato](https://indice.custoporprato.com.br/cidades/sorriso-mt)

## Fotos e busca com erros de digitação

Os coletores usam o campo de foto do próprio item. O VipCommerce publica o arquivo em `imagem` e a vitrine o carrega do CDN de produtos na variante 250×250. O aiqfome usa `item_dir` e `imagem`; o Egao publica `foto`; a D’Macena publica `foto_principal`. O SQLite conserva a primeira URL pública encontrada para o produto canônico. As próximas coletas preenchem produtos antigos conforme as fontes voltam a publicá-los.

A API retorna `photo_path` e entrega a foto em `GET /v1/products/{id}/photo`. A primeira visualização busca o arquivo em uma origem HTTPS permitida, converte-o em WebP de até 384 pixels por lado e armazena o resultado no SQLite. O download é limitado a 5 MiB, sem redirecionamentos. Itens sem foto de origem aparecem somente com seus dados de produto. Para publicação externa, as condições de uso e redistribuição de fotos devem ser verificadas com cada fonte.

A busca tenta primeiro a consulta normal. Se não houver resultados, usa o vocabulário de nomes e marcas para aceitar troca de letras, inserção ou remoção. Uma consulta como `arorz brnaco 5kg` mantém o filtro de 5 kg. GTIN e identidade canônica continuam exatos.

## Carrinho por filial

O carrinho fica salvo no aparelho por endereço de API e modo de demonstração. Adicione pela comparação de um produto e abra **Carrinho** para alterar a quantidade. A cotação usa o preço efetivo atual de cada SKU canônico na mesma filial, multiplicado pela quantidade. O total completo aparece primeiro. Filiais com itens faltando exibem subtotal parcial e quantos itens têm preço. Valores vencidos, indisponíveis e registros históricos ficam fora da soma. A cotação não inclui frete. O limite é de 25 produtos distintos e 99 unidades por item.

Ao tocar em uma oferta, o app expande endereço, horário da observação, origem e condições. **Abrir fonte original** usa a URL da oferta. **Copiar endereço** copia o endereço da filial.

## Auditoria de fotos e bases alternativas

A checagem do SQLite em 03/11/2026 encontrou 21.929 produtos varejistas distintos, dos quais 2.824 estavam sem URL de foto após o preenchimento de parte dos cardápios na coleta seguinte. Egao e D’Macena têm fotos em parte dos itens na API pública. Exemplos como o Painço Zaeli 500 g do Sorrinorte têm campo `imagem` nulo e mostram o arquivo genérico da própria vitrine também na página de detalhes. O catálogo oficial da Zaeli lista o pacote exato, mas a validação TLS falhou no acesso direto. O cookie da Doce Mel tem imagem vazia no item. Em cada caso, o item mantém o preço e fica sem ilustração.

Para repetir a conferência, compare `products.photo_source_url` com a foto da oferta em `source_catalog_items.raw_json` (campo `imagem` no VipCommerce) e com a página pública do item. Se existir `photo_path`, consulte a rota local. HTTP 200 com `image/webp` confirma a entrega, HTTP 404 indica ausência de fonte e HTTP 503 indica que o backend não conseguiu obter ou processar a imagem naquela tentativa. Uma imagem alternativa precisa corresponder ao GTIN ou à marca, variante e embalagem verificadas no fabricante. [Painço no Sorrinorte](https://www.sorrinorte.com.br/produto/374/painco-zaeli-500g) · [Painço na Zaeli](https://www.zaeli.com.br/produto?id=557&item=PAINCO-500g) · [Doce Mel](https://aiqfome.com/MT/sorriso/panificadora-doce-mel)

- **Nota MT.** Base de preços praticados em NF-e e NFC-e com filtro de município e período. A SEFAZ exige cadastro para a consulta. O portal NFC-e oferece consulta individual; não foi confirmado um feed público de preços por filial. O Procon municipal publica médias de combustível e GLP. O relatório estadual continua exclusivamente no modo histórico. [FAQ Nota MT](https://www.sefaz.mt.gov.br/notamt/duvida) · [Portal NFC-e](https://www.sefaz.mt.gov.br/portal/nfce/) · [Procon de Sorriso](https://site.sorriso.mt.gov.br/procon)
- **DelNorte.** O link de tabloide redireciona à página da rede, que lista preços semanais e as unidades de Sorriso e Nova Mutum. A seção de preços não comprova vigência nem quais unidades participam. [Tabloide oficial](https://delnorteatacarejo.com.br/tabloide)
- **Atacadão.** A página oficial da loja abriu com o CEP padrão 02170-901. O agregador tem a ficha da filial da Rua Idemar Riedi 8150, mas os encartes examinados eram de setembro e remetiam à lista de lojas em cada folheto. [Filial oficial](https://www.atacadao.com.br/loja/sorriso) · [Índice de encartes da filial](https://seusfolhetos.com.br/lojas/sorriso/atacadao/rua-idemar-riedi-8150)
- **Casa Aurora.** O CEP 78896-042, publicado na página da filial, foi testado em um SQLite isolado. O coletor fez sete requisições e registrou zero ofertas porque a API não apresentou a filial local para retirada ou entrega. [Filial de Sorriso](https://casaaurora.com.br/lojas/sorriso-mt/)
- **Cooperriso e feiras.** A prefeitura confirma a quitanda da Cooperriso na Ceasa e lista cinco feiras. A vitrine localizada para a cooperativa falhou na validação TLS e ficou pendente de catálogo e preços verificáveis. [Quitanda da Cooperriso](https://site.sorriso.mt.gov.br/noticia/reinaugurada-quitanda-da-cooperriso-na-ceasa-68e42cd7ee5f3) · [Feiras locais](https://turismo.sorriso.mt.gov.br/produtos-servicos/feira-livre)
