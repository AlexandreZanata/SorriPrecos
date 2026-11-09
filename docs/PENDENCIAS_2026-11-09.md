# O que ainda ficou para trás — 09/11/2026

## Corrigido nesta revisão

- **API `/v1/supermarkets` estourava o timeout de 10 s** com 543 fontes. A consulta
  foi reescrita com subconsultas correlacionadas e recebeu os índices
  `offers(supermarket_id, collected_at)`, `offers(supermarket_id, product_id)` e
  `branches(supermarket_id, active)`. Migração: `0010_api_indexes.sql`. Resposta
  caiu de 30 s para ~0,07 s.
- **Duas fontes aiqfome falhavam por endereço desatualizado.** `caseirinhos-da-vivi`
  e `cheiro-e-tempero-marmitaria` foram corrigidas para os endereços atuais e
  importaram 81 e 34 itens.
- **Duas fontes sem filial:** `São Lourenço` ganhou a filial Rod. BR-163, km 742;
  `Mercado Extra Sorriso` foi removido por não ter endereço confirmado.
- **21 falhas de menu normalizável** caíram para **3** após ativar
  `allow_portions` nas 345 fontes novas. As 3 restantes (`acai-master`,
  `acai-oliveira`, `acaiteria-aurora`) exigem complementos obrigatórios e não têm
  preço fixo; a recusa é correta.

## Ainda em andamento

- **222 das 345 novas vitrines aiqfome** ainda não foram tentadas. O serviço
  `sorriprecos-local-food-user.service` segue em lotes de 12; não é perda, é fila.
- **3 falhas de cardápio** acima, que precisam de complemento obrigatório.

## Pendências reais

- **17 mercados sem coletor e sem catálogo público:** Babi, Novos Campos, Real,
  Amazonas, Londrina, Vipi, Sol, Realeza, Docelar, União, Ferreirão, Amorim,
  Filezão, Nortão, Berté, Líder e Chico. Nenhum foi encontrado no diretório
  aiqfome; os que têm ofertas antigas (`sol`, `uniao`, `delnorte`, `atacadao`)
  não têm coletor ativo.
- **71 fontes de farmácia / 83 filiais**, mas a maioria sem catálogo público. As
  novas 27 do Guia Cidade estão pendentes; Farmácia Economize é Linktree;
  Nebrasca responde 403/certificado; FarMelhor, Farma & Farma e Madeira não
  publicam preço local de Sorriso.
- **11.518 CNPJs** do Guia Cidade fora do app. Os 29 com site foram testados:
  são institucionais, hotéis, agro, Instagram ou bloquearam acesso. Não há
  catálogo de produto/preço por SKU neles.
- **Machado e Casa Aurora:** sem catálogo público atribuível a Sorriso. Nada de
  Sinop foi importado.
- **Serviços:** o Guia Cidade tem 11.740 CNPJs, mas serviços com preço são
  esparsos/inconsistentes. Para um diretório de serviços seria necessária uma
  aba/modelo separado; o catálogo atual é de ofertas.

## Estado

- 546 fontes no banco; 543 no endpoint público (3 DEMO).
- 71 fontes de farmácia / 83 filiais.
- 446 entradas aiqfome; 123 novas tentadas, 120 sucesso, 3 falhas; 222 na fila.
