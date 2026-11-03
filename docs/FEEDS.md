# Importação e feeds autorizados

Todas as redes descobertas começam desabilitadas em `config/sources.example.json`. Há uma configuração independente por rede; campos de autorização e URL são intencionalmente nulos. O operador precisa obter e guardar a autorização, confirmar a filial e ajustar o formato antes de habilitar uma fonte. Uma referência de autorização é uma declaração do operador, não uma verificação jurídica automática feita pelo software.

JSON é um array de objetos; CSV usa cabeçalho com os mesmos campos. Valores opcionais ficam nulos/ausentes no JSON ou vazios no CSV. Booleanos são `true`/`false`; preço não aceita ponto ou vírgula decimal. `quantity` é inteiro, com unidade g/kg/ml/l/un. Para 1,5 kg, use 1500 g.

Campos obrigatórios: `source_id`, `branch_id`, `name`, `brand`, `quantity`, `unit`, `presentation`, `category`, `collected_at`. Importação real também exige `source_city: "Sorriso"` e `source_state: "MT"` em cada linha; ausência, outra cidade ou outra UF rejeitam o lote. DEMO pode omitir esses dois campos. Disponibilidade ausente significa desconhecida, não disponível. Campos opcionais: `gtin`, `price_cents`, `promotional_price_cents`, `offer_type` (padrão regular), `conditions`, `available`, `source_url`, `valid_until`.

Exemplo sintético completo em `backend/fixtures/demo.json` e `demo.csv`. Cada linha identifica uma filial; os códigos aceitos podem ser consultados em `/v1/branches`. Lotes com até 10 mil registros/8 MiB são importados atomicamente. Divida arquivos maiores em lotes; não concatene filiais ou produtos de redes diferentes na mesma importação.

```sh
# Inicializar fontes DEMO; não cria nenhuma integração real.
cargo run -- demo
cargo run -- import --source demo-a --file backend/fixtures/demo.csv

# Somente depois de receber arquivo/autorização da loja:
cargo run -- import --source sorriso --file /caminho/precos.json \
  --authorization 'Acordo de fornecimento de dados, referência interna 123'
```

Adaptador HTTP aceita array JSON, CSV ou página no formato `{"items":[...],"next":"/feed?page=2"}`. `next` pode ser URL relativa ou absoluta, sempre da mesma origem. Fonte deve usar HTTPS; HTTP de localhost é permitido para testes de contrato. Não segue redirecionamentos e não admite credenciais embutidas em URLs. Não há suporte a login em plataformas varejistas.

Copie a configuração para um arquivo privado. Por exemplo, uma entrada fornecida pelo estabelecimento deve conter `enabled: true`, `authorization_reference`, `url`, `format`, `interval_seconds: 86400`, `max_pages: 100`, além de `supermarket_id`. Não use uma URL hipotética como se fosse uma API existente.

```sh
cargo run -- collect --config /caminho/sources.json
```

ETag/Last-Modified são persistidos somente ao completar todas as páginas. `collection_runs.requests` mede requisições reais, incluindo retentativas. Até a obtenção de um feed verdadeiro, não há medição de requisições de catálogo real. O teste automatizado do feed sintético de duas páginas mede duas requisições.

Datas devem representar a observação na origem, não a data de publicação de um arquivo antigo. `demo` é um comando separado que atualiza apenas as fixtures fictícias para facilitar os testes de interface. O importador normal nunca altera as datas fornecidas.

## Relatório público histórico

`python3 scripts/load_public_history.py` carrega o snapshot verificado em `datasets/procon-2026-sorriso/` pelo comando separado `import-public-report`. Não exige autorização comercial fictícia: registra proveniência pública histórica, sem afirmar contrato com a loja. Esse caminho aceita somente a URL/página do relatório Procon-MT conhecido, datas antigas e aviso `HISTÓRICO`, além de todas as regras de escopo e filial do importador comum. A validação de campos não autentica o conteúdo contra o PDF; a extração auditada e o hash do documento fornecem sua rastreabilidade.

Para reproduzir a extração: instale Poppler (`pdftotext`) e as dependências de `scripts/requirements-research.txt`; execute `python3 scripts/collect_procon_sorriso.py`. O coletor faz uma leitura do PDF público, sem acessar Nota MT autenticado. Município deve ser uma coluna própria, e nome, logradouro/número e Sorriso-MT devem coincidir com `config/procon_sorriso_branches.json`. Não infere a cidade pelo nome do produto ou da rede.

As datas das notas de março de 2026 e o estoque desconhecido são preservados. Nenhum cronômetro transforma esse snapshot em preço atual. Sete redes com observações históricas não representam sete integrações comerciais automáticas; ver `SORRISO_INTEGRATIONS.md`.
