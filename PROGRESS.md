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
