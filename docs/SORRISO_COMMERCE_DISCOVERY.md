# Descoberta de comércio eletrônico em Sorriso-MT

O [OpenCNPJ](https://github.com/AlexandreZanata/OpenCNPJ) é o ponto de partida para conferir CNPJ e endereço. Com o snapshot 2026-10, o filtro de estabelecimentos ativos no município RFB 9907 devolveu 23.134 registros. As divisões 47 (varejo) ou 56 (alimentação), no CNAE principal ou secundário, selecionaram 6.573 candidatos únicos, incluindo 60 com CNAEs 4771701, 4771702 ou 4771703 de farmácia humana. A triagem cadastral não comprova a existência de loja virtual.

```sh
python3 scripts/discover_opencnpj_sorriso.py /caminho/snapshot.ocnpj \
  --opencnpj /caminho/opencnpj > /tmp/sorriso-candidatos.json
```

1. Cruze o CNPJ completo com o rodapé, endereço de retirada ou identificador de filial exposto pela vitrine. Para redes, confira também a loja ou o CEP usado na resposta que contém a oferta.
2. Abra o catálogo público como visitante e confira a resposta usada pela vitrine para exibir produto, preço e disponibilidade. Use a navegação pública e a sessão anônima da própria loja quando aplicável.
3. Vincule o preço à identidade da loja virtual ou da filial verificada. Confira SKU, apresentação, quantidade, preço em reais, disponibilidade e URL. Valores sob consulta e opções obrigatórias sem preço final ficam fora. Estoque desconhecido não entra no ranking.
4. Registre a evidência por fonte, teste em SQLite isolado e compare o resultado com a vitrine antes de ativar a coleta principal.

## Farmácias verificadas

- [Therapeutica](https://www.therapeuticafarmacia.com.br/) tem uma loja virtual vinculada ao CNPJ ativo 01.780.384/0001-75, Avenida Curitiba, 2734, Sorriso-MT. O cadastro fiscal e a vitrine usam nomes diferentes para o bairro, com coincidência de CNPJ, logradouro, número e CEP. As páginas públicas têm preço e estoque em JSON-LD. Itens [sob encomenda](https://www.therapeuticafarmacia.com.br/actrisave) podem expor R$ 0,01 no HTML e são descartados.
- [Farma & Farma no aiqfome](https://aiqfome.com/MT/sorriso/farma-farma) identifica a unidade de Sorriso (26.661.675/0001-75), mas a vitrine consultada devolveu zero categorias.
- [FarMelhor Centro](https://unidades.franquiafarmelhor.com.br/farmelhor-centro-sorriso-mt-73568) tem páginas locais de produtos sem preço público verificável.
- [Drogaria Nebrasca](https://drogarianebrasca24horas.com.br/contato/), [Farmácia Aurora](https://www.farmaciaaurora.com.br/sorriso-mt/) e [Fórmula Certa](https://www.formulacertasorriso.com.br/) têm presença local. As páginas consultadas direcionam para atendimento ou orçamento.
- [Pague Menos](https://www.paguemenos.com.br/), [Drogaria São Paulo](https://www.drogariasaopaulo.com.br/) e [Drogasil](https://www.drogasil.com.br/) têm filiais ativas no OpenCNPJ e catálogos nacionais. Falta comprovar a identidade local na resposta pública que contém preço e estoque. Para Ultra Popular, o cadastro confirmou filiais locais e ainda falta comprovar uma vitrine local com preços.

## Coleta

```sh
cargo run -- collect-pharmacy --limit 40
cargo run -- collect-pharmacy --limit 40 --watch
```

O coletor lê o sitemap e visita um lote de até 40 páginas por ciclo de dez minutos. O cursor fica no SQLite. O ciclo seguinte continua a varredura e os preços coletados expiram em quatro horas. HTTP 429 interrompe a rodada e respeita pelo menos dez minutos ou um `Retry-After` maior. A periodicidade é por lote. A cobertura completa depende do tamanho e da disponibilidade da vitrine.

O app oferece Mercado e Farmácia como buscas separadas. A API de busca recebe `sector=pharmacy` para a farmácia. O padrão continua sendo Mercado.
