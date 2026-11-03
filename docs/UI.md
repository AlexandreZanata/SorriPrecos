# Interface do aplicativo

A revisão visual aproxima o MVP da referência fornecida pelo usuário: paleta clara com verde e azul escuro, marca com carrinho, início centralizado, grade de mercados, busca em tela própria, cartões com miniatura e comparação compacta por filial. A navegação inferior oferece Início, Buscar, Favoritos e Ajustes. Todas as fontes continuam acessíveis em Fontes; a grade inicial é um atalho para seis redes, sem afirmar que suas coletas estejam ativas.

## Comportamento preservado

Busca continua usando a API local e SQLite. Paginação, favoritos, cache offline, reconexão, seleção de servidor e distinção DEMO/real são mantidos. Cada linha da comparação expande endereço, data, condições e link de origem. Inicialmente aparecem cinco filiais, com controle para visualizar todas. A melhor oferta só recebe destaque verde quando recente e elegível; observações antigas seguem sinalizadas como históricas.

## Acessibilidade e aparência

O padrão é o tema claro da referência, independentemente do tema do sistema. A e A+ na comparação selecionam texto padrão ou 20% maior, preservando também a escala de fonte configurada no aparelho. O controle de contraste usa fundo escuro, texto claro e cores com maior contraste. As mesmas opções estão em Ajustes e persistem no armazenamento local. Android ajusta os ícones das barras do sistema conforme o contraste; iOS mantém a aparência das barras definida pelo UIViewController e ainda precisa de validação em Mac/aparelho.

Navegação expõe abas e seleção para acessibilidade. Preferências são controles de alternância, e botões de ícone possuem rótulos. A interface reorganiza o conteúdo com texto maior; não há altura fixa para títulos de produtos.

## Imagens e histórico

A API atual não fornece fotos de produtos. Miniaturas são ilustrações vetoriais genéricas de categoria, acompanhadas de “Ilustração” e descrição acessível; não são fotos de embalagens comerciais. Nenhuma imagem de marca foi copiada da referência.

Não há uma série diária de preços exposta pela API. A área Histórico informa essa ausência; não desenha um gráfico de sete dias com valores inventados. As datas reais das observações continuam disponíveis em cada filial. O aviso sobre março de 2026 permanece visível no modo histórico, sem anunciar última verificação como “hoje”.

## Validação

Compile `:composeApp:assembleDebug` e execute `:composeApp:testDebugUnitTest`. Instale com `scripts/android-install.sh SERIAL`. Com o backend ativo e o aparelho físico autorizado, `python3 scripts/android_ui_smoke.py SERIAL` verifica as telas, expansão de origem, A+, contraste e persistência. O script termina no início claro com texto padrão. Capturas locais ficam em `artifacts/android-ui/`; relatório versionado em `docs/testing/android-ui-2026-11-03.json`.

Os fluxos funcionais anteriores são exercitados por `scripts/android_smoke.py` (DEMO) e `scripts/android_real_smoke.py` (registros reais históricos). Não se utiliza emulador.
