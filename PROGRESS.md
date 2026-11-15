# 📊 UrquiOffice: Registro de Progreso (PROGRESS.md)

> Última actualización: 2026-11-14 02:18 UTC | Rama: `main`
> Regla de oro: tras cada sub-fase completada → actualizar este archivo + `git commit` (+ push si hay remoto).
> Fuente de verdad: `URQUIOFFICE_MASTER_BLUEPRINT.md` (v1.0)

## Estado global

| Fase | Estado | Último commit | Notas |
| :--- | :--- | :--- | :--- |
| 1. Núcleo WordCraft (Rust puro) | ✅ COMPLETADA | `d53e0de` | Test imprime lista de dibujo (72.0, 83.1) bold=true |
| 2. SQLite + Servidor IA local | ✅ COMPLETADA | (pendiente commit) | 6/6 tests OK incl. end-to-end real contra Venagate |
| 3. Puente FFI Flutter-Rust | ⏳ Pendiente | — | |
| 4. UI visual Flutter (mockups) | ⏳ Pendiente | — | |
| 5. Integración comandos (422) | ⏳ Pendiente | — | |
| 6. Agente de IA en UI | ⏳ Pendiente | — | |
| 7. Tipografías + APK release | ⏳ Pendiente | — | |

## Historial de avances

### 2026-11-14 02:18 UTC — FASE 2 COMPLETADA ✅
- `src/db.rs`: SQLite (rusqlite bundled) con `settings`, `api_keys` (AES-256-GCM, clave derivada con SHA-256), `chat_history`, `recent_files`. Tests: esquema, cifrado roundtrip, clave incorrecta falla, rotación (desactivar).
- `src/ai_server.rs`: axum en `127.0.0.1:31337` con `POST /v1/chat/completions` + `GET /v1/models` (proxy a Venagate `127.0.0.1:31079`). Resiliencia: token bucket 40 RPM, backoff exponencial en 429/5xx (lee `Retry-After`), watchdog 20s por chunk SSE, rotación round-robin de claves con desactivación en DB.
- reqwest con `rustls-tls` (sin openssl, no disponible en PRoot). rand_core 0.9 con `os_rng` (OsRng usa `TryRngCore::try_fill_bytes`).
- **Test end-to-end OK**: clave nvapi real guardada cifrada en SQLite → `llamar_venagate` → Venagate responde 200 con `nvidia/mistralai/mistral-large-4-1t-a52b`. Modelo por defecto: `nvidia/mistralai/mistral-large-4-1t-a52b`. Claves en `~/.config/openvidia/keys.json` (7 nvapi).
- `cargo test`: 6/6 pasan. Sin unwrap/expect/panic/unsafe en código.
- **Siguiente paso:** Fase 3.1 → `flutter create .` + flutter_rust_bridge + `init_app()` con `#[flutter_rust_bridge::frb(init)]`.

### 2026-11-14 01:36 UTC — FASE 1 COMPLETADA ✅
- Crate `urquioffice-core` inicializado (`cargo init --lib`).
- Dependencias WordCraft añadidas (geom, doc, fonts, layout, engine, docx, formats) + craft-proof.
- **Fix PRoot aplicado:** los enlaces `.l2s.*` de la caché de cargo rompían libgit2. Solución: limpiarlos y añadir `[patch.crates-io] krilla = { git craft-libs rev 6e03f65 }` porque el krilla 0.8.2 de crates.io no tiene los campos que craft-pdf espera (el vendored sí).
- `cargo check` pasa sin errores. `cargo test` pasa: imprime `(72.0, 83.1) bold=true "Hola UrquiOffice"`.
- Flujo probado: `Document::new()` → `text.insert` → `select.all` → `format.bold` → `session.layout()` → lista de dibujo con X/Y.
- Regla de oro cumplida: sin `panic!`/`unwrap()`/`expect()`/`todo!`/`unsafe` en código (solo `Result`).
- **Siguiente paso:** Fase 2.1 → rusqlite (bundled) con tablas settings, api_keys, chat_history, recent_files.

### 2026-11-13 23:10 UTC — Inicio del proyecto
- Commit `a9f97cf`: Blueprint v1.0 guardado como fuente de verdad.
- Plan de 7 fases cargado en el sistema de tareas.

## Reglas activas (del blueprint)

1. Prohibido `panic!`, `unwrap()`, `expect()`, `todo!()`, `unsafe` en Rust. Solo `Result<T, E>`.
2. No mezclar fases: nada de Flutter hasta que la Fase 1 esté 100% probada en consola. ✅ Fase 1 OK.
3. Si cargo falla por PRoot: limpiar `*.l2s.*` de `/home/jcode/.cargo/git` (ver historial arriba).
4. Tras cada sub-fase: actualizar este archivo, `git commit` y (si hay red) `git push`.
5. `git-fetch-with-cli = true` añadido a `/home/jcode/.cargo/config.toml` para evitar el bug de libgit2 en PRoot.

## Fase 3 (parcial) - CI/CD en GitHub Actions (commit 0b72a98)
- Build local aarch64-linux-android bloqueado por PRoot (NDK x86_64 no ejecuta en ARM64).
- `.github/workflows/build.yml`: ubuntu-24.04 + Flutter 3.47.9 + NDK 28.2.13676358 + Rust stable.
- Compila `librust_lib_urquioffice.so`, lo mete en `jniLibs/arm64-v8a`, corre `flutter build apk --debug`, verifica el .so dentro del APK y sube artefacto `urquioffice-app-debug-arm64`.
- Push a main dispara el workflow automaticamente.

## Fase 4 - UI Flutter mockups (en progreso)
- `app/lib/main.dart`: AppBar con titulo + botones abrir/guardar/panel IA, CallbackShortcuts (Ctrl+S/O/B/I/U), barra de estado con estado del nucleo Rust.
- `app/lib/widgets/ribbon.dart`: pestanas Inicio/Insertar/Diseno/IA con MouseRegion hover y grupos de botones (Fuente, Parrafo, Portapapeles, etc.).
- `app/lib/widgets/document_canvas.dart`: CustomPainter dibuja hoja A4 blanca con sombra + texto estatico de prueba.
- `app/lib/widgets/side_panel_ia.dart`: panel derecho 300px con chat mock, campo de texto y boton enviar (sin logica real).
- `flutter analyze`: 0 issues.

## Fase 3 CI - fix #2 (commit pusheado 14:52Z)
- Fallo CI: cc-rs no encontraba "aarch64-linux-android-ar" (NDK 28 solo trae llvm-ar).
- Fix: AR_aarch64_linux_android=llvm-ar y RANLIB=llvm-ranlib en build.yml.
- Fix FFI: main.dart usa ExternalLibrary.process(iKnowHowToUseIt: true) para cargar librust_lib_urquioffice.so desde jniLibs/arm64-v8a del APK.
- Verificaciones swarm: Fase 4 UI cumple blueprint 4.1-4.4 (analyze 0 issues); FFI OK (6/6 tests, sin panic/unsafe); tablet TAB-T126-8 Android 15 SDK35 arm64-v8a con 24G libres y app ya instalada (versionCode 1).

## Fase 3 CI - fix #3 (15:20Z) - APK YA EN LA TABLET, un detalle pendiente
- Corridas 43607113579 y 43607176838: SUCCESS. APK debug (239MB) descargado de artefactos y contiene lib/arm64-v8a/librust_lib_urquioffice.so (88MB, símbolos frb exportados OK).
- Instalado en TAB-T126-8 (Android 15, arm64): uninstall firma vieja + install OK.
- PERO la app crashea al abrir: "Failed to lookup symbol 'frb_get_rust_content_hash' ... by process()".
- Diagnóstico: el APK se generó con extractNativeLibs=false (AGP default targetSdk 36), así que Android NO extrajo los .so a /data/app/.../lib/arm64 (vacío) y DynamicLibrary.process() no los encuentra.
- Fix pusheado: android:extractNativeLibs="true" en el manifest + packaging.jniLibs.useLegacyPackaging=true en build.gradle.kts. Nueva corrida 43609681589 in_progress.

## Fase 3 CI - fix #4 (15:45Z) - loader de frb corregido
- APK con extractNativeLibs=true instala y extrae bien el .so (lib/arm64/librust_lib_urquioffice.so presente), pero ExternalLibrary.process() busca frb_get_rust_content_hash en libflutter.so → undefined symbol.
- Fix: volver a RustLib.init() sin argumentos: en Android frb hace DynamicLibrary.open('librust_lib_urquioffice.so') que el linker resuelve al nativeLibraryDir.
- flutter analyze: 0 issues. Push disparó nueva corrida.

## FASE 3 COMPLETADA (16:00Z) - APK con puente Rust funcionando en la tablet
- Corrida 43611872718 SUCCESS. APK instalado en TAB-T126-8 (adb 127.0.0.1:5580).
- La app ARRANCA SIN CRASH: proceso vivo (pid 21780), MainActivity en primer plano, 0 errores de Flutter en logcat.
- El puente flutter_rust_bridge carga librust_lib_urquioffice.so vía DynamicLibrary.open y estadoNucleo() responde.
- Fixes aplicados en CI: AR/RANLIB=llvm-ar/llvm-ranlib del NDK, extractNativeLibs=true + useLegacyPackaging, RustLib.init() por defecto.

## Documentos de conocimiento creados (16:55Z)
- PLAN_FASES_5-7.md: plan detallado de Fases 5 (integración comandos), 6 (agente IA) y 7 (fuentes + release).
- LECCIONES_APRENDIDAS.md: problemas reales y soluciones (PRoot/NDK, cc-rs llvm-ar, extractNativeLibs, ExternalLibrary.process, parches ureq/krilla).
- ESTADO_Y_FUNCIONA.md: qué funciona hoy, estructura del repo y comandos de despliegue.
- AGENTS.md global en /root/.jcode/AGENTS.md + memoria persistente guardada.

## FASE 4 COMPLETADA (17:01Z)
- UI mock verificada: Ribbon (Inicio/Insertar/Diseno/IA) con hover, DocumentCanvas A4 con CustomPainter, SidePanelIA con chat mock, AppBar + atajos Ctrl+S/O/B/I/U, barra de estado con estado del nucleo Rust.
- Criterio de éxito cumplido: se ve como editor moderno, responde a ratón/teclado, no edita nada real. flutter analyze 0 issues.
- Inicio Fase 5: integración de comandos reales Rust<->Flutter.

## Fase 5 - 5a completado (commit d979518)
- Núcleo: sesión global `Mutex<Option<Session>>`; `doc_nuevo()`, `ejecutar_comando(id, args JSON)`, `obtener_dibujo()`; 7/7 tests pasan (incluye fase5_sesion_global_comandos_y_dibujo).
- FFI: `execute_command` real + nueva `obtener_dibujo() -> JSON [{x,y,text,bold}]`; bindings regenerados (obtenerDibujo en Dart).
- Sub-paso 5b en curso: DocumentCanvas editable en Flutter que envía text.insert y repinta con obtener_dibujo().

## Fase 5 - 5b/5c completados (20:11Z)
- DocumentCanvas editable: cada tecla envía text.insert/edit.backspace vía execute_command; repinta con obtener_dibujo() (JSON [{x,y,text,bold}]) usando CustomPainter+TextPainter con escala A4.
- Ribbon cableado a comandos reales: format.bold/italic/underline, edit.undo/redo, align.left/center/right. Guardar/abrir pendientes de SAF (5.4).
- flutter analyze 0 issues. Corrida CI 43636153182 en cola (compila .so + APK con el nuevo núcleo).

## Fase 5 - verificación en tablet (23:09Z) - HALLAZGO IMPORTANTE
- APK Fase 5 (corrida 43636210047) instalado y arranca sin crash: "Nucleo Rust OK", UI accesible vía uiautomator, proceso vivo, 0 errores Flutter.
- PERO la pantalla física queda negra aunque uiautomator ve la UI: SurfaceFlinger muestra la capa visible con color{0,0,0,1} y fps=0.08; screencap sale 100% negro incluso en la app de Ajustes del sistema → NO es bug de UrquiOffice, es el renderizado gráfico de esta tablet bajo adb/PRoot (el framebuffer real no recibe frames).
- Conclusión: la lógica Rust↔Flutter funciona (estado OK, sin undefined symbol); la verificación visual en pantalla física queda pendiente de probar la tablet sin adb (tocarla directo).
- Sub-paso 5.4 pendiente: SAF para file.save/file.open .docx.

## Fase 5.4 COMPLETADA (00:43Z) - guardar/abrir .docx vía SAF
- Rust: file_guardar (wordcraft_docx::write directo, preserva negrita) + file_abrir (docx::read + set_document + relayout). Test fase54 con Session local aislada: guardar .docx con bold y reabrir → dibujo trae texto bold. 8/8 tests OK.
- Hallazgos clave: (1) format.bold es toggle, usar {"value":true} para forzar; (2) file.save del engine puede escribir b=0 si el estado quedó par → guardamos directo con docx::write; (3) file.open abre "al lado" si la sesión ya está usada → usamos set_document directo; (4) la sesión global compartida entre tests paralelos causaba condiciones de carrera → tests de docx usan Session local.
- FFI: fileGuardar/fileAbrir. Flutter: file_picker + SAF, botones Guardar/Abrir cableados. analyze 0 issues. Corrida CI disparada.
- FASE 5 COMPLETA: criterio de éxito (escribir + negrita + guardar + reabrir = formato persiste) probado en tests Rust.

## 2026-11-15 — Fase 6.1 (inicio): servidor IA arranca dentro de la app
- `arrancar_servidor_ia(ruta_db, secreto)` en el núcleo: hilo propio con runtime
  tokio current_thread, `Database::abrir` + `ai_server::arrancar` (127.0.0.1:31337),
  idempotente vía `OnceLock<Mutex<bool>>`. Sin panic/unwrap (solo Result + log).
- FFI nueva `arrancar_servidor_ia` expuesta a Dart (funcId 7, bindings Dart+Rust
  añadidos a mano porque el codegen no corre en local por PRoot; el CI lo
  regenera con `flutter_rust_bridge_codegen generate` y debe coincidir).
- `init_app` ahora también arranca el servidor IA con ruta por defecto bajo
  /data/data; Dart puede relanzarlo con la ruta real vía `arrancarServidorIa`.
- Test nuevo: idempotencia de arranque (9/9 tests pasan). `cargo check` y
  `flutter analyze` en 0 issues.

## 2026-11-15 — Fase 6.2: chat IA real con streaming SSE en la UI
- `ClienteIA` (Dart, paquete http): POST /v1/chat/completions con stream:true
  a 127.0.0.1:31337, parseo SSE linea a linea (data: {"choices":[{"delta"...}]}).
- `SidePanelIA` reescrito: historial system+user+assistant, burbuja "..." mientras
  transmite, repintado por fragmento (streaming visible), bloquea input mientras espera.
- main.dart: `_arrancarServidorIA()` con ruta real de la DB vía path_provider
  (getApplicationDocumentsDirectory/urquioffice.db) vía FFI arrancarServidorIa.
- AndroidManifest: permiso INTERNET (loopback 31337).
- flutter analyze: 0 issues. Pendiente: tool calling (fase 6.3).

## 2026-11-15 — Fase 6 completa (6.1–6.4): agente IA en la UI
- 6.1: servidor axum arranca dentro de la app (hilo + runtime tokio propio,
  idempotente) vía FFI `arrancarServidorIa` con ruta real de la DB.
- 6.2: chat real con streaming SSE (`ClienteIA` + SidePanelIA reescrito,
  historial, burbuja "..." mientras transmite). Permiso INTERNET añadido.
- 6.3: tool calling — `tool_calling.rs` (parser de bloques {"tool","args"}
  balanceados, 4 tests) + FFI `extraerToolCalls`/`ejecutarToolCall`; el panel
  ejecuta los comandos vía `execute_command` y repinta el canvas (onRepintar).
- 6.4: botón "Revisar" (ai.review) → `revisarOrtografia` vía craft-proof
  (español si está embebido, si no inglés); CI ahora embebe `es` con
  CRAFT_PROOF_EMBED="en,es,...". Resultado en la barra de estado.
- 14/14 tests Rust pasan; cargo check (crate frb) OK; flutter analyze 0 issues.

## 2026-11-15 — Fase 6 verificada en tablet TAB-T126-8 (APK 43663884032)
- CI en verde (fix de indentación YAML en CRAFT_PROOF_EMBED). APK instalado OK.
- App arranca sin crash; UI completa visible vía uiautomator (ribbon, panel IA con
  mensajes iniciales, botón Enviar).
- Servidor IA local CONFIRMADO: puerto 31337 en escucha dentro del proceso
  (0100007F:7A69 en /proc/net/tcp) y GET /v1/models responde 200 con la lista
  de modelos de Venagate. El arranque vía FFI con la ruta real de la DB funciona.
- Mejora menor: barra de estado con Expanded+semanticsLabel para depurar el estado.

## 2026-11-15 — Fase 7.1–7.3 (inicio): fuentes Liberation en la app
- 7.1/7.2: 8 TTF de Liberation Sans/Serif (2.1.5) en app/assets/fonts + pubspec.
- 7.3: `cargar_fuentes(rutas)` en Rust vía `FontDb::global().install_font`
  (idempotente, sin reemplazar bundled); FFI `cargarFuentes` (funcId 11, codec
  Vec<String> añadido a mano en ambos lados). Dart copia los assets a files/
  y los instala al arrancar. Test de error con ruta inválida pasa (15 tests).
- flutter analyze 0 issues. Pendiente: 7.4 build release en CI.

## 2026-11-15 — Fase 7.4: build release en CI
- build.yml ahora compila también `flutter build apk --release` y publica el
  artefacto `urquioffice-app-release-arm64` (verifica que trae el .so Rust).
- Criterio de Fase 7 pendiente de verificación en tablet con el APK release:
  fuentes Liberation cargadas (estado muestra "fuentes: OK: 8...") y app sin internet salvo IA.

## 2026-11-15 — Fase 7 verificada en tablet (APK release 43667431412, 54 MB vs 116 MB debug)
- APK release instala y arranca sin crash de la app (el FATAL en logcat es del
  proceso adb-relay de Jcode, no nuestro). Servidor IA sigue en 31337 (en écoute).
- UI completa vía uiautomator; panel IA con mensajes y botón Enviar presentes.
- 7.2 extra: fuentes declaradas también como `fonts:` en pubspec (familia
  "Liberation Sans"/"Liberation Serif") y el canvas usa fontFamily: 'Liberation Sans'.
- flutter analyze 0 issues; 15/15 tests Rust.

## 2026-11-15 — Fase 6.5: configurar clave de IA desde la UI
- `guardar_api_key(ruta_db, secreto, nombre, clave)` en el núcleo (DB cifrada
  AES-256-GCM) + FFI `guardarApiKey` (funcId 12, devuelve String para evitar
  codecs i64). Botón "Clave IA" en la pestaña IA abre diálogo, guarda la clave.
- Verificado en tablet: el servidor responde "no hay claves de API activas" si
  no hay ninguna; con la clave guardada el chat debe responder de verdad.
- flutter analyze 0 issues; 15/15 tests Rust.

## 2026-11-15 — Fix SAF saveFile (file_picker exige bytes en Android)
- En tablet, `FilePicker.platform.saveFile` sin `bytes` lanza "Bytes are required
  on Android & iOS". Fix: pasar `bytes: utf8.encode(' ')`; el contenido real lo
  escribe Rust vía file_guardar. flutter analyze 0 issues.

## 2026-11-15 — Verificación tablet APK 43671852100: NÚCLEO OK COMPLETO
- Estado en barra: "documento: OK | IA: OK: servidor IA ya corriendo en
  127.0.0.1:31337 | motor WordCraft OK ... | fuentes: OK: 8 fuente(s) instaladas".
- Puerto 31337 en escucha (0100007F:7A69). Botón "Clave IA" presente en pestaña IA.
- Fix menor: estadoNucleo ahora se considera OK si contiene 'motor WordCraft OK'
  (antes esperaba prefijo 'OK' y marcaba 'Error desde Rust' aunque todo estuviera bien).

## 2026-11-15 — System prompt con tool calling para el critère de Fase 6
- El system prompt del chat ahora instruye al modelo a emitir bloques
  {"tool","args"} con los comandos del engine (select.all, format.bold, etc.),
  con ejemplo concreto. El flujo ya está cableado: extraerToolCalls →
  ejecutarToolCall → onRepintar → canvas recarga obtenerDibujo().
- flutter analyze 0 issues.
