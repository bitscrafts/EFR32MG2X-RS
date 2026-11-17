# android — cliente BLE do dosímetro

App mínima: procura o dispositivo, liga-se, subscreve três características e mostra
**nível, intervalo e dose**. Não é um produto — é o cliente que prova que o contrato GATT
funciona e que os números chegam ao telemóvel.

**Já existe:** `app/src/main/java/br/ufc/dosimeter/MainActivity.kt`

**Falta criar** (conteúdo abaixo, pronto a copiar): manifesto, recursos, gradle.

## Contrato GATT (o mesmo que `spec/E1-APP.DOSIMETER.md`, tarefa E1-BLE.1)

```
serviço     D05E0001-0000-4000-8000-00805F9B34FB
nivel       D05E0002-…   f32 LE, dB(A)          notify 1 Hz
intervalo   D05E0003-…   2 x f32 LE (min,max)   notify 1 Hz
dose        D05E0004-…   f32 LE, %              notify 0,1 Hz
comando     D05E0005-…   write 0x01 = repor dose
```

## `app/src/main/AndroidManifest.xml`

```xml
<?xml version="1.0" encoding="utf-8"?>
<manifest xmlns:android="http://schemas.android.com/apk/res/android">

    <uses-feature android:name="android.hardware.bluetooth_le" android:required="true" />

    <uses-permission android:name="android.permission.BLUETOOTH_SCAN"
        android:usesPermissionFlags="neverForLocation" />
    <uses-permission android:name="android.permission.BLUETOOTH_CONNECT" />
    <!-- Android <= 11 exige localizacao para varrer BLE -->
    <uses-permission android:name="android.permission.ACCESS_FINE_LOCATION"
        android:maxSdkVersion="30" />

    <application
        android:label="Dosímetro"
        android:theme="@android:style/Theme.Material.Light">
        <activity android:name=".MainActivity" android:exported="true">
            <intent-filter>
                <action android:name="android.intent.action.MAIN" />
                <category android:name="android.intent.category.LAUNCHER" />
            </intent-filter>
        </activity>
    </application>
</manifest>
```

## `app/src/main/res/values/strings.xml`

```xml
<?xml version="1.0" encoding="utf-8"?>
<resources>
    <string name="a_procurar">à procura do dosímetro…</string>
    <string name="a_ligar">a ligar a %1$s…</string>
    <string name="ligado">ligado — à espera de notificações</string>
    <string name="desligado">desligado</string>
    <string name="servico_ausente">serviço do dosímetro não encontrado</string>
    <string name="sem_permissoes">sem permissões de Bluetooth</string>
    <string name="sem_bluetooth">este aparelho não tem Bluetooth</string>
    <string name="ligue_bluetooth">ligue o Bluetooth</string>
    <string name="falha_procura">falha na procura: %1$d</string>
    <string name="painel">Nível: %1$s\nDose:  %2$s</string>
</resources>
```

## `settings.gradle.kts`

```kotlin
pluginManagement {
    repositories { google(); mavenCentral(); gradlePluginPortal() }
}
dependencyResolutionManagement {
    repositories { google(); mavenCentral() }
}
rootProject.name = "dosimeter"
include(":app")
```

## `build.gradle.kts` (raiz)

```kotlin
plugins {
    id("com.android.application") version "8.5.2" apply false
    id("org.jetbrains.kotlin.android") version "1.9.24" apply false
}
```

## `app/build.gradle.kts`

```kotlin
plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}

android {
    namespace = "br.ufc.dosimeter"
    compileSdk = 34

    defaultConfig {
        applicationId = "br.ufc.dosimeter"
        minSdk = 26          // BLE moderno + permissões por runtime
        targetSdk = 34
        versionCode = 1
        versionName = "0.1"
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    kotlinOptions { jvmTarget = "17" }
}

dependencies {
    implementation("androidx.activity:activity-ktx:1.9.0")
    implementation("androidx.core:core-ktx:1.13.1")
}
```

## `gradle.properties`

```properties
org.gradle.jvmargs=-Xmx2g
android.useAndroidX=true
```

## Como compilar no lnx, por container

```bash
ssh mvcorrea@lnx
podman pull docker.io/mingc/android-build-box:latest      # ~4-5 GB, uma vez
cd /podman/assistants/hermes/projects/efr32mg24-rs/crates/dosimeter-mg24/android
podman run --rm -v "$PWD":/project -w /project \
  docker.io/mingc/android-build-box:latest \
  bash -lc './gradlew assembleDebug || gradle assembleDebug'
```

O APK sai em `app/build/outputs/apk/debug/app-debug.apk`. Instalar no telemóvel com
`adb install` (o container não vê o USB do telemóvel; o APK copia-se e instala-se por
ficheiro, ou usa-se `adb` no Mac).

## Nota de estratégia

A app **não dá nota** e exige o telemóvel para depurar. A demo da apresentação faz-se com o
**nRF Connect** (grátis, mostra as mesmas notificações) e com os gráficos gerados no host.
Esta app existe para quem quiser ver o resultado como produto — e porque o contrato GATT
estar definido torna o cliente substituível.
