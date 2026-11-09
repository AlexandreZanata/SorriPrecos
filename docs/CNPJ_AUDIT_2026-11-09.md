# Auditoria de CNPJs de Sorriso — 09/11/2026

Fonte: API pública `https://api.guiacidade.net/api/v1/companies` (mesma base do
app Guia Cidade). A consulta foi paginada em 118 páginas, sem login.

## Números

| Situação | Quantidade |
|---|---:|
| Empresas listadas | 11.747 |
| CNPJs únicos | 11.744 |
| CNPJs já presentes no app antes da auditoria | 475 |
| CNPJs ainda não catalogados | **11.518** |

Categorias com mais CNPJs não catalogados: Serviços (6.956), Construção (791),
Transporte (605), Automotivo (557), Saúde (392), Imóveis (380), Jurídico (293),
Moda (251) e Mercado (182).

## Teste de catálogo público

- **29** empresas não catalogadas informam site. A maioria é site institucional
  (Cargill, Fiagril, Sicredi, Coacen, Bunge, Amaggi), hotel, agro ou Instagram.
  Nenhuma vitrine de produto com preço por SKU foi localizada.
- **8** têm alguma avaliação no Guia Cidade; os serviços com preço são esparsos
  e alguns valores são de teste.
- **54** estão na categoria “Farmácias e Drogarias”. Desses, **27** tinham
  endereço/CNPJ ainda não cadastrado e foram adicionados como fontes pendentes
  em `config/pharmacies_sorriso.json`; 12 eram duplicados de endereço/CNPJ, 14
  eram veterinária/agro/pet/clínica e 1 era programa público.
- A única farmácia nova com site é **Farmácia Economize Sorriso**, CNPJ
  17.577.531/0001-44, Avenida Tancredo Neves, 2991. O site
  `farmaciaeconomize.com.br` é um Linktree, sem catálogo de produto/preço.

## Resultado no app

- Farmácias: **71 fontes / 83 filiais** após os 27 novos CNPJs.
- Total de fontes: **546 no banco / 543 no endpoint público** (três DEMO excluídas).
- Nenhum preço foi criado para CNPJ sem catálogo público. As 27 farmácias novas
  aparecem como **coleta pendente**, com CNPJ, endereço e site quando existe.
- A ingestão de produtos desta rodada veio da expansão aiqfome (ver
  [AIQFOME_EXPANSAO_2026-11-09.md](AIQFOME_EXPANSAO_2026-11-09.md)).

## Limite

Os 11.518 CNPJs restantes são, em sua maioria, empresas de serviço sem site ou
sem catálogo público. Cadastrá-los como fonte de preço produziria milhares de
entradas vazias. Para um diretório de serviços, o caminho correto é uma aba/modelo
separado, não o catálogo de ofertas.
