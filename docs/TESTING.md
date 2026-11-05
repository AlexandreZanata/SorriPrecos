# Verificação da entrega

Executada em 03/11/2026, em Linux, com Rust 1.99.0, Java 21, Gradle 8.14.4 e Android SDK 36. APK `0.1.0`, application ID `br.com.sorriprecos`, Android mínimo 26 e target 36.

## Atualização de cobertura — 05/11/2026

- **39 testes Rust** e **11 testes Kotlin** passaram; `cargo fmt --check`, `cargo clippy -D warnings`, `:composeApp:lintDebug` e `:composeApp:assembleDebug` concluíram sem erro.
- **101 fontes locais** (`aiqfome`, `Ola`, `Egao` e `D’Macena`) terminaram com status `success`. A Blumenau Bebidas publicou 2.626 variantes em uma página de 12,2 MiB; o leitor aceitou a página com limite de 16 MiB e normalizou 921 ofertas. As outras seis novas vitrines também concluíram.
- As varreduras paginadas do VipCommerce concluíram ao menos uma passagem completa em Sorrinorte, Sorriso e DelMoro. O Machado continua sem contexto público de Sorriso: `/busca` e o `sitemap-produtos-3.xml` identificam a filial 3, CNPJ 03.790.896/0003-00, em Sinop, e o CEP da unidade Tancredo Neves segue recusado.
- APK **0.1.1** (versionCode 2) instalado por ADB no aparelho físico. O `android_live_smoke.py` passou nos seis checks de catálogo atual, busca por GTIN, comparação, detalhes, cache offline e reconexão.

## Automatizados

**35 testes Rust passaram**, incluindo normalização de acentos; checksum GTIN; conversão e distinção de embalagens; preços inteiros; rejeição de medicamentos; migração, WAL e FKs; importação idempotente e histórico; conflito de GTIN/embalagem; preservação em lote inválido; observação antiga; busca FTS por embalagem; ordenação/vínculo de filial; promoções condicionadas; preços ausentes, vencidos, indisponíveis e desatualizados; isolamento DEMO/real e autorização; CSV/JSON; paginação; ETag; limites de busca/rate; leituras simultâneas a escritas; coleta paginada sintética; resposta 429; ausência de coleta por busca do consumidor; deduplicação de múltiplos SKUs da mesma filial; rejeição de município ausente/estrangeiro/UF incorreta; preço histórico separado do mínimo recente, com estoque desconhecido.

`cargo fmt --all -- --check` e `cargo clippy --workspace --all-targets -- -D warnings` passaram. O backend foi compilado em debug e release e iniciado com migrações reais em arquivo SQLite.

**5 testes Python passaram** para seleção estrita de município, endereço de filial, UF, centavos e preservação da data histórica. Execute `python3 -m unittest discover -s scripts -p 'test_*.py'`. A extração PDF depende opcionalmente de Poppler e `scripts/requirements-research.txt`; os testes não precisam de PDF nem rede.

**6 testes do cliente Kotlin passaram**: moeda/embalagem; contrato JSON com DEMO/centavos; fallback do cache persistente em falha; cancelamento sem resultados antigos; isolamento do cache entre servidores e isolamento do cache histórico/atual. `:composeApp:assembleDebug` passou.

`:composeApp:lintDebug` terminou com **0 erros de regras e 9 avisos de versões mais recentes**. O analisador embutido do AGP 8.13.2 também emitiu diagnósticos de metadados Kotlin 2.4 versus analisador 2.2. Portanto, esse resultado **não** comprova análise semântica Kotlin completa pelo lint. O compilador Kotlin 2.4.10 e os testes do APK passaram. Antes de lançar, alinhar uma versão de lint/AGP com suporte completo aos metadados usados; não foram suprimidos esses diagnósticos.

## Celular físico por ADB

**Não foi utilizado emulador.** Aparelho `2311DRK48G`, Android 16/API 36, resolução física 1220×2712. O teste usa `adb reverse` com loopback e não publica a API na rede.

Os **9 checks de interface passaram**:

1. Inicialização e identificação explícita dos dados DEMO.
2. Pesquisa “arroz 5kg” na API, sem resultado de embalagem 1 kg.
3. Comparação ordenada com nome de filial e horário. Valores sintéticos observados: R$ 22,70, R$ 23,90 e R$ 26,50.
4. Inclusão e visualização de favorito.
5. Favorito permanece após force-stop/reabertura do próprio aplicativo.
6. Detalhes do produto e identificação da origem sintética.
7. Comparação previamente salva continua disponível offline, com aviso.
8. Reconexão e atualização após restaurar USB.
9. Botão Voltar do Android retorna à tela anterior do aplicativo.

O teste offline removeu temporariamente apenas a rota reversa `tcp:8080`; não alterou Wi-Fi, dados móveis ou configurações do aparelho. O encaminhamento foi restaurado e o app foi deixado conectado à API. A rodada posterior utiliza o modo real histórico. A revisão visual posterior usa tema claro por padrão e alto contraste selecionável; sua validação no aparelho está descrita em UI.md e no relatório específico abaixo.

Relatório de checks preservado em `docs/testing/android-2026-11-03.json`. Capturas locais em `artifacts/android-smoke/`: `home.png`, `search.png`, `comparison.png`, `favorites.png`, `details.png`, `offline.png` e `reconnected.png`. Execute `scripts/android_smoke.py SERIAL` para repetir.

## Dados reais históricos no celular

Os **7 checks adicionais passaram**: aviso de dados reais históricos sem DEMO; busca Lacta no localhost com R$ 69,90 documentados; comparação sem destaque de oferta atual; duas filiais DelMoro corretamente identificadas; cache offline dos registros reais; reconexão; comparação de um mesmo produto de 357g entre DelMoro e DelNorte, com os respectivos valores históricos. O aplicativo foi deixado no modo real histórico.

A base tem 27 produtos/31 observações de março de 2026, sete redes/oito filiais exclusivamente de Sorriso-MT. `min_price_cents` e `effective_price_cents` ficam nulos; `available` também é nulo, pois o estoque atual não foi confirmado. Valor conhecido permanece como referência histórica.

Repetição: `python3 scripts/android_real_smoke.py SERIAL`, com API ativa, snapshot carregado e APK instalado. Evidência preservada em `docs/testing/android-real-2026-11-03.json`; capturas locais em `artifacts/android-real/`. O teste preserva as configurações de rede do telefone e restaura `adb reverse` no bloco `finally`.

## Banco e desempenho

Backup online e restauração para um novo arquivo foram executados usando `scripts/sqlite_backup.py`. Origem e cópia passaram `PRAGMA integrity_check` e conservaram 12 produtos/36 ofertas DEMO. Uma segunda rodada com o banco histórico real preservou 27 produtos/31 observações, sete redes/oito filiais, em backup e restauração para arquivos novos. Não foi sobrescrito o banco do aplicativo.

Seis cenários de benchmark combinaram 10 mil/50 mil/100 mil produtos com uma/três ofertas. Foram medidas importação, banco, RSS, CPU, RPS e p50/p95/p99 em leitura e com escritor iniciado em paralelo. Resultados e limitações em PERFORMANCE.md; nenhum teste gerou requisição a supermercado real.

Não validados: build iOS/macOS/Xcode; catálogos das demais redes; rede móvel contra VPS/HTTPS; assinatura release/lojas de aplicativos; carga sustentada e restauração de base de produção. Nenhuma VPS foi implantada.

## Revisão visual baseada na referência

APK recompilado e instalado no mesmo celular físico. A interface usa tema claro por padrão, marca com carrinho, quatro abas, grade de mercados, busca em cartões e comparação compacta com expansão de procedência. A+ aumenta o texto respeitando a escala do sistema; contraste e tamanho persistem ao reiniciar. Ilustrações genéricas são identificadas; não há série diária simulada.

Checks de UI em `scripts/android_ui_smoke.py`; evidência em `docs/testing/android-ui-2026-11-03.json` e capturas locais `artifacts/android-ui/`. A regressão DEMO foi repetida nesta revisão, com relatório em `docs/testing/android-ui-demo-2026-11-03.json`. Passaram nove checks de interface e os nove checks da regressão DEMO, além dos cinco testes Kotlin. As capturas de início, busca, comparação, texto maior, contraste e ajustes foram inspecionadas visualmente. Detalhes de implementação e limites em [UI.md](UI.md).

## Sessão anônima e catálogos atuais

A primeira coleta completa VipCommerce recebeu 35.101 itens de origem nas três filiais de Sorriso. 27.088 foram normalizados e 8.013 ficaram registrados para revisão. São 20.316 produtos canônicos; 16.011 tinham oferta disponível informada. Contadores por rede, requisições, datas e exemplo de comparação em `testing/vipcommerce-2026-11-03.json`. Os ciclos iniciais duraram 89 s, 99 s e 95 s, com 139, 130 e 120 requisições respectivamente. Quatro CDs DelMoro de outros municípios foram excluídos antes de consultar produtos.

Seis testes do adaptador verificam centavos exatos/rejeição de valores inválidos, sessão com cliente/operador/organização divergentes, cidade+CNPJ+endereço, embalagem/porção fracionada, promoção condicionada e recusa de enviar sessão a outra origem. O teste adicional de API cobre GTIN com 13/14 dígitos e isolamento DEMO/histórico. O teste histórico existente agora exige modo explícito e verifica que ele não entra nas agregações atuais. A primeira rodada física detectou a ausência da busca por GTIN no índice de texto; a consulta foi corrigida para o índice de GTIN canônico.

O coletor contínuo roda no computador local; para reiniciar após desligamento, use `collect-vip --watch`. Não foi instalado serviço global nesta máquina nem implantada VPS. O arquivo systemd está preparado para implantação.

Os **seis checks físicos com catálogo atual passaram**, usando `scripts/android_live_smoke.py`: modo atual separado de Procon/DEMO, busca por GTIN com embalagem/preço iguais à API, comparação do mesmo produto em três redes locais, detalhes com GTIN e origem, leitura do cache offline e reconexão. Exemplo observado: Arroz Urbano Parboilizado 5 kg, R$ 26,99 no Sorrinorte, R$ 27,90 no DelMoro e R$ 28,99 no Sorriso Supermercados. Esses valores são uma observação datada, não promessa de preço futuro. Relatório em `testing/android-live-2026-11-03.json`; capturas locais em `artifacts/android-live/`. A captura de comparação foi inspecionada visualmente.

Foi executado backup online posterior à coleta; origem e cópia passaram `PRAGMA integrity_check`. Sessões temporárias da investigação foram removidas e a árvore versionável foi verificada contra os valores reais de sessão/configuração, sem correspondências.

A segunda coleta automática completa também passou nas três redes, sem alteração das contagens. Intervalos observados entre os inícios: Sorrinorte 603 s, Sorriso Supermercados 609 s e DelMoro 601 s. As seis execuções e seus contadores estão no relatório VipCommerce. Os sete checks do Procon foram repetidos com o modo histórico explícito e passaram no APK atualizado.
