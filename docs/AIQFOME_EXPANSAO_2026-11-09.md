# Expansão aiqfome Sorriso — 09/11/2026

## Descoberta

O diretório público `https://aiqfome.com/restaurantes/sorriso-MT` listou **537
vitrines**. O cadastro anterior tinha 101. Para cada candidato novo, a página
pública foi lida no mesmo formato embutido usado pelo coletor, com pausa de
1,2 s entre requisições. A identidade foi conferida contra:

- `cidade_id=4430`, cidade Sorriso, estado 13;
- `menu.restaurante_id == id`;
- CNPJ, logradouro, número e CEP do vendedor;
- pelo menos uma categoria com itens e uma categoria não excluída.

## Resultado

| Situação | Quantidade |
|---|---:|
| Vitrines no diretório | 537 |
| Já cadastradas antes | 101 |
| Candidatas novas analisadas | 441 |
| Aceitas com identidade e itens | 349 |
| Duplicadas por CNPJ de fonte já cadastrada | 91 |
| Fora de Sorriso | 1 |
| Duplicadas de outra fonte do app (Egao, D’Macena, D’Tália, Top 30) | 4 |
| **Novas fontes adicionadas** | **345** |

O cadastro passou a ter **446 entradas aiqfome**, **473 fontes de comércio** e
**546 fontes no banco** (**543 no endpoint público**, que exclui três fontes DEMO). O nome exibido é derivado do
slug público; o CNPJ e o `seller_id` permanecem na origem.

## Coleta

O serviço `sorriprecos-local-food-user.service` foi criado em
`deploy/sorriprecos-local-food-user.service` e está ativo. Ele usa o agendador
existente: até 12 vitrines por lote, 120 s entre lotes e 1 s entre requisições.
Com 446 vitrines, um ciclo completo leva algumas horas; a cobertura cresce a
cada lote.

Depois de ativar `allow_portions` nas 345 fontes novas, **123 já foram
tentadas: 120 sucesso e 3 falhas**. As falhas restantes (`acai-master`,
`acai-oliveira` e `acaiteria-aurora`) têm itens que exigem complementos
obrigatórios, então o preço não é fixo e a importação é recusada corretamente.
As outras **222 fontes novas** seguem na fila do serviço, que processa 12 por
lote.

## Limites

- A coleta importa apenas itens com preço e embalagem normalizáveis. Cardápios
  baseados em complementos obrigatórios ou combinações sem preço fixo são
  rejeitados, conforme a regra existente.
- Os 91 CNPJs duplicados já estavam representados por outra fonte; não foram
  recriados.
- O diretório `api.guiacidade.net` tem 11.740 CNPJs de Sorriso, mas os serviços
  com preço são esparsos e inconsistentes (ex.: valores de teste). Não foi
  criado coletor de serviços a partir dele nesta rodada.
