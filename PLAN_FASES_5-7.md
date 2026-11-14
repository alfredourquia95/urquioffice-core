# PLAN DE CONTINUACION - Fases 5, 6 y 7 de UrquiOffice

Estado: Fases 1-4 COMPLETADAS y pusheadas. Fase 3 probada en tablet (APK arranca sin crash, puente Rust OK).
Blueprint: URQUIOFFICE_MASTER_BLUEPRINT.md (fuente de la verdad).

## Infraestructura que YA funciona (no tocar salvo que falle)
- CI/CD: .github/workflows/build.yml compila en ubuntu-24.04: Flutter 3.47.9 + NDK 28.2.13676358 + Rust stable.
  - Env clave del paso Rust: CC_aarch64_linux_android=<NDK>/aarch64-linux-android21-clang --sysroot=<sysroot>, AR_aarch64_linux_android=<NDK>/llvm-ar, RANLIB_aarch64_linux_android=<NDK>/llvm-ranlib, CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER=<NDK>/aarch64-linux-android31-clang, CARGO_NET_GIT_FETCH_WITH_CLI=true.
  - El .so se copia a app/android/app/src/main/jniLibs/arm64-v8a/ y el APK se publica como artefacto `urquioffice-app-debug-arm64`.
- Android: build.gradle.kts tiene abiFilters arm64-v8a, jniLibs.srcDir ../../rust/target/aarch64-linux-android/debug, packaging.jniLibs.useLegacyPackaging=true; AndroidManifest tiene android:extractNativeLibs="true" (OBLIGATORIO, si no el .so no se extrae y DynamicLibrary falla).
- Dart: main.dart usa RustLib.init() SIN argumentos (el loader por defecto hace DynamicLibrary.open('librust_lib_urquioffice.so')). NO usar ExternalLibrary.process().
- Puente FFI: rust/src/api.rs expone init_app(), estado_nucleo() y execute_command(command_name, args) -> Result<String,String>. Regla dura: sin panic!/unwrap()/expect()/todo!/unsafe en produccion.
- Despliegue: adb inalámbrico a 127.0.0.1:5580 (TAB-T126-8, Android 15, arm64-v8a). Comando: adb -s 127.0.0.1:5580 install <apk>. Si falla INSTALL_FAILED_UPDATE_INCOMPATIBLE, hacer uninstall primero.
- Descarga de artefacto: gh run download <run-id> --repo alfredourquia95/urquioffice-core -D /tmp/apk

## FASE 5 - La Integración (comandos reales)
Meta: escribir texto, aplicar negrita, guardar/abrir .docx y que el formato persista.
Pasos:
1. 5.1 execute_command ya existe en api.rs; falta despacharlo de verdad al motor WordCraft (Document/Session) con un estado global del documento en Rust (Mutex<Document>).
2. 5.2 En Flutter, el botón Negrita llama execute_command("format.bold",""); el DocumentCanvas debe recibir la lista de dibujo (JSON) y repintarse con CustomPainter.
3. 5.3 Comandos prioritarios: text.insert, file.save, file.open, edit.undo, edit.redo, format.italic, format.underline, align.left/center/right.
4. 5.4 SAF en Android: usar file_picker (o flutter's SAF) para obtener content:// URI, convertir a ruta real y pasarla a Rust para file.save/file.open .docx.
5. Criterio de éxito: texto + negrita + guardar + cerrar + reabrir = formato se mantiene.

Orden sugerido de implementación:
- 5a. Estado global en Rust: static DOC: Mutex<Option<Document>> + Session; execute_command despacha por nombre.
- 5b. Nueva función FFI: obtener_dibujo() -> Result<String,String> que devuelve JSON con las posiciones/glyphs del layout para que Flutter pinte.
- 5c. En Flutter: DocumentCanvas editable (TextField/EditableText invisible o CustomPainter con caret) que envía text.insert por cada tecla; botón Negrita envía format.bold; repintado al recibir obtener_dibujo().
- 5d. file.save/file.open con SAF.

## FASE 6 - Agente de IA en la UI
Meta: "pon el primer párrafo en negrita" y que el canvas cambie.
Pasos:
1. 6.1 SidePanelIA llama a http://127.0.0.1:31337/v1/chat/completions (servidor axum de Fase 2 ya corriendo en Rust; arrancarlo en init_app en un hilo aparte).
2. 6.2 Streaming SSE: Rust reenvía chunks; en Flutter usar http package con stream o frb stream (RustStreamSink) para pintar en tiempo real.
3. 6.3 Tool calling: si la respuesta trae {"tool":"format.bold","args":{...}}, Rust lo ejecuta vía execute_command y luego pide obtener_dibujo() para refrescar el canvas.
4. 6.4 Botón "Corregir ortografía": envía texto, recibe corregido, reemplaza selección.

## FASE 7 - Tipografías y release
1. 7.1 Descargar Liberation Sans / Liberation Serif (métricamente compatibles con Arial/Times).
2. 7.2 assets/fonts/ + declarar en pubspec.yaml.
3. 7.3 Pasar la ruta de assets a Rust (CRAFT_I18N_DIR / font dir) en init_app vía execute_command("config.fonts_dir", path).
4. 7.4 flutter build apk --release en CI (añadir paso release al workflow), instalar y probar sin internet (salvo IA).

## Reglas de trabajo (invariables)
- Español para hablar; commit + push tras cada sub-paso; actualizar PROGRESS.md.
- Builds locales con -j 2 (8 GB RAM). En PRoot: borrar .l2s.* de ~/.cargo/git si falla git fetch; CARGO_NET_GIT_FETCH_WITH_CLI=true.
- No mezclar fases. Cada fase termina con su criterio de éxito verificado en la tablet real.
