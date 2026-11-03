# Entrega do MVP — 03/11/2026

O aplicativo está instalado e foi testado no celular físico Android conectado por ADB, consumindo a API Rust em `http://127.0.0.1:8080` pelo USB. Não foi utilizado emulador. O código foi publicado em [AlexandreZanata/SorriPrecos](https://github.com/AlexandreZanata/SorriPrecos).

## Dados e cobertura

- 26 redes/estabelecimentos pesquisados individualmente; 29 referências de filiais, uma inativa. Isso não constitui censo de lojas operantes.
- 27 produtos e 31 observações **reais históricas**, em sete redes e oito filiais de Sorriso-MT, extraídos do relatório público Procon-MT / Páscoa 2026. Datas originais de março de 2026, origem/página e embalagem preservadas; estoque atual desconhecido.
- **Zero preços atuais confirmados e zero feeds comerciais varejistas autorizados ativos.** O histórico público não equivale a integração automática com a loja. Não foram atribuídos preços nacionais ou de outros municípios às filiais locais.
- DelMoro, DelNorte, Machado, Sorriso Supermercados, Sol, União e Atacadão possuem registros históricos importados. Pesquisa, critérios por mercado e impedimentos estão em [SORRISO_INTEGRATIONS.md](SORRISO_INTEGRATIONS.md).
- DEMO opcional e separado: 12 produtos/36 ofertas em três lojas fictícias. Não são preços inventados atribuídos às redes reais.

## Implementação e validação

API Rust/Axum, SQLite WAL/FTS5, migrações, normalização conservadora, importação CSV/JSON transacional, histórico/retenção e adaptador de feeds autorizados. Coletor específico do relatório público filtra município, UF, nome e endereço/número; importações reais rejeitam município ausente ou diferente de Sorriso-MT. Consultas do consumidor usam apenas SQLite local.

App KMP/Compose com busca, comparação, favoritos, cache persistente e avisos explícitos para DEMO, histórico e offline. Histórico não entra no menor preço recente. Android validado; iOS preparado, mas depende de Mac/Xcode para compilação e testes.

Passaram 28 testes Rust, cinco Kotlin e cinco Python; nove checks físicos DEMO e sete checks físicos com dados reais históricos. Rustfmt/Clippy e APK debug passaram. Lint tem limitação de metadados Kotlin descrita em [TESTING.md](TESTING.md). CI contém jobs Rust/Python e Android. Backup/restauração SQLite foi verificado.

Seis benchmarks sintéticos iniciais cobrem 10 mil/50 mil/100 mil produtos com uma/três ofertas. O maior registrou banco de 122,68 MiB, RSS API de 23,81 MiB, p95 de 17,812 ms e 640,83 req/s em fase curta local; não é teste sustentado nem capacidade de VPS. São medições anteriores à adição dos campos de referência histórica: [PERFORMANCE.md](PERFORMANCE.md).

## Reproduzir

```sh
cargo build --release --locked
./target/release/sorriprecos init
python3 scripts/load_public_history.py
./target/release/sorriprecos serve
# Em outro terminal, com SDK/Java configurados:
cd apps/mobile
./gradlew :composeApp:assembleDebug
cd ../..
scripts/android-install.sh SERIAL
python3 scripts/android_real_smoke.py SERIAL
```

Use o modo real, chip Lacta, para os registros históricos. O endereço no app é `http://127.0.0.1:8080`; mantenha backend e USB ativos. APK local em `artifacts/sorriprecos-0.1.0-debug.apk`.

Para oferecer comparação de preços atuais, falta uma origem local verificável com filial, validade e condições de acesso, ou arquivo/feed autorizado da loja. Também permanecem domínio/VPS/HTTPS, carga sustentada, alinhamento lint/AGP e assinatura release para lançamento público. Não foi implantada VPS.
