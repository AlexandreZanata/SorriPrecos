# Preparar o iOS em um Mac

O código compartilhado e os targets `iosArm64`/`iosSimulatorArm64` estão preparados. **Build e execução iOS não foram validados nesta entrega em Linux.**

Com macOS, Xcode e Java 21, instale XcodeGen pelo gerenciador de sua preferência e execute nesta pasta:

```sh
xcodegen generate
open SorriPrecosIOS.xcodeproj
```

Selecione seu time de assinatura e um aparelho/simulador Apple Silicon. A fase de build executa `embedAndSignAppleFrameworkForXcode`, conforme a [integração oficial](https://kotlinlang.org/docs/multiplatform/multiplatform-direct-integration.html). O framework é estático; não há CocoaPods. O sandbox da fase de build é desabilitado para que o Gradle compile o framework, conforme essa configuração de integração.

Use uma API HTTPS no aparelho. Não habilitamos exceção ATS para HTTP. Um simulador pode exigir uma configuração debug local específica; valide no Mac antes de distribuir. O endereço da API é configurável pela tela Conexão. Favoritos/cache usam NSUserDefaults.
