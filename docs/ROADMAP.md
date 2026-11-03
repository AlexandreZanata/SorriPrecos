# Execução do MVP

Prioridade: um fluxo Android ↔ API Rust ↔ SQLite reproduzível e preços com origem explícita.

1. Concluído: levantamento inicial de fontes locais, com evidências e pendências explícitas.
2. Concluído: migrações, normalização conservadora, FTS5, API e importação CSV/JSON transacional.
3. Concluído: adaptador genérico de feeds autorizados e testes de contrato sintéticos. Integrações varejistas reais continuam pendentes de autorização/validação.
4. Concluído: app KMP/Compose conectado à API, comparação, favoritos e cache persistente.
5. Concluído: APK instalado e nove checks de interface executados em celular físico Android por ADB, conforme orientação do usuário.
6. Concluído: 26 testes Rust, cinco Kotlin e seis cenários sintéticos de 10 mil, 50 mil e 100 mil produtos.
7. Concluído: documentação VPS/TLS, teste de backup/restauração, limites conhecidos e preparação do iOS.

Próximas prioridades: obter pelo menos um feed de loja de Sorriso com autorização e filial identificada; reconfirmar os endereços divergentes; validar preços reais no aparelho; configurar domínio/VPS e repetir carga sustentada; alinhar lint/AGP aos metadados Kotlin; assinar APK de produção. iOS depende de macOS/Xcode.
