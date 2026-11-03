# Arquitetura

Um binário Rust oferece comandos separados `serve`, `import`, `collect`, `maintain` e `demo`. Uma API Axum/Tokio atende exclusivamente o SQLite local. Nenhuma consulta do consumidor chama supermercados. Não há serviços externos obrigatórios, login, pagamentos ou telemetria.

O coletor roda por systemd timer, em outro processo do mesmo binário. Cada estabelecimento tem uma instância configurada do adaptador `AuthorizedFeedAdapter`, implementando `SourceAdapter`. Os 26 registros de pesquisa começam pendentes; não existem raspadores ou endpoints privados disfarçados de integração. O adaptador reutilizável consome CSV, arrays JSON ou páginas JSON com `items` e `next`, após autorização documentada pelo operador.

Paginação aceita somente a mesma origem do feed. Redirects não são seguidos. As requisições têm timeout, tamanho máximo, retentativas limitadas e backoff. `Retry-After` longo encerra a execução para reagendamento, sem insistência. Coletas são sequenciais, inclusive entre lojas que compartilham domínio. O intervalo padrão por fonte é de 24 horas, com mínimo de uma hora. O timer horário apenas verifica quais fontes estão vencidas.

Cada página é uma transação atômica de até 10 mil linhas/8 MiB. Uma página inválida não altera os seus registros. Páginas anteriores válidas permanecem se uma página posterior falhar; o cursor HTTP não avança e a próxima execução pode reaplicá-las idempotentemente. Nenhuma ausência de SKU ou falha de rede apaga ofertas. Disponibilidade precisa ser informada explicitamente pela fonte.

SQLite usa WAL, chaves estrangeiras, busy timeout de 10 s, synchronous NORMAL e pool de quatro conexões. Índices atendem produto/filial/tempo. FTS5 normaliza acentos e a consulta usa termos prefixados, escapados pelo servidor, com paginação por ID. O importador mantém o índice FTS na mesma transação. Histórico grava somente mudanças de valores; manutenção elimina histórico anterior a 90 dias e execuções anteriores a 30 dias.

Preços usam inteiros em centavos. Produtos reais e DEMO têm identidades distintas. O menor preço considera dados disponíveis de até 48 horas e validade não expirada. Preço de clube/atacado nunca substitui o preço comum no ranking geral. A API expõe também condições, preço promocional, observação, origem e filial.

O aplicativo usa Kotlin Multiplatform, Compose, Ktor, Coroutines e Serialization. Interface, modelos, cliente, cache e regras de apresentação são compartilhados; Android utiliza CIO e SharedPreferences; iOS utiliza Darwin e NSUserDefaults. O cache guarda no máximo 60 respostas; favoritos são locais e limitados a 200 produtos. Consultas têm debounce de 350 ms e cancelamento; falha de conexão permite leitura do último resultado salvo, explicitamente marcado offline. Não há sincronização de favoritos com servidor.

O APK debug permite HTTP somente para loopback e o host do emulador. O teste autorizado nesta entrega utiliza exclusivamente celular físico via `adb reverse`; nenhuma porta é publicada na rede. Builds release exigem HTTPS. A configuração do endereço da API está acessível em Conexão.

A API limita 64 consultas concorrentes, 180 consultas/minuto por IP de conexão e até 50 resultados por página. O mapa de IPs tem limite de memória. ETag, Cache-Control de 30 s e gzip reduzem transferência. Atrás do Caddy, o IP da conexão é o proxy; o limite passa a ser compartilhado. Não se confia automaticamente em X-Forwarded-For enviado pelo cliente. Para lançamento, dimensionar o limite total conforme os benchmarks e implementar limitação no proxy se necessário.

iOS está preparado para aparelho arm64 e simulador Apple Silicon. Compose 1.11 não oferece o antigo target Intel x64 utilizado em versões anteriores. Não houve compilação iOS em Linux.
