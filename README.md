# SorriPreços

MVP open source para comparar preços de supermercados de **Sorriso-MT**, com backend Rust/SQLite e aplicativo Kotlin Multiplatform/Compose.

**Estado da entrega:** Android instalado e testado em celular físico por ADB, API Rust/SQLite e importação CSV/JSON funcionais. O catálogo local agora contém **27 produtos reais e 31 observações históricas de sete redes de Sorriso-MT**, extraídas do relatório público Procon-MT de março de 2026. **Não há preço atual confirmado nem feed varejista autorizado ativo.** Dados históricos exibem aviso explícito e não entram como menor oferta atual. Há também 12 produtos DEMO isolados para testes. Código publicado em [AlexandreZanata/SorriPrecos](https://github.com/AlexandreZanata/SorriPrecos).

## Executar o backend

Requisitos: Rust stable, Python 3, compilador C para SQLite e acesso às dependências na primeira compilação. Execute da raiz do repositório:

```sh
cargo build --release --locked
./target/release/sorriprecos init
python3 scripts/load_public_history.py
./target/release/sorriprecos serve
```

O banco fica em `data/sorriprecos.db` e a API em `http://127.0.0.1:8080`. O comando `load_public_history.py` importa somente a amostra histórica verificada de Sorriso, sem atualizar artificialmente as datas. Para fixtures fictícias, existe o comando separado `./target/release/sorriprecos demo`. O servidor fica em primeiro plano; mantenha esse terminal aberto durante o teste no celular.

```sh
curl http://127.0.0.1:8080/health
curl 'http://127.0.0.1:8080/v1/products/search?q=lacta%20357g'
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

No app, permaneça no modo real e pesquise **Lacta**. A comparação mostra registros reais históricos, suas filiais e as datas de março de 2026; eles não são ofertas atuais. O modo **Experimentar com dados DEMO** continua disponível separadamente. Em **Conexão**, o endereço local para o teste USB é `http://127.0.0.1:8080`. Mantenha o backend e a conexão USB ativos. Ao desconectar a API, consultas já salvas continuam acessíveis com aviso offline; novas buscas ainda sem cache precisam de conexão.

Teste de interface reproduzível, em aparelho desbloqueado com o app instalado:

```sh
python3 scripts/android_real_smoke.py SERIAL_DO_CELULAR
# Opcional: depois de carregar DEMO, teste também o fluxo sintético:
# python3 scripts/android_smoke.py SERIAL_DO_CELULAR
```

O teste usa somente a interface do SorriPreços e remove temporariamente o encaminhamento USB da porta 8080 para validar offline. Ele restaura o encaminhamento ao terminar; não desliga Wi-Fi/dados móveis. Evidências ficam em `artifacts/android-smoke/`. Relato e limitações: [TESTING.md](docs/TESTING.md).

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
