# API pública v1

Base local: `http://127.0.0.1:8080`. Respostas JSON UTF-8; preços inteiros em centavos e datas Unix UTC em segundos. Consultas não exigem cadastro. Não existe endpoint administrativo público.

- `GET /health`: saúde do serviço e acesso ao SQLite.
- `GET /v1/supermarkets?demo=false`: `items` com estado da integração, classe, contagens e última observação.
- `GET /v1/branches?demo=false`: `items` com rede, filial, endereço, cidade e última observação.
- `GET /v1/products/search?q=arroz%205kg&limit=20&after=0&demo=false`: produtos, `next_after` e `is_demo`.
- `GET /v1/products/{id}`: produto com embalagem, marca e menor preço recente conhecido.
- `GET /v1/products/{id}/prices`: `product`, `items` por filial, `coverage_note`. Inclui filiais sem oferta, com valores nulos.
- `GET /v1/catalog/status`: redes/filiais referenciadas, importadas, integrações automáticas, produtos reais e DEMO, última observação real e janela de frescor.

`demo` deve ser explicitamente `true` para encontrar o catálogo sintético. IDs dos produtos também retornam `is_demo`; não se deve inferir sua natureza pelo número do ID. `imported_supermarkets` contabiliza redes com algum preço importado localmente; `automatic_integrations` contabiliza fontes que tiveram uma coleta autorizada efetivamente concluída. Descoberta não equivale a integração.

Busca exige de 2 a 100 caracteres, `limit` de 1 a 50 e cursor não negativo. A ordenação de produtos por ID permite paginação estável. Na comparação, ofertas recentes são ordenadas pelo preço efetivo comum, seguidas por filiais sem preço válido. Empates mantêm ordem de rede/filial.

Estados de oferta:

- `recent`: disponibilidade confirmada, preço e observação de até 48 horas, validade não expirada.
- `stale`: preço conhecido, mas observado há mais de 48 horas.
- `unavailable`: fonte informou indisponibilidade; a data indica quando isso foi constatado.
- `unknown`: nenhuma oferta, preço/estoque desconhecido ou validade expirada.

`effective_price_cents` é nulo fora do estado `recent`. Em ofertas com condições, utiliza-se o preço comum; o promocional é exibido separadamente. A interface informa que se trata do menor preço conhecido entre fontes cobertas.

As consultas de catálogo têm ETag e cache de 30 s; envie `If-None-Match` para receber 304 quando aplicável. Gzip é negociado por Accept-Encoding. Erros usuais: 400 para limites inválidos, 404 para ID inexistente, 429 com Retry-After de 60 s para excesso de consultas e 504 para timeout. A API não retorna detalhes de banco em erros 500. Limite padrão é compartilhado quando a conexão chega por proxy; veja ARCHITECTURE.md.

```sh
curl 'http://127.0.0.1:8080/v1/products/search?q=arroz%205kg&demo=true'
curl 'http://127.0.0.1:8080/v1/products/1/prices'
```
