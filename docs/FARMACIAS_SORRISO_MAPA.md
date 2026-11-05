# Mapa das farmácias de Sorriso-MT — 05/11/2026

Levantamento público para o escopo Sorriso-MT (IBGE 5107925). A base combina o CNES (estabelecimentos tipo 43), o diretório farmaciaqui.net, os sites oficiais das redes e o cadastro já existente em `config/pharmacies.json`. O inventário bruto está em `config/pharmacy_audit.json`; esta página resume o que tem catálogo público e o que depende de integração comercial.

## Resultado

- **73 registros** brutos e **61 chaves de endereço** após consolidar duplicatas de nome/endereço.
- **Catálogo público com preço/estoque por filial:** Therapeutica (matriz Curitiba 2734; a filial Blumenau 2265 existe e não é uma segunda fonte automaticamente atribuível), Drogaria São Paulo Brescansin e Pague Menos Brescansin/Tancredo.
- **Estoque público sem preço local:** três unidades Drogasil (Blumenau 2995, Brescansin 488 e Tancredo 2942).
- **Somente aplicativo (E-Delivery/FEBRAFAR/Farmarcas):** Farma & Farma, Ultra Popular e FarMelhor. Os apps são personalizados por rede e o catálogo/estoque não tem endpoint web público verificado. Não foram descompilados; a obtenção de produtos exige feed/API autorizado da rede.
- **App/vitrine sem vínculo local confirmado:** Farmácia Madeira. A vitrine publica catálogo, mas a retirada/preço da unidade de Sorriso não foi confirmada; o app é próprio e não foi descompilado.
- **Programa público:** Farmácia Popular, Farmácia Cidadã (seis unidades), CAF e congêneres; não são ofertas de varejo comparáveis.
- **Demais farmácias independentes:** sem catálogo público de produto/preço verificado; contato por WhatsApp, site institucional ou diretório local.

## Inventário completo

| ID | Farmácia | CNPJ/CNES | Endereço | Telefone | Status | Rota |
|---|---|---|---|---|---|---|
| ph-09 | DROGARIA ULTRAPOPULAR AV BLUMENAU | 10341045001660 / CNES 8404720 | BLUMENAU, 1101 - JARDIM AMAZONIA | 66 35666767 | Somente app | https://play.google.com/store/apps/details?id=br.com.febrafar.edelivery.ultrapopular https://www.drogariasultrapopular.com.br/ |
| ph-10 | DROGARIA ULTRAPOPULAR AV TANCREDO NEVES | 10341045001821 / CNES 8408769 | TANCREDO NEVES, 1439 - BELA VISTA | 66 35664747 | Somente app | https://play.google.com/store/apps/details?id=br.com.febrafar.edelivery.ultrapopular https://www.drogariasultrapopular.com.br/ |
| ph-11 | DROGARIA ULTRAPOPULAR BRESCANSIN LOJA 02 | 10341045001740 / CNES 8529728 | NATALINO JOAO BRESCANSIN, 1075 - CENTRO SUL | 663544-7757 | Somente app | https://play.google.com/store/apps/details?id=br.com.febrafar.edelivery.ultrapopular https://www.drogariasultrapopular.com.br/ |
| ph-02 | DROGARIAS FARMELHOR | 31216969000128 / CNES 9689338 | TANCREDO NEVES, 2302 - CENTRO | 66 35450026 | Somente app | https://play.google.com/store/apps/details?id=br.com.febrafar.edelivery.farmelhor https://unidades.franquiafarmelhor.com.br/farmelhor-centro-sorriso-mt-73568 |
| ph-39 | Drogarias FarMelhor | dup. ph-02 | Av. Tancredo Neves, 2302 | (66) 3544-0303 | Somente app | https://play.google.com/store/apps/details?id=br.com.febrafar.edelivery.farmelhor https://unidades.franquiafarmelhor.com.br/farmelhor-centro-sorriso-mt-73568 |
| ph-25 | FARMACIA FARMA | 26661675000175 / CNES 9396675 | NATALINO JOAO BRESCANSIN, 451 - CENTRO | 66 35449275 | Somente app | https://play.google.com/store/apps/details?id=br.com.febrafar.edelivery.farmaefarma https://www.farmaefarma.com.br/ |
| ph-58 | Farmácia Ultra Popular Loja 01 | dup. ph-10 | Av. Tancredo Neves, 1439 Esquina com a | (66) 3544-1507 | Somente app | https://play.google.com/store/apps/details?id=br.com.febrafar.edelivery.ultrapopular https://www.drogariasultrapopular.com.br/ |
| ph-59 | Farmácia Ultra Popular Loja 03 | dup. ph-42 | Av. Natalino João Brescansin, 936 | (66) 3544-1555 | Somente app | https://play.google.com/store/apps/details?id=br.com.febrafar.edelivery.ultrapopular https://www.drogariasultrapopular.com.br/ |
| ph-60 | Farmácia Ultra Popular Loja 04 |  | Av. São Francisco de Assis, 862 | (66) 3544-3836 | Somente app | https://play.google.com/store/apps/details?id=br.com.febrafar.edelivery.ultrapopular https://www.drogariasultrapopular.com.br/ |
| ph-61 | Farmácia Ultra Popular Loja 04 |  | MT-242, 3011 | (66) 3544-3001 | Somente app | https://play.google.com/store/apps/details?id=br.com.febrafar.edelivery.ultrapopular https://www.drogariasultrapopular.com.br/ |
| ph-23 | FARMACIA MADEIRA | 09404121000173 / CNES 9395954 | NATALINO JOAO BRESCANSIN, 577 - CENTRO | 66 35444162 | App/vitrine sem vínculo local | https://play.google.com/store/apps/details?id=com.inteliger.madeira.app https://app.farmaciamadeira.com.br/ |
| ph-52 | FARMACIA MADEIRA | dup. ph-23 | Av. Natalino João Brescansin, 577 | (66) 3544-4168 | App/vitrine sem vínculo local | https://play.google.com/store/apps/details?id=com.inteliger.madeira.app https://app.farmaciamadeira.com.br/ |
| ph-14 | BIOMAGISTRAL | 46455133000153 / CNES 4638263 | AVENIDA BLUMENAU, 2234 - JARDIM ALVORADA |  | Sem catálogo público |  |
| ph-24 | DROGARIA AMARAL | 03245248000100 / CNES 9396586 | AVENIDA TANCREDO NEVES, 960 - CENTRO | 66 3544-2112 | Sem catálogo público |  |
| ph-28 | Drogaria Aurora |  | Avenida Belo Horizonte 94 Anexo ao Del Moro Supermercado | (66) 3544-9004 | Sem catálogo público |  |
| ph-29 | Drogaria Biofarma Filial |  | R. Tangará, 770 | (66) 3544-5354 | Sem catálogo público |  |
| ph-30 | Drogaria Bom Jesusltda |  | Av. Natalino João Brescansin, 300 | (66) 3544-1455 | Sem catálogo público |  |
| ph-31 | Drogaria Drogamedic |  | Av. Natalino João Brescansin, 435 | (66) 3544-2868 | Sem catálogo público |  |
| ph-32 | Drogaria Jurema |  | Av. Tancredo Neves, 2123 | (66) 3544-6287 | Sem catálogo público |  |
| ph-33 | Drogaria Nebrasca | 09039622000106 | Av. Natalino João Brescansin, 955 | (66) 3544-5544 | Sem catálogo público | https://drogarianebrasca24horas.com.br/ |
| ph-71 | Drogaria Nebrasca Blumenau | 09039622000289 | Av. Blumenau, 3014 | (66) 3544-5544 | Sem catálogo público |  |
| ph-73 | Drogaria Nebrasca Porto Alegre Sul | 09039622000440 | Av. Porto Alegre Sul, 1321, sala 01 |  | Sem catálogo público |  |
| ph-72 | Drogaria Nebrasca Tangará | 09039622000360 | Rua Tangará, 709 | (66) 3544-1723 | Sem catálogo público |  |
| ph-34 | Drogaria Poupe Mais |  | Av. Brasil, 2204 | (66) 99912-5001 | Sem catálogo público |  |
| ph-35 | Drogaria Sansei Ltda |  | Av. Natalino João Brescansin, 364 | (66) 3544-0579 | Sem catálogo público |  |
| ph-36 | Drogaria São Domingos |  | R. Tangará, 540 | (66) 3544-2004 | Sem catálogo público |  |
| ph-37 | Drogaria São Matheus |  | Rua Passo Fundo | (66) 3544-2550 | Sem catálogo público |  |
| ph-38 | Drogaria Vida e Saúde |  | Av. dos Imigrantes, 1140 | (66) 3544-5850 | Sem catálogo público |  |
| ph-40 | ELIFARMA DROGARIA LTDA |  | Av. Perimental Sudoeste, 2345 | (66) 3544-9585 | Sem catálogo público |  |
| ph-16 | FARMACIA DE MANIPULACAO ARTE FARMA | 39538037000294 / CNES 4874196 | AVENIDA TANCREDO NEVES, 1075 - JARDIM ALVORADA | 66 99647-8696 | Sem catálogo público |  |
| ph-07 | FARMACIA MULTIZ POPULAR | 40721975000476 / CNES 8016976 | NOROESTE, 2319 - JARDIM BELA VISTA | 66 35471484 | Sem catálogo público |  |
| ph-26 | FARMACIA SORRIFARMA | 09589272000143 / CNES 9422366 | AVENIDA TANCREDO NEVES, 1208 - CENTRO |  | Sem catálogo público |  |
| ph-27 | FARMACIAS SUPER PRECO POPULAR 1 | 09632338000130 / CNES 9603697 | AVENIDA NATALINO JOAO BRESCANSIN, 364 - CENTRO | 66 3544-0020 | Sem catálogo público |  |
| ph-13 | FORMULA CERTA MANIPULACAO | 05456662000194 / CNES 4553497 | AVENIDA BLUMENAU, 3165 - CENTRO | 66 3544-0043 | Sem catálogo público |  |
| ph-42 | Farmacia Descontao Popular |  | Av. Natalino João Brescansin, 936 | (66) 3544-1555 | Sem catálogo público |  |
| ph-47 | Farmacia FarmaVerde |  | Sorriso - MT | (66) 99692-3866 | Sem catálogo público |  |
| ph-48 | Farmacia FarmaVerde |  | R. Graciliano Ramos, 323 - Sala 02 | (66) 99673-2575 | Sem catálogo público |  |
| ph-49 | Farmacia Hiper Popular |  | Av. Tancredo Neves, 1208 | (66) 3544-4555 | Sem catálogo público |  |
| ph-53 | Farmacia Mais Saude |  | Rua Caminho do Sol, 1245 | (66) 99937-6315 | Sem catálogo público |  |
| ph-57 | Farmacia Sorriso |  | Rua São Francisco de Assis, 379 | (66) 99689-3844 | Sem catálogo público |  |
| ph-51 | Farmacia karina |  | R. Tangará, 490 | (66) 3544-2592 | Sem catálogo público |  |
| ph-41 | Farmácia Bela Vista |  | Perimetral Noroeste, 2162 | (66) 3544-5062 | Sem catálogo público |  |
| ph-45 | Farmácia Estrela |  | Av. Porto Alegre, 2984 | (66) 3544-4839 | Sem catálogo público |  |
| ph-46 | Farmácia Farma & Farma Popular Sorriso | dup. ph-25 | Av. Natalino João Brescansin, 451 | (66) 99717-1697 | Sem catálogo público |  |
| ph-54 | Farmácia Mais Saúde |  | R. Vale Dourado, nº 1641 | (66) 99905-6874 | Sem catálogo público |  |
| ph-55 | Farmácia Proquifarma |  | Rua S Francisco Assis, 1393 | (66) 3544-5031 | Sem catálogo público |  |
| ph-56 | Farmácia Santa Rita |  | Av. Natalino João Brescansin, 774 | (66) 3544-4539 | Sem catálogo público |  |
| ph-43 | Farmácia do Trabalhador |  | R. Tangará, 554 | (66) 3544-5821 | Sem catálogo público |  |
| ph-44 | Farmácia e Drogaria Bom Jesus | dup. ph-30 | Av. Natalino João Brescansin, 300 | (66) 3544-1455 | Sem catálogo público |  |
| ph-62 | Farmácia – Unidade Sorriso |  | Av. Luiz Amadeu Lodi, 949 | (66) 3544-6579 | Sem catálogo público |  |
| ph-63 | Farmácias Desconto Fácil |  | Perimetral Sudoeste, 1478 | (66) 3544-4262 | Sem catálogo público |  |
| ph-65 | Florallis Fármacia de Manipulação e Drogaria |  | Av. Blumenau Sul, 2265 | (66) 3544-8565 | Sem catálogo público |  |
| ph-50 | Fármacia Hiper Popular Sorriso | dup. ph-49 | Av. Tancredo Neves, 1208 | (66) 99640-8888 | Sem catálogo público |  |
| ph-66 | IDEAL FARMA POPULAR |  | Av. Tancredo Neves, 218 | (66) 98417-8258 | Sem catálogo público |  |
| ph-67 | R Nardi & Cia Ltd |  | Av. Tancredo Neves, 1228 | (66) 3544-1507 | Sem catálogo público |  |
| ph-05 | SUPREMA FARMA NOEMIA TONELLO DALMOLIN | 52950355000226 / CNES 8570582 | NOEMIA TONELLO DALMOLIN, 2285 - PARQUE UNIVERSITARIO | 66 99652626 | Sem catálogo público |  |
| ph-04 | SUPREMA FARMA SORRISO | 52950355000145 / CNES 8560420 | TANCREDO NEVES, 1935 - BELA VISTA | 66 99651812 | Sem catálogo público |  |
| ph-06 | DROGARIA SAO PAULO | 61412110120487 / CNES 5897858 | NATALINO JOAO BRESCANSIN, 1722 - CENTRO NORTE | 11 32747400 | Catálogo público | https://www.drogariasaopaulo.com.br/ |
| ph-64 | Farmácias Pague Menos | 06626253098020 / dup. ph-69 | Av. Natalino João Brescansin, 1314 | (66) 3544-1752 | Catálogo público | https://www.paguemenos.com.br/ |
| ph-17 | PHARMACIA THERAPEUTICA | 01780384000337 / CNES 5424399 | AVENIDA BLUMENAU, 2265 - BELA VISTA |  | Catálogo público | https://www.therapeuticafarmacia.com.br/ |
| ph-18 | PHARMACIA THERAPEUTICA | 01780384000175 / CNES 5424402 | AVENIDA CURITIBA, 2734 - CENTRO | 66 3544-3337 | Catálogo público | https://www.therapeuticafarmacia.com.br/ |
| ph-69 | Pague Menos Brescansin | 06626253098020 | Av. Natalino João Brescansin, 1314 | (66) 3544-1752 | Catálogo público | https://www.paguemenos.com.br/ |
| ph-70 | Pague Menos Tancredo | 06626253140670 | Av. Tancredo Neves, 2198 |  | Catálogo público | https://www.paguemenos.com.br/ |
| ph-68 | THERAPEUTICA PHARMÁCIA | 01780384000337 / dup. ph-17 | Av. Blumenau Sul, 2265 | (66) 3544-8565 | Catálogo público | https://www.therapeuticafarmacia.com.br/ |
| ph-22 | CAF CENTRAL DE ABASTECIMENTO DE FARMACIAS | CNES 6588573 | AVENIDA PERIMETRAL SUDOESTE, 2258 - BELA VISTA | 66 3545-8051 | Programa público |  |
| ph-20 | FARMACIA CIDADA CENTRAL II | CNES 6588557 | AVENIDA PORTO ALEGRE, S/N - CENTRO NORTE | 66 3545-8027 | Programa público |  |
| ph-21 | FARMACIA CIDADA PRIMAVERA III | CNES 6588565 | AVENIDA PERIMETRAL NOROESTE, 1881 - JARDIM PRIMAVERA | 66 39076948 | Programa público |  |
| ph-19 | FARMACIA CIDADA SAO DOMINGOS I | CNES 6588549 | RUA PANAMBI, S/N - INDUSTRIAL | 66 3545-8045 | Programa público |  |
| ph-12 | FARMACIA CIDADA VI ROTA DO SOL | CNES 4181719 | RUA CAMINHO DO SOL, 1208 - ROTA DO SOL | 66 3545-8077 | Programa público |  |
| ph-01 | FARMACIA POPULAR | 07549438000172 / CNES 9689133 | AVENIDA PERIMETRAL NOROESTE, 2110 - BELA VISTA | 66 3544-7771 | Programa público |  |
| ph-15 | DROGASIL DS1357 | 61585865228411 / CNES 4837940 | AVENIDA BLUMENAU, 2995 - CENTRO NORTE | 66 3544-6842 | Estoque público | https://www.drogasil.com.br/ |
| ph-03 | DROGASIL DS2739 | 61585865413405 / CNES 8543763 | TANCREDO NEVES, 2942 - CENTRO NORTE | 11 37695731 | Estoque público | https://www.drogasil.com.br/ |
| ph-08 | RAIA DROGASIL S A | 61585865287345 / CNES 8295190 | NATALINO JOAO BRESCANSIN, 488 - CENTRO SUL | 11 37695678 | Estoque público | https://www.drogasil.com.br/ |

## Limite de cobertura

Os catálogos de app-only não são públicos. A extração de endpoints por descompilação, engenharia reversa ou uso de credenciais internas contraria a regra do projeto (`docs/LEGAL_CONSIDERATIONS.md`) e os termos das plataformas. Para levar essas redes ao ingestor, o caminho correto é solicitar à rede/FEBRAFAR/Farmarcas um feed CSV/JSON ou uma API autorizada, com filial, produto, preço, estoque e validade. O mesmo vale para as farmácias independentes que hoje só atendem por WhatsApp.
