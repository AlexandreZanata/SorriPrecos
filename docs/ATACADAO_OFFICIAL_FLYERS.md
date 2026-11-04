# Encarte público do Atacadão de Sorriso

Revalidado em 04/11/2026. A [página oficial da filial](https://www.atacadao.com.br/loja/sorriso) respondeu com HTTPS válido. O `__NEXT_DATA__` contém `props.pageProps.storeInfo` com `storeId=899`, Sorriso-MT, Rua Idemar Riedi, 8150, e uma lista `flyers` com URL do PDF e datas. O [encarte observado](https://apigw.cloud.carrefour.com.br/api-middleware-flyer-services/api/v2/Flyer/?id=1Po4Z8zo6x1jUvJwEoJV-PT8eiSItBYBl) confirma a filial e vigência de 03/11 a 09/11/2026. Os PDFs são servidos pelo endpoint público de documentos do grupo Carrefour; não foi necessária autenticação. É um encarte parcial, não o catálogo completo da loja.

O PDF observado contém JSON `TABLOIDI-DADOS 2.0` no campo `Subject`: páginas, descrições, caixas, códigos de foto e campos de preços. A legenda impressa confirma que `P` é o preço varejo/atacado, `PSEG` exige oferta ativada no aplicativo e `PTER` exige aplicativo e cartão participante. O preço comum é armazenado em `price_cents`; o preço de aplicativo é registrado como condicional e a condição também informa o preço com cartão e o limite. O preço com cartão não é tratado como menor preço geral. As coordenadas e nomes de arquivos de foto não são URLs de imagem e não foram inventados.

O estoque individual não é informado pelo encarte. As observações entram com `available = NULL`. A busca e a comparação exibem o preço anunciado com o aviso **estoque não confirmado**, separado dos preços confirmados. Elas não entram no menor preço nem nos totais do carrinho. O app esconde a observação quando vence a validade ou se a coleta fica desatualizada. Ofertas com descrições de vários sabores/tipos, multipacks ambíguos, embalagens retornáveis ou sem medida explícita são ignoradas. Na primeira coleta principal, o filtro final aceitou **90 itens**; dez itens aceitos em uma passagem preliminar tiveram os preços removidos por ambiguidade, com histórico mantido.

## Reprodução e limites

Requisitos: Python 3 com banco de fusos horários, `pdfinfo` e `pdftotext` do Poppler, binário Rust do projeto e acesso HTTPS. A partir da raiz do projeto:

```sh
cargo build --locked --bin sorriprecos
python3 scripts/collect_atacadao_flyers.py --database-url sqlite:///tmp/atacadao-teste.db --binary target/debug/sorriprecos
python3 -m unittest discover -s scripts -p test_collect_atacadao_flyers.py
```

O coletor confere o identificador e o endereço da loja, a validade tanto no estado da página como no texto do PDF, a legenda de preços e o domínio do documento; formatos sem esses sinais são ignorados. A interpretação dos campos foi verificada para o documento acima, e novas diagramações podem precisar de adaptação. A coleta não desabilita TLS, não contorna login e não adivinha identificadores de filial. O carimbo de expiração usa o fim do dia em `America/Cuiaba`.

Neste computador, `deploy/sorriprecos-atacadao-user.service` foi habilitado como serviço de usuário e repete a coleta a cada 600 segundos após cada rodada. O serviço requer que o binário `artifacts/sorriprecos-atacadao` exista; depois de atualizar o código, reconstrua e copie o binário e reinicie o serviço. O serviço de usuário inicia quando o gerenciador do usuário estiver ativo, portanto não é uma garantia de execução enquanto o computador ou a sessão estiverem desligados. Inspecione `journalctl --user -u sorriprecos-atacadao-user.service` para distinguir agendamento de coleta bem-sucedida.

## Machado: canal adicional

O [Clube Mais Machado](https://web-clubemaismachado.bnex.com.br/) usa Zoombox/Bnex. A página inicial e a rota `/tabloide`, consultadas como visitante, exibiram formulário de CPF e senha. O [aplicativo oficial](https://play.google.com/store/apps/details?id=br.com.gsgroup.zoombox.machadoprimaveras) anuncia loja virtual e ofertas de clube, mas isso não comprova uma rota anônima de preços da filial de Sorriso. Nenhuma credencial pessoal foi usada. A vitrine comercial Agile continua vinculada a Sinop; o Clube Mais permanece uma pista para investigação pública futura, sem ingestão de preços de outra cidade.
