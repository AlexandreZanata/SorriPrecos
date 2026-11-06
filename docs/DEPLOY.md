# VPS Linux

Uma única VPS executa Caddy, API e coletor agendado. Os limites nos units são pontos iniciais, não garantia de capacidade. Revise os resultados locais em PERFORMANCE.md e repita o benchmark na VPS escolhida.

1. Compile `cargo build --release --locked` para a arquitetura da VPS.
2. Crie usuário de sistema `sorriprecos`, `/opt/sorriprecos` para o binário, `/var/lib/sorriprecos` com permissão somente ao usuário do serviço e `/etc/sorriprecos` para configuração privada. A API não exige o checkout do código para executar: migração e registro de fontes são embutidos no binário.
3. Copie `target/release/sorriprecos` para `/opt/sorriprecos/sorriprecos`. Inicialize como o usuário do serviço: `sorriprecos --database-url sqlite:///var/lib/sorriprecos/catalog.db init`. Não carregue DEMO na base de produção.
4. Instale os units de `deploy/` em `/etc/systemd/system`, ajuste domínio no Caddyfile e configure DNS. Mantenha 8080 apenas em loopback; exponha 80/443 pelo Caddy.
5. Copie `config/sources.example.json` para `/etc/sorriprecos/sources.json`. Habilite somente feeds com autorização e contrato validado. Restrinja a leitura/escrita da configuração ao operador e ao grupo do serviço.
6. Copie `config/aiqfome.json`, `config/vipcommerce.json` e `config/radar.json` para `/etc/sorriprecos/`. Execute `systemctl daemon-reload` e `systemctl enable --now sorriprecos.service sorriprecos-collect.timer sorriprecos-maintain.timer sorriprecos-vip.service sorriprecos-local-food.service sorriprecos-pharmacy.service sorriprecos-therapeutica.service sorriprecos-radar.service`. Verifique `journalctl -u sorriprecos` e `/health` por HTTPS. O serviço do Atacadão precisa de `SORRIPRECOS_ATACADAO_CA_FILE` quando a rede faz inspeção TLS; sem o CA confiável ele falha fechado e mantém os encartes já importados.
7. Configure no app a URL HTTPS da sua API, gere e assine um APK release com chave própria. Não publique o APK debug como versão final.

O timer de coleta roda por hora com jitter. A configuração de cada rede controla seu intervalo efetivo. `flock` impede dois coletores ao mesmo tempo. Falhas são registradas e não apagam o catálogo. O unit diário de manutenção aplica retenção mesmo se todas as fontes estiverem falhando.

O limite HTTP padrão de 180/min considera o IP da conexão. Atrás do Caddy, é um limite agregado; configure `RATE_LIMIT` no serviço conforme medições e coloque limitação no proxy antes de atender muitos clientes. Não habilite confiança irrestrita em cabeçalhos de IP.

## Backup e restauração

Use a API de backup do SQLite, que inclui corretamente as alterações ainda em WAL. Não copie apenas o arquivo `.db` de uma base ativa.

```sh
python3 scripts/sqlite_backup.py backup /var/lib/sorriprecos/catalog.db /backup/catalog-2026-11-03.db
python3 scripts/sqlite_backup.py restore /backup/catalog-2026-11-03.db /var/lib/sorriprecos/restored.db
```

O script verifica `integrity_check` e recusa sobrescrever destinos existentes. Para trocar uma base: pare API, timer e coletor; faça um último backup; restaure para um novo nome; preserve o banco anterior; aponte `DATABASE_URL` para a cópia restaurada; ajuste proprietário; reinicie e valide `/health`, busca e contagens. Não renomeie uma base enquanto há processos ou arquivos WAL/SHM ativos.

Agende backups diários com cron/systemd do operador, retenção definida e cópia em outro disco/local. Valide uma restauração periodicamente. A entrega testa backup/restauração em base sintética local; nenhuma VPS foi provisionada.
