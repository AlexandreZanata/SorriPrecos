# Entrega do MVP — 03/11/2026

O aplicativo está instalado e foi testado no celular físico Android conectado por ADB, consumindo a API Rust em `http://127.0.0.1:8080` pelo USB. Não foi utilizado emulador. O código foi publicado em [AlexandreZanata/SorriPrecos](https://github.com/AlexandreZanata/SorriPrecos).

## Dados e cobertura

- 26 redes/estabelecimentos pesquisados individualmente; 29 referências de filiais, uma inativa. Isso não constitui censo de lojas operantes.
- 27 produtos e 31 observações **reais históricas**, em sete redes e oito filiais de Sorriso-MT, extraídos do relatório público Procon-MT / Páscoa 2026. Datas originais de março de 2026, origem/página e embalagem preservadas; estoque atual desconhecido.
- **Três catálogos públicos atuais:** Sorrinorte, Sorriso Supermercados e DelMoro Belo Horizonte. Primeira coleta: 35.101 itens recebidos, 27.088 normalizados, 20.316 produtos canônicos, 16.011 com oferta disponível informada. Intervalo de dez minutos, sem login de cliente. Não equivale a contrato de redistribuição comercial.
- DelMoro, DelNorte, Machado, Sorriso Supermercados, Sol, União e Atacadão possuem registros históricos importados. Pesquisa, critérios por mercado e impedimentos estão em [SORRISO_INTEGRATIONS.md](SORRISO_INTEGRATIONS.md).
- DEMO opcional e separado: 12 produtos/36 ofertas em três lojas fictícias. Não são preços inventados atribuídos às redes reais.

## Implementação e validação

API Rust/Axum, SQLite WAL/FTS5, migrações, normalização conservadora, importação CSV/JSON transacional, histórico/retenção e adaptador de feeds autorizados. Coletor específico do relatório público filtra município, UF, nome e endereço/número; importações reais rejeitam município ausente ou diferente de Sorriso-MT. Consultas do consumidor usam apenas SQLite local.

App KMP/Compose com busca, comparação, favoritos, cache persistente e avisos explícitos para DEMO, histórico e offline. Histórico não entra no menor preço recente. Android validado; iOS preparado, mas depende de Mac/Xcode para compilação e testes.

Passaram 35 testes Rust, seis Kotlin e cinco Python; nove checks físicos DEMO, sete checks físicos com dados reais históricos e seis checks físicos com catálogos atuais. Rustfmt/Clippy e APK debug passaram. Lint tem limitação de metadados Kotlin descrita em [TESTING.md](TESTING.md). CI contém jobs Rust/Python e Android. Backup/restauração SQLite foi verificado.

Seis benchmarks sintéticos iniciais cobrem 10 mil/50 mil/100 mil produtos com uma/três ofertas. O maior registrou banco de 122,68 MiB, RSS API de 23,81 MiB, p95 de 17,812 ms e 640,83 req/s em fase curta local; não é teste sustentado nem capacidade de VPS. São medições anteriores à adição dos campos de referência histórica: [PERFORMANCE.md](PERFORMANCE.md).

## Reproduzir

```sh
cargo build --release --locked
./target/release/sorriprecos init
./target/release/sorriprecos collect-vip
python3 scripts/load_public_history.py # histórico opcional
# Em terminal separado: collect-vip --watch
./target/release/sorriprecos serve
# Em outro terminal, com SDK/Java configurados:
cd apps/mobile
./gradlew :composeApp:assembleDebug
cd ../..
scripts/android-install.sh SERIAL
python3 scripts/android_live_smoke.py SERIAL
python3 scripts/android_real_smoke.py SERIAL
```

Use Arroz 5kg para os preços atuais; Pesquisa histórica do Procon → Lacta para os registros antigos. O endereço no app é `http://127.0.0.1:8080`; mantenha backend e USB ativos. APK atualizado em `artifacts/sorriprecos-live-debug.apk`.

As demais fontes permanecem pendentes de catálogo/contexto local verificável; Machado e Casa Aurora retornaram contexto de Sinop e foram excluídos. Protocolo e implantação do coletor em PUBLIC_CATALOGUES.md. Para lançamento, permanecem domínio/VPS/HTTPS, carga sustentada, alinhamento lint/AGP e assinatura release para lançamento público. Não foi implantada VPS.

## Revisão de interface

A interface foi aproximada da referência fornecida pelo usuário, com tema claro, marca com carrinho, grade de mercados, cartões com ilustrações, quatro abas e comparação compacta. Texto maior e alto contraste persistem entre reinícios. Nenhum preço, foto ou gráfico fictício foi apresentado como dado real. APK atualizado no celular físico: nove checks de UI, nove de regressão DEMO e cinco testes Kotlin passaram. Evidências e limitações em [UI.md](UI.md) e [TESTING.md](TESTING.md).
