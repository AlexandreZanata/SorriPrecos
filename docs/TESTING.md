# Verificação da entrega

Executada em 03/11/2026, em Linux, com Rust 1.99.0, Java 21, Gradle 8.14.4 e Android SDK 36. APK `0.1.0`, application ID `br.com.sorriprecos`, Android mínimo 26 e target 36.

## Automatizados

**28 testes Rust passaram**, incluindo normalização de acentos; checksum GTIN; conversão e distinção de embalagens; preços inteiros; rejeição de medicamentos; migração, WAL e FKs; importação idempotente e histórico; conflito de GTIN/embalagem; preservação em lote inválido; observação antiga; busca FTS por embalagem; ordenação/vínculo de filial; promoções condicionadas; preços ausentes, vencidos, indisponíveis e desatualizados; isolamento DEMO/real e autorização; CSV/JSON; paginação; ETag; limites de busca/rate; leituras simultâneas a escritas; coleta paginada sintética; resposta 429; ausência de coleta por busca do consumidor; deduplicação de múltiplos SKUs da mesma filial; rejeição de município ausente/estrangeiro/UF incorreta; preço histórico separado do mínimo recente, com estoque desconhecido.

`cargo fmt --all -- --check` e `cargo clippy --workspace --all-targets -- -D warnings` passaram. O backend foi compilado em debug e release e iniciado com migrações reais em arquivo SQLite.

**5 testes Python passaram** para seleção estrita de município, endereço de filial, UF, centavos e preservação da data histórica. Execute `python3 -m unittest discover -s scripts -p 'test_*.py'`. A extração PDF depende opcionalmente de Poppler e `scripts/requirements-research.txt`; os testes não precisam de PDF nem rede.

**5 testes do cliente Kotlin passaram**: moeda/embalagem; contrato JSON com DEMO/centavos; fallback do cache persistente em falha; cancelamento sem resultados antigos; isolamento do cache entre servidores. `:composeApp:assembleDebug` passou.

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

Não validados: build iOS/macOS/Xcode; feed varejista atual; rede móvel contra VPS/HTTPS; assinatura release/lojas de aplicativos; carga sustentada e restauração de base de produção. Nenhuma VPS foi implantada.

## Revisão visual baseada na referência

APK recompilado e instalado no mesmo celular físico. A interface usa tema claro por padrão, marca com carrinho, quatro abas, grade de mercados, busca em cartões e comparação compacta com expansão de procedência. A+ aumenta o texto respeitando a escala do sistema; contraste e tamanho persistem ao reiniciar. Ilustrações genéricas são identificadas; não há série diária simulada.

Checks de UI em `scripts/android_ui_smoke.py`; evidência em `docs/testing/android-ui-2026-11-03.json` e capturas locais `artifacts/android-ui/`. A regressão DEMO foi repetida nesta revisão, com relatório em `docs/testing/android-ui-demo-2026-11-03.json`. Passaram nove checks de interface e os nove checks da regressão DEMO, além dos cinco testes Kotlin. As capturas de início, busca, comparação, texto maior, contraste e ajustes foram inspecionadas visualmente. Detalhes de implementação e limites em [UI.md](UI.md).
