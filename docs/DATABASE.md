# Banco SQLite

`backend/migrations/0001_catalog.sql` é aplicada automaticamente pelo SQLx. O `Cargo.lock` fixa o conjunto de dependências testado. Migrações já aplicadas não devem ser editadas em produção; crie uma migração posterior.

- `supermarkets`: rede, site, classe da auditoria, estado automático e indicador DEMO.
- `branches`: unidade referenciada em Sorriso-MT, nome, endereço e código externo quando verificado.
- `products`: produto canônico, GTIN, marca, quantidade inteira em g/ml/un, apresentação, categoria, nome normalizado e identidade estável.
- `products_fts`: índice de nomes, marcas e equivalências de embalagem (5000g/5kg, 1000ml/1l).
- `source_products`: SKU e título da origem, URL original e produto canônico. Um SKU não pode mudar silenciosamente de produto.
- `offers`: uma oferta por SKU/filial, valores em centavos, disponibilidade trivalente, condições, observação UTC e validade opcional. FKs compostas impedem filial de outra rede.
- `collection_runs`: início/fim, resultado, linhas importadas, requisições HTTP, falha resumida e referência de autorização para importações locais.
- `price_history`: valores anteriores, inseridos somente quando um valor muda.
- `source_cursors`: ETag, Last-Modified e última coleta bem-sucedida.

GTINs de 8/12/13/14 dígitos passam pelo dígito verificador e são armazenados com 14 dígitos. Zeros iniciais não criam produtos diferentes. Mesmo GTIN com quantidade, unidade ou apresentação incompatível causa rejeição do lote. Sem GTIN, usa-se nome normalizado exato, marca, quantidade base, unidade e apresentação. Não há agrupamento aproximado por similaridade; variantes permanecem separadas. Equivalências textuais diferentes podem exigir curadoria; prefere-se perder uma correspondência a comparar produtos distintos.

Categorias farmacêuticas e medicamentos comuns são rejeitados. Essa lista determinística não substitui a curadoria de um feed: o operador deve fornecer somente itens dentro do escopo de supermercado. Não há classificação automática universal de medicamentos.

O índice de preços começa por `product_id`; outro índice atende filial/tempo. FTS é uma tabela dedicada, sem gatilhos que dependeriam de colunas artificiais. Escritas externas diretas ao banco não são suportadas; utilize o importador.

`collect` não marca um preço como novo ao receber HTTP 304: preserva a data de observação declarada. `import` ignora snapshots mais antigos de uma oferta. Ausência em um lote incremental não significa indisponibilidade. `maintain` usa checkpoint PASSIVE e PRAGMA optimize, sem VACUUM durante consultas.
