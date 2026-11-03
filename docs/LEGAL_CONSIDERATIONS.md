# Dados e operação responsável

Decisões do MVP: não há cadastro de consumidores, coleta de localização, publicidade, analytics ou sincronização de favoritos. Favoritos e consultas em cache ficam no aparelho. A API recebe o termo de busca e o IP da conexão; o rate limiter mantém identificadores de IP apenas em memória por uma janela curta, com tamanho limitado. Os logs padrão não registram o conteúdo das consultas. O operador ainda deve avaliar os logs do proxy e do provedor da VPS.

A [LGPD, Lei 13.709/2018](https://www.planalto.gov.br/ccivil_03/_ato2015-2018/2018/lei/l13709.htm) e o [Marco Civil, Lei 12.965/2014](https://www.planalto.gov.br/ccivil_03/_ato2011-2014/2014/lei/l12965.htm) são referências para a revisão antes do lançamento. A página consolidada da LGPD apresentou bloqueio de acesso durante esta pesquisa; não se apresenta aqui uma conclusão jurídica ou transcrição do seu texto vigente. Requisitos concretos de base legal, transparência, atendimento ao titular e retenção devem ser definidos conforme a operação real e orientação jurídica adequada.

Informações comerciais públicas não são automaticamente um feed com licença de redistribuição comercial. A classe A exige autorização ou condições expressas que cubram a finalidade, o mecanismo de acesso, a frequência, a filial, o volume e eventual uso de nomes/imagens. Um arquivo entregue pela loja deve conservar proveniência e data, além de uma referência verificável ao acordo. O campo de autorização aceita uma referência documental; o software não comprova sozinho a validade do acordo.

Não foram habilitados coletores reais nesta entrega. Não se contornam CAPTCHA, autenticação, robots, 403, rate limits ou medidas de segurança. Não se usam credenciais privadas nem endpoints internos protegidos. Uma negociação com a loja pode fornecer um CSV/JSON simples e reduzir custo e impacto para ambas as partes.

O aplicativo deve informar cobertura incompleta, observação, validade e condições comerciais. Preços condicionados a clube ou quantidade mínima não entram como melhor preço irrestrito. Dados DEMO não podem ser anunciados como ofertas locais. O consumidor deve poder abrir a origem quando ela for fornecida.

Antes de publicar: verificar identidade e filiais, autorização de cada fonte, política de privacidade proporcional aos dados efetivamente tratados, retenção de logs, canal de correção/remoção, domínio HTTPS e responsável pela atualização. A licença MIT do código não substitui as licenças dos dados ou o cumprimento de obrigações aplicáveis.
