# AEGIS — SDK Java (zokastech.fr)

Modules Maven (`fr.zokastech.aegis`) :

| Artefact | Rôle |
|----------|------|
| **aegis-sdk** | API `AegisEngine`, JNI vers `libaegis_jni` |
| **aegis-spring-boot-starter** | Auto-configuration Spring Boot 3 (`aegis.*`) |

## Bibliothèque native

```bash
cd sdk-java/aegis-jni-native
cargo build --release
```

Copier la bibliothèque produite (`target/release/libaegis_jni.so`, `libaegis_jni.dylib`, ou `aegis_jni.dll`) dans un répertoire du `java.library.path`, ou lancer la JVM avec :

```text
-Daegis.jni.library.path=/chemin/absolu/vers/libaegis_jni.dylib
```

## Build Maven

```bash
cd sdk-java
mvn -q -pl aegis-sdk,aegis-spring-boot-starter -am install
```

## Publication Maven Central

Le parent définit `distributionManagement` (OSSRH). Configurez les identifiants `ossrh` dans `~/.m2/settings.xml`, la signature GPG et le plugin de staging Sonatype selon la [documentation officielle](https://central.sonatype.org/publish/publish-maven/).

Coordonnées à publier : **`fr.zokastech.aegis:aegis-sdk`** (et le starter si besoin).

## Exemples

- `examples/spring-boot-service` — `POST /api/scan` avec `{"text":"..."}`
- `examples/spark-udf-demo` — UDF `aegis_mask_email` (mode `local[*]`)

Licence : Apache-2.0 **ou** MIT.
