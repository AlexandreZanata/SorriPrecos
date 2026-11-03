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

Apesar de a pesquisa funcionar, a [vitrine Machado consultada](https://www.redemachado.com.br/) identifica Sinop: Av. das Itaúbas 4001, CNPJ 03.790.896/0003-00, contexto público de filial 3. Não foi encontrado seletor público validado para a unidade Noêmia Tonello Dalmolin 2501 de Sorriso. Não se tenta trocar IDs por adivinhação, nem se considera o número comercial “loja 11” como parâmetro de API comprovado. **Acesso anônimo confirmado; ingestão local desabilitada por falta de filial de Sorriso validada.**

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
