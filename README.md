# SorriPreços

MVP open source para comparar preços de supermercados de **Sorriso-MT**, com backend Rust/SQLite e aplicativo Kotlin Multiplatform/Compose.

**Estado da entrega.** 20 catálogos públicos de estabelecimentos de Sorriso-MT, com coleta a cada dez minutos. O app usa fotos publicadas nas vitrines e aceita erros de digitação na busca. O Procon permanece na pesquisa histórica. Consulte o [protocolo de descoberta e validação](docs/PUBLIC_CATALOGUES.md#descoberta-de-estabelecimentos-e-validação).

## Executar o backend

Requisitos: Rust stable, Python 3, compilador C para SQLite e acesso às dependências na primeira compilação. Execute da raiz do repositório:

```sh
cargo build --release --locked
./target/release/sorriprecos init
./target/release/sorriprecos collect-vip --config config/vipcommerce.json
python3 scripts/load_public_history.py # opcional, separado do catálogo atual
./target/release/sorriprecos serve
```

O banco fica em `data/sorriprecos.db` e a API em `http://127.0.0.1:8080`. O comando `load_public_history.py` importa somente a amostra histórica verificada de Sorriso, sem atualizar artificialmente as datas. Para fixtures fictícias, existe o comando separado `./target/release/sorriprecos demo`. O servidor fica em primeiro plano; mantenha esse terminal aberto durante o teste no celular.

```sh
curl http://127.0.0.1:8080/health
curl 'http://127.0.0.1:8080/v1/products/search?q=arroz%205kg'
curl http://127.0.0.1:8080/v1/collectors
# Em outro terminal, mantenha o coletor ativo:
flock -n data/vip-collector.lock ./target/release/sorriprecos collect-vip --config config/vipcommerce.json --watch
```

`DATABASE_URL`, `BIND` e `RATE_LIMIT` são configuráveis; alternativamente use `--database-url sqlite:///caminho/catalog.db` **antes** do subcomando. Não exponha HTTP local diretamente à internet; a implantação utiliza Caddy/HTTPS.

## Compilar e testar no celular Android

Requisitos: Java 21, Android SDK plataforma 36, ADB e depuração USB autorizada. `ANDROID_HOME` deve apontar para o SDK; se `ANDROID_SDK_ROOT` estiver definido, use o mesmo caminho.

```sh
cd apps/mobile
./gradlew :composeApp:assembleDebug :composeApp:testDebugUnitTest
cd ../..
scripts/android-install.sh
```

O script seleciona um único celular conectado, instala o APK e cria `adb reverse tcp:8080 tcp:8080`. Para selecionar explicitamente um aparelho, passe o serial como argumento. Não inicia emulador. O APK está em `apps/mobile/composeApp/build/outputs/apk/debug/composeApp-debug.apk`.

No app, use a busca padrão por **Arroz 5kg**, café, leite ou GTIN para os catálogos atuais. **Pesquisa histórica do Procon** abre os registros antigos separadamente; **Voltar aos preços atuais** restaura a busca varejista. DEMO continua opcional. Em **Ajustes → Conexão**, use `http://127.0.0.1:8080`. Mantenha backend, coletor e USB ativos. Consultas salvas continuam legíveis offline, com aviso; novas consultas precisam de conexão.

Teste de interface reproduzível, em aparelho desbloqueado com o app instalado:

```sh
python3 scripts/android_live_smoke.py SERIAL_DO_CELULAR
# Histórico separado:
python3 scripts/android_real_smoke.py SERIAL_DO_CELULAR
# Opcional: depois de carregar DEMO, teste também o fluxo sintético:
# python3 scripts/android_smoke.py SERIAL_DO_CELULAR
```

Os testes usam somente a interface do SorriPreços e remove temporariamente o encaminhamento USB da porta 8080 para validar offline. Ele restaura o encaminhamento ao terminar; não desliga Wi-Fi/dados móveis. Evidências ficam em `artifacts/android-smoke/`. Relato e limitações: [TESTING.md](docs/TESTING.md).

## Interface e acessibilidade

O aplicativo usa o visual claro da referência, com carrinho, grade de mercados, busca em cartões e comparação compacta. Ajustes oferece texto maior e alto contraste, com preferências persistentes. Fotos indisponíveis usam ilustrações identificadas; não há gráfico com dados inventados. Detalhes e validação em [UI.md](docs/UI.md).

## Preparar o iOS

Targets arm64 para aparelho e simulador Apple Silicon, cliente Darwin e persistência NSUserDefaults estão preparados. O shell Swift e a configuração XcodeGen estão em [apps/mobile/iosApp](apps/mobile/iosApp/README.md). Gere o projeto em um Mac com Xcode e configure a assinatura. **Build iOS não foi validado em Linux.**

## Importar preços e habilitar fontes

Feeds/arquivos fornecidos pelo varejista exigem referência documental de autorização. O snapshot público histórico tem comando e proveniência próprios. Campos, limites, validade e formatos estão em [FEEDS.md](docs/FEEDS.md). Dados reais exigem `source_city: "Sorriso"` e `source_state: "MT"`; o importador rejeita município/UF divergentes ou ausentes. CSV/JSON têm preço inteiro em centavos e identificação obrigatória de filial.

```sh
./target/release/sorriprecos import --source demo-a --file backend/fixtures/demo.csv
# Somente após receber dados e autorização da loja:
./target/release/sorriprecos import --source sorriso --file /caminho/precos.json \
  --authorization 'Acordo de fornecimento de dados, referência interna 123'
```

`config/sources.example.json` contém uma configuração pendente para cada rede. Copie para um arquivo privado, registre autorização, URL e formato e habilite somente fontes verificadas. `collect --config /caminho/sources.json` executa as fontes vencidas sequencialmente. Nenhuma pesquisa do consumidor provoca coleta. Não há scraper de API privada ou integração ficticiamente declarada como pronta.

## Verificação e desempenho

```sh
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all -- --check
cd apps/mobile
./gradlew :composeApp:testDebugUnitTest :composeApp:lintDebug :composeApp:assembleDebug
```

Foram executados testes de normalização, GTIN, embalagens, centavos, migração/WAL, idempotência, rollback, falhas, busca, ranking, estados de validade, concorrência, rate limit, autorização e isolamento DEMO, além de testes do cliente/cache e verificações no aparelho. Veja [TESTING.md](docs/TESTING.md).

O benchmark reproduzível cria bases sintéticas separadas, com 10 mil, 50 mil e 100 mil produtos e uma/três ofertas por produto:

```sh
python3 scripts/benchmark.py --output artifacts/minha-medicao
```

O diretório precisa ser novo para não sobrescrever medições. Parâmetros, hardware e resultados em [PERFORMANCE.md](docs/PERFORMANCE.md). Não são estimativas de usuários simultâneos em produção.

## Arquitetura e implantação

Um binário Rust, Axum/Tokio/Serde/SQLx, SQLite com WAL/FTS5, pool pequeno, coletor agendado e Caddy. Sem PostgreSQL, Redis ou microserviços. O app usa Compose, Ktor, Coroutines e Serialization; favoritos não exigem cadastro.

- [Arquitetura](docs/ARCHITECTURE.md), [banco](docs/DATABASE.md) e [API](docs/API.md).
- [Análise individual e separação por cidade/filial](docs/SORRISO_INTEGRATIONS.md), [auditoria](docs/SUPERMARKETS_AUDIT.md) e [fontes](docs/SOURCES.md).
- [Implantação, backup e restauração](docs/DEPLOY.md); units e Caddyfile em `deploy/`.
- [Uso de dados e privacidade](docs/LEGAL_CONSIDERATIONS.md), [prioridades](docs/ROADMAP.md).

## Contribuir

Abra uma proposta com problema, evidência, filial e autorização quando envolver fonte real. Para código, mantenha as regras determinísticas, valores em centavos e distinção entre DEMO e real; rode os checks relevantes e documente alterações de contrato. Não envie credenciais, bancos reais, dados pessoais ou arquivos privados de autorização. Use migrações novas ao alterar bancos já implantados.

Código licenciado sob [MIT](LICENSE). Dados e marcas de terceiros dependem de suas próprias condições de utilização. As mudanças foram enviadas ao repositório GitHub indicado pelo usuário.

## Regenerar os dados históricos públicos

O snapshot conferido está em `datasets/procon-2026-sorriso/`, com referência de página e hash do PDF original. Para repetir a extração, instale `pdfplumber` de `scripts/requirements-research.txt` e Poppler (`pdftotext`). O coletor usa somente o relatório público, sem acessar login ou endpoints do Nota MT:

```sh
python3 scripts/collect_procon_sorriso.py --output artifacts/nova-coleta-procon
python3 -m unittest discover -s scripts -p 'test_*.py'
python3 scripts/android_real_smoke.py SERIAL_DO_CELULAR
```

São aceitas apenas linhas com município Sorriso, UF MT no endereço, loja e logradouro/número correspondentes ao mapeamento de filiais. Lojas de outras cidades, varejistas fora do escopo, embalagens incompletas e marcas não confirmadas são excluídos. Detalhes em [SORRISO_INTEGRATIONS.md](docs/SORRISO_INTEGRATIONS.md).
