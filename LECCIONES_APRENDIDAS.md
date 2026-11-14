# LECCIONES APRENDIDAS - UrquiOffice (Fases 1-4)

Documento vivo: añadir aquí cada problema real y su solución. NO borrar entradas.

## PRoot / NDK (bloqueo resuelto con CI)
- **PROBLEMA**: compilar aarch64-linux-android localmente falla: el NDK trae binarios x86_64 (clang, ld.lld) que no ejecutan bajo PRoot ARM64 → `posix_spawn failed: No such file or directory`.
- **SOLUCIÓN**: no compilar Android en la tablet. Usar GitHub Actions (ubuntu-24.04) con el NDK real. Workflow: .github/workflows/build.yml.
- **LECCIÓN**: PRoot solo sirve para `cargo test` en host aarch64-linux-gnu, nunca para cross-compile Android.

## cc-rs y el NDK 28
- **PROBLEMA**: `failed to find tool "aarch64-linux-android-ar"` en CI.
- **SOLUCIÓN**: exportar AR_aarch64_linux_android=<NDK>/bin/llvm-ar y RANLIB_aarch64_linux_android=<NDK>/bin/llvm-ranlib. El NDK 28 ya no expone los wrappers con prefijo target.
- CC: CC_aarch64_linux_android="<NDK>/bin/aarch64-linux-android21-clang --sysroot=<NDK sysroot>".
- Linker: CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER=<NDK>/bin/aarch64-linux-android31-clang.

## flutter_rust_bridge 2.13.0
- **PROBLEMA**: `undefined symbol: frb_get_rust_content_hash` al abrir la app.
- **CAUSA 1**: extractNativeLibs=false (default AGP targetSdk 36) → los .so no salen del APK a lib/arm64 (carpeta vacía) y no hay símbolo.
- **SOLUCIÓN 1**: android:extractNativeLibs="true" en AndroidManifest.xml + packaging { jniLibs { useLegacyPackaging = true } } en build.gradle.kts.
- **CAUSA 2**: usar ExternalLibrary.process() → busca símbolos en libflutter.so, no en librust_lib_urquioffice.so.
- **SOLUCIÓN 2**: llamar RustLib.init() SIN argumentos; en Android el loader hace DynamicLibrary.open('librust_lib_urquioffice.so') y el linker lo resuelve al nativeLibraryDir.
- El .so exporta los símbolos frb_* correctamente (verificar con dynsym si duda).
- Codegen: flutter_rust_bridge_codegen 2.13.0 instalado aparte (cargo install --root evita los symlinks .l2s rotos de PRoot).

## Cargo bajo PRoot
- Si falla git fetch: find /home/jcode/.cargo/git -name '.l2s.*' -delete y CARGO_NET_GIT_FETCH_WITH_CLI=true.
- ring NO compila con el NDK → parchear ureq a vendor/ureq-no-tls (stub sin TLS) en [patch.crates-io].
- krilla de crates.io no sirve para craft-pdf → parchear krilla al git de craft-libs rev 6e03f65.
- reqwest en HTTP plano (default-features=false, features json+stream) porque Venagate es 127.0.0.1.

## Despliegue en tablet (adb inalámbrico)
- Dispositivo: 127.0.0.1:5580, TAB-T126-8, Android 15 (SDK 35), arm64-v8a, 24 GB libres.
- `adb install -r` falla con INSTALL_FAILED_UPDATE_INCOMPATIBLE si cambió la firma → hacer `adb uninstall com.example.urquioffice` primero.
- Verificar que arranca: `adb shell am start -n com.example.urquioffice/.MainActivity`, leer `adb logcat -d | grep "E flutter"`, confirmar pid con `pidof`.

## GitHub Actions
- El runner usa /home/runner/work, NO /workspace → nunca rutas absolutas del repo en los scripts.
- sdkmanager: descargar cmdline-tools, mover a cmdline-tools/latest, `yes | sdkmanager --licenses`, instalar platforms;android-36, build-tools;36.0.0, ndk;28.2.13676358.
- Artefacto: actions/upload-artifact@v4 con el APK; bajar con `gh run download <id> -D dir`.
- Flutter 3.47.9 pide NDK 28.2.13676358, compileSdk 36, minSdk 24.

## Reglas del proyecto
- Sin panic!/unwrap()/expect()/todo!/unsafe en Rust de producción (solo Result). Tests con assert!/matches!.
- Commitear y pushear después de CADA sub-paso; actualizar PROGRESS.md.
- Español siempre; términos técnicos en inglés.
- flutter analyze debe quedar en 0 issues antes de cada push de UI.
