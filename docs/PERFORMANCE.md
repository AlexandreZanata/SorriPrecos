# Desempenho medido

Medição local em 2026-11-03T13:52:21Z. Binário Rust release, SQLite WAL, quatro conexões, oito workers no gerador Python e 400 requisições por fase/cenário, após 20 requisições de aquecimento. Nenhuma consulta acessa supermercados.

Ambiente: 13th Gen Intel(R) Core(TM) i7-13620H, 16 CPUs lógicas; Linux-7.2.6-76070206-generic-x86_64-with-glibc2.39; Python 3.12.3. Este computador também executava tarefas de compilação e teste Android. Não se trata de uma VPS isolada.

## Resultados

| Produtos | Ofertas | Importação (s) | SQLite (MiB) | RPS | p50/p95/p99 (ms) | RAM API (MiB) | CPU (% de um núcleo) | p95 com escrita (ms) |
|---:|---:|---:|---:|---:|---|---:|---:|---:|
| 10,000 | 10,000 | 1.306 | 6.81 | 3144.81 | 2.421 / 3.789 / 4.727 | 14.14 | 274.78 | 5.602 |
| 10,000 | 30,000 | 3.673 | 11.90 | 897.39 | 8.429 / 16.001 / 20.925 | 14.67 | 94.18 | 22.511 |
| 50,000 | 50,000 | 8.330 | 34.41 | 1748.03 | 4.483 / 6.816 / 8.659 | 18.87 | 401.78 | 7.616 |
| 50,000 | 150,000 | 23.194 | 60.99 | 2027.03 | 3.890 / 5.933 / 6.852 | 18.58 | 425.31 | 6.624 |
| 100,000 | 100,000 | 10.849 | 68.61 | 1072.60 | 7.575 / 10.956 / 11.972 | 22.90 | 410.07 | 10.225 |
| 100,000 | 300,000 | 45.316 | 122.68 | 640.83 | 12.919 / 17.812 / 20.540 | 23.81 | 376.29 | 18.597 |

Todos os cenários concluíram sem erro HTTP/JSON. A RAM da API é uma leitura de RSS ao fim da fase, não um pico rigoroso. O pico observado do importador foi amostrado a cada 50 ms e está nos arquivos JSON. O tamanho SQLite é medido após checkpoint da carga inicial; não inclui overhead transitório de WAL/SHM, executáveis ou backups. CPU acima de 100% representa uso de mais de um núcleo.

Cada produto recebe uma ou três ofertas, em filiais sintéticas diferentes. Os cinco termos de busca alternam uma correspondência ampla com embalagem, busca ampla textual, prefixos seletivos e termo inexistente. O conjunto é sintético, a página tem 20 itens e as fases duram menos de um segundo: RPS é vazão de uma execução curta em loopback, não capacidade sustentável ou quantidade de usuários.

Na fase mista, um segundo processo reaplica até mil produtos enquanto os leitores iniciam. Em várias execuções o escritor termina antes do fim da fase de leitura; o campo `writer_still_running_after_read_phase` explicita essa limitação. Há também teste automatizado de 30 escritas concorrentes com 30 leituras no mesmo WAL. Teste de longa duração, disco lento, TLS, rede celular e carga sustentada na VPS continuam pendentes.

## Ajuste guiado pela medição

A consulta inicial agrupava as ofertas de todos os resultados FTS antes de aplicar LIMIT. No cenário inicial de 10 mil produtos/30 mil ofertas, 120 requisições e oito workers, p95 foi 102.453 ms. A consulta passou a materializar apenas os IDs da página antes de agregar os preços. Na rodada posterior de 400 requisições, o mesmo tamanho de catálogo apresentou p95 de 16.001 ms. As rodadas não foram isoladas nem tiveram a mesma duração; o resultado é indicativo e o mecanismo é reproduzível pelo SQL atual, não uma promessa de ganho fixo.

Não foram introduzidos caches em RAM, motores externos ou infraestrutura extra. A paginação anterior à agregação é a principal alteração fundamentada na medição. Etags/gzip reduzem bytes, mas não tiveram ganho de vazão isoladamente atribuído neste benchmark.

## Reproduzir

```sh
cargo build --release --locked
python3 scripts/benchmark.py --sizes 10000 50000 100000 --offers 1 3 --requests 400 --workers 8 --output artifacts/novo-benchmark
```

O script recusa reutilizar um arquivo de banco existente. Usa porta 8099 e não toca na base `data/sorriprecos.db`. Os resultados brutos preservados estão em `docs/benchmarks/2026-11-03.json` e `2026-11-03-before-pagination.json`. Repita no hardware da VPS, com o mesmo conjunto de versões e carga sustentada, antes de dimensionar uso público.
