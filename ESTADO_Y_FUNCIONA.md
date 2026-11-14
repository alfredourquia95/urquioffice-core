# ESTADO ACTUAL Y LO QUE FUNCIONA - UrquiOffice

Última actualización: 2026-11-14, Fase 3 completada (commit ba7467c).

## ✅ Qué funciona y está probado
- **Motor Rust (Fase 1)**: crate urquioffice-core con WordCraft; demo_fase1() crea documento, inserta "Hola UrquiOffice", aplica bold, layout → imprime posiciones. 6/6 tests pasan en host.
- **DB + servidor IA (Fase 2)**: src/db.rs (SQLite bundled, AES-256-GCM para api_keys) y src/ai_server.rs (axum 127.0.0.1:31337, proxy /v1/chat/completions a Venagate 127.0.0.1:31079, rate limit 40 RPM, backoff con Retry-After, watchdog SSE 20s, rotación round-robin de claves). Probado end-to-end con clave nvapi real.
- **Puente FFI (Fase 3)**: rust/ cdylib librust_lib_urquioffice.so con init_app/estado_nucleo/execute_command; bindings generados en app/lib/src/rust/; Flutter llama estadoNucleo() y muestra el estado. **APK instalado y corriendo en la tablet TAB-T126-8 sin crash**.
- **CI/CD**: cada push a main compila el .so + APK en GitHub Actions y publica artefacto descargable (~7 min por corrida).
- **UI mock (Fase 4)**: Ribbon (Inicio/Insertar/Diseño/IA), DocumentCanvas con hoja A4, SidePanelIA con chat mock, AppBar + atajos Ctrl+S/O/B/I/U. flutter analyze 0 issues.

## 📦 Cómo obtener el último APK probado
1. `gh run list --repo alfredourquia95/urquioffice-core --limit 1` → último run exitoso.
2. `gh run download <run-id> --repo alfredourquia95/urquioffice-core -D /tmp/apk`
3. `adb -s 127.0.0.1:5580 uninstall com.example.urquioffice` (si falla la firma)
4. `adb -s 127.0.0.1:5580 install /tmp/apk/urquioffice-app-debug-arm64/app-debug.apk`
5. `adb -s 127.0.0.1:5580 shell am start -n com.example.urquioffice/.MainActivity`

## 🗂️ Estructura del repo
- `src/` (crate raíz urquioffice-core): lib.rs (motor), db.rs, ai_server.rs
- `rust/` (FFI): src/api.rs, src/frb_generated.rs, Cargo.toml con crate-type cdylib
- `app/` (Flutter): lib/main.dart, lib/widgets/{ribbon,document_canvas,side_panel_ia}.dart, lib/src/rust/ (bindings)
- `vendor/ureq-no-tls/`: stub de ureq sin ring (NDK no compila ring)
- `.github/workflows/build.yml`: pipeline completo
- `URQUIOFFICE_MASTER_BLUEPRINT.md`: blueprint maestro (7 fases)
- `PLAN_FASES_5-7.md`: plan de continuación
- `LECCIONES_APRENDIDAS.md`: problemas reales y soluciones
- `PROGRESS.md`: bitácora por fase

## ⚠️ Qué hay que evitar (resumen)
- NO compilar Android localmente (PRoot no ejecuta binarios x86_64 del NDK). Siempre CI.
- NO usar ExternalLibrary.process() en Android; usar RustLib.init() por defecto.
- NO quitar extractNativeLibs=true / useLegacyPackaging.
- NO usar ring/TLS en el lado Android; ureq está parcheado sin TLS.
- NO mezclar fases; NO panic!/unwrap() en Rust; commit+push tras cada sub-paso.
