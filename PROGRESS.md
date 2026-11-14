# 📊 UrquiOffice: Registro de Progreso (PROGRESS.md)

> Última actualización: 2026-11-14 01:36 UTC | Rama: `main`
> Regla de oro: tras cada sub-fase completada → actualizar este archivo + `git commit` (+ push si hay remoto).
> Fuente de verdad: `URQUIOFFICE_MASTER_BLUEPRINT.md` (v1.0)

## Estado global

| Fase | Estado | Último commit | Notas |
| :--- | :--- | :--- | :--- |
| 1. Núcleo WordCraft (Rust puro) | ✅ COMPLETADA | (pendiente commit) | Test pasa: lista de dibujo impresa en consola |
| 2. SQLite + Servidor IA local | ⏳ Pendiente | — | |
| 3. Puente FFI Flutter-Rust | ⏳ Pendiente | — | |
| 4. UI visual Flutter (mockups) | ⏳ Pendiente | — | |
| 5. Integración comandos (422) | ⏳ Pendiente | — | |
| 6. Agente de IA en UI | ⏳ Pendiente | — | |
| 7. Tipografías + APK release | ⏳ Pendiente | — | |

## Historial de avances

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
