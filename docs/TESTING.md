# Verificação da entrega

Executada em 03/11/2026, em Linux, com Rust 1.99.0, Java 21, Gradle 8.14.4 e Android SDK 36. APK `0.1.0`, application ID `br.com.sorriprecos`, Android mínimo 26 e target 36.

## Automatizados

**26 testes Rust passaram**, incluindo normalização de acentos; checksum GTIN; conversão e distinção de embalagens; preços inteiros; rejeição de medicamentos; migração, WAL e FKs; importação idempotente e histórico; conflito de GTIN/embalagem; preservação em lote inválido; observação antiga; busca FTS por embalagem; ordenação/vínculo de filial; promoções condicionadas; preços ausentes, vencidos, indisponíveis e desatualizados; isolamento DEMO/real e autorização; CSV/JSON; paginação; ETag; limites de busca/rate; leituras simultâneas a escritas; coleta paginada sintética; resposta 429; ausência de coleta por busca do consumidor; deduplicação de múltiplos SKUs da mesma filial.

`cargo fmt --all -- --check` e `cargo clippy --workspace --all-targets -- -D warnings` passaram. O backend foi compilado em debug e release e iniciado com migrações reais em arquivo SQLite.

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

O teste offline removeu temporariamente apenas a rota reversa `tcp:8080`; não alterou Wi-Fi, dados móveis ou configurações do aparelho. O encaminhamento foi restaurado e o app foi deixado na comparação DEMO, conectado à API. Tema escuro foi inspecionado visualmente; o tema claro está implementado conforme o sistema, mas não teve renderização separadamente validada no aparelho.

Relatório de checks preservado em `docs/testing/android-2026-11-03.json`. Capturas locais em `artifacts/android-smoke/`: `home.png`, `search.png`, `comparison.png`, `favorites.png`, `details.png`, `offline.png` e `reconnected.png`. Execute `scripts/android_smoke.py SERIAL` para repetir.

## Banco e desempenho

Backup online e restauração para um novo arquivo foram executados usando `scripts/sqlite_backup.py`. Origem e cópia passaram `PRAGMA integrity_check` e conservaram 12 produtos/36 ofertas DEMO. Não foi sobrescrito o banco do aplicativo.

Seis cenários de benchmark combinaram 10 mil/50 mil/100 mil produtos com uma/três ofertas. Foram medidas importação, banco, RSS, CPU, RPS e p50/p95/p99 em leitura e com escritor iniciado em paralelo. Resultados e limitações em PERFORMANCE.md; nenhum teste gerou requisição a supermercado real.

Não validados: build iOS/macOS/Xcode; integração varejista real; rede móvel contra VPS/HTTPS; assinatura release/lojas de aplicativos; carga sustentada e restauração de base de produção. Nenhuma VPS foi implantada.
