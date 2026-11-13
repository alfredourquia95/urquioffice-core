
Guárdalo como URQUIOFFICE_MASTER_BLUEPRINT.md y úsalo como la única fuente de verdad para tu agente.

---

```markdown
# 👑 UrquiOffice: Master Blueprint de Desarrollo (Rust-First + Flutter)
**Versión:** 1.0 | **Modelo Asignado:** Mistral Large 4 | **Fecha:** 2026-11-13
**Objetivo:** Construir una suite de ofimática profesional para Android (tablet + teclado/ratón), con motor de texto en Rust (WordCraft), interfaz en Flutter, base de datos SQLite y un agente de IA autónomo con servidor local compatible con OpenAI.

---

## 📜 1. Filosofía del Proyecto y Reglas de Oro

### 1.1. Principio Rector: "Rust es el Cerebro, Flutter es la Cara"
*   **Rust (Core):** Contiene toda la lógica de negocio. El modelo del documento, el motor de maquetación (`layout`), la base de datos, el cliente de IA y el sistema de comandos. **Prohibido** que Flutter realice cálculos de posicionamiento de texto o lógica de edición.
*   **Flutter (UI):** Es una capa de presentación "tonta". Recibe una **lista de dibujo** (posiciones X/Y, texto, colores) desde Rust y la pinta con `CustomPainter`. Captura eventos de teclado/ratón y los traduce en comandos para Rust.

### 1.2. Reglas de Oro para el Agente (Mistral Large 4)
1.  **No mezclar fases:** No se escribe una sola línea de Flutter hasta que la fase de Rust esté 100% probada en consola. El fracaso anterior se debió a la sobrecarga de contexto.
2.  **Manejo de Errores:** En Rust, están **prohibidos** `panic!`, `unwrap()`, `expect()`, `todo!()` e `unsafe`. Todo error debe manejarse con `Result<T, E>`.
3.  **Supervivencia en PRoot:** Si `cargo` falla por los enlaces simbólicos de Git, ejecutar:
    ```bash
    export CARGO_NET_OFFLINE=true
    find ~/.cargo/git/db -name "*.l2s.*" -delete
    ```
4.  **Gestión de Contexto:** Tras completar cada sub-fase, ejecutar `git commit`, `git push` y un `/compact` para liberar memoria.
5.  **Ética y Licencias:** No copiar iconos, fuentes ni assets de Microsoft Office. Todo debe ser original o de licencia abierta (CC0, MIT, Apache 2.0).

---

## 🏗️ 2. Arquitectura del Sistema y Capas de WordCraft

WordCraft está diseñado en capas estrictas (L0 a L6)[reference:0]. Tu app `UrquiOffice` solo necesitará las capas **L0 a L4**. Las capas L5 (MCP) y L6 (UI de escritorio en egui) serán reemplazadas por tu propia interfaz en Flutter.

| Capa | Crate (Nombre en Cargo) | ¿Qué hace? | ¿Por qué lo necesitas? |
| :--- | :--- | :--- | :--- |
| **L0** | `wordcraft-geom` | Unidades de medida (puntos, twips, EMU, cm), rectángulos. | Es la base para posicionar cualquier elemento en la página[reference:1]. |
| **L1** | `wordcraft-doc` | Modelo del documento: párrafos, runs, tablas, estilos, secciones. | Es el corazón del editor. Sin él, no hay documento que editar[reference:2]. |
| **L1** | `wordcraft-fonts` | Base de datos de fuentes, métricas, shaping con HarfRust. | Necesario para que el texto se vea idéntico a Word[reference:3]. |
| **L1** | `wordcraft-proof` (`craft-proof`) | Corrector ortográfico y gramatical. **Incluye español (España, México, Latinoamérica, Uruguay)**. | Tu ventaja competitiva: corrección nativa en español[reference:4]. |
| **L2** | `wordcraft-layout` | **El motor de maquetación.** Calcula saltos de línea, paginación y genera la **lista de dibujo** (posiciones X/Y de cada glifo). | Es la pieza clave. Le entregará a Flutter todo el trabajo "masticado" para que solo pinte[reference:5]. |
| **L2** | `wordcraft-docx` | Lector/escritor de archivos `.docx` (OOXML). | Indispensable para la compatibilidad profesional con Word[reference:6]. |
| **L2** | `wordcraft-formats` | Importación/exportación de TXT, Markdown, HTML, RTF, ODT, LaTeX. | Para que tu app no se limite solo a `.docx`[reference:7]. |
| **L4** | `wordcraft-engine` | `Session`, estado de edición, **registro de 422 comandos**, undo/redo. | Es el orquestador. Tu UI de Flutter llamará a comandos como `format.bold`[reference:8]. |

### 2.1. Dependencias en `Cargo.toml`
```toml
[dependencies]
wordcraft-geom = { git = "https://github.com/storytold/wordcraft", package = "geom" }
wordcraft-doc = { git = "https://github.com/storytold/wordcraft", package = "doc" }
wordcraft-fonts = { git = "https://github.com/storytold/wordcraft", package = "fonts" }
wordcraft-layout = { git = "https://github.com/storytold/wordcraft", package = "layout" }
wordcraft-engine = { git = "https://github.com/storytold/wordcraft", package = "engine" }
wordcraft-docx = { git = "https://github.com/storytold/wordcraft", package = "docx" }
wordcraft-proof = { package = "craft-proof", git = "https://github.com/storytold/craft-libs" }
rusqlite = { version = "0.40", features = ["bundled"] }
reqwest = { version = "0.12", features = ["json", "stream"] }
axum = "0.7"
tokio = { version = "1", features = ["full"] }
```

---

🗺️ 3. Las 7 Fases de Desarrollo (Guía Paso a Paso)

Fase 1: El Núcleo de WordCraft (Rust Puro)

Meta: Demostrar que el motor de WordCraft funciona en la tablet sin interfaz gráfica.

· Paso 1.1: Crear el crate urquioffice-core con cargo new --lib.
· Paso 1.2: Añadir las dependencias de WordCraft al Cargo.toml.
· Paso 1.3: Escribir un main.rs (o un test en lib.rs) que:
  1. Cree un doc::Document vacío.
  2. Inserte un párrafo con el texto "Hola UrquiOffice".
  3. Ejecute el comando format.bold sobre ese texto usando el engine.
  4. Invoque a layout::DocLayout para generar la lista de dibujo.
  5. Imprima en consola las posiciones (X, Y) y el contenido de cada glifo.
· Criterio de Éxito: cargo run debe imprimir la lista de dibujo sin errores.

Fase 2: Base de Datos y Servidor IA (Rust Puro)

Meta: Preparar el cerebro de datos y comunicaciones antes de la UI.

· Paso 2.1: Implementar SQLite con rusqlite (modo bundled para evitar dependencias del sistema). Crear tablas:
  · settings (configuración de usuario).
  · api_keys (claves de API cifradas, idealmente con ring o aes-gcm).
  · chat_history (historial de mensajes de la IA).
  · recent_files (archivos abiertos recientemente).
· Paso 2.2: Implementar el servidor HTTP local con axum. Debe escuchar en 127.0.0.1:31337 y exponer un endpoint compatible con OpenAI (/v1/chat/completions).
· Paso 2.3: Implementar la lógica de resiliencia:
  · Rate Limiter: Token bucket para no superar las 40 RPM de NVIDIA.
  · Backoff Exponencial: Reintentos para errores 429 y 5xx, respetando Retry-After.
  · Stream Watchdog: Temporizador de inactividad por chunk de 20s (no usar timeout global de petición).
  · Rotación de Claves: Si una clave falla, saltar a la siguiente.
· Criterio de Éxito: Un test en Rust debe guardar una clave en SQLite, hacer una petición a Venagate (http://127.0.0.1:31079) y recibir una respuesta.

Fase 3: El Puente (FFI con Flutter)

Meta: Conectar el motor de Rust con la interfaz de Flutter.

· Paso 3.1: Inicializar el proyecto Flutter (flutter create .).
· Paso 3.2: Configurar flutter_rust_bridge (agregar dependencia, crear el archivo rust/src/api.rs).
· Paso 3.3: Exponer una función init_app() en Rust con la anotación #[flutter_rust_bridge::frb(init)].
· Paso 3.4: Generar los bindings con flutter_rust_bridge_codegen generate.
· Paso 3.5: En Flutter, llamar a init_app() en el main() y mostrar un Text('Conexión con Rust establecida') si la llamada es exitosa.
· Criterio de Éxito: El APK compila e instala, y al abrirlo muestra el mensaje de conexión sin el error undefined symbol: init_app.

Fase 4: La Interfaz Visual (Flutter Puro - Mockups)

Meta: Construir la "cara" de UrquiOffice sin lógica real todavía.

· Paso 4.1: Crear el widget Ribbon (barra superior con pestañas: Inicio, Insertar, Diseño, IA). Usar MouseRegion para efectos hover y Shortcuts para atajos de teclado.
· Paso 4.2: Crear el widget DocumentCanvas usando CustomPainter. Por ahora, dibujar un rectángulo blanco (la hoja) y texto estático de prueba.
· Paso 4.3: Crear el widget SidePanelIA (panel lateral derecho) con un campo de texto y un botón de enviar. Por ahora, el botón no hace nada.
· Paso 4.4: Crear el AppBar con el título "UrquiOffice" y botones de guardar/abrir.
· Criterio de Éxito: La UI se ve como un editor de texto moderno, responde al ratón y al teclado, pero no edita nada real.

Fase 5: La Integración (Los 422+ Comandos)

Meta: Conectar los botones de Flutter con el motor real de Rust.

· Paso 5.1: Exponer en Rust una función unificada: execute_command(command_name: String, args: String) -> Result<String, String>.
· Paso 5.2: En Flutter, mapear el botón "Negrita" para que llame a execute_command("format.bold", ""). El DocumentCanvas debe recibir la nueva lista de dibujo y repintarse.
· Paso 5.3: Repetir para los comandos más usados: text.insert, file.save, file.open, edit.undo, edit.redo, format.italic, format.underline, align.left, align.center, align.right.
· Paso 5.4: Implementar el guardado y apertura de archivos .docx usando el Storage Access Framework (SAF) de Android. Flutter obtiene la ruta del archivo y se la pasa a Rust.
· Criterio de Éxito: Escribir texto en la app, aplicar negrita, guardar el archivo, cerrarlo, volver a abrirlo y ver que el formato se mantiene.

Fase 6: El Agente de IA en la UI

Meta: Darle vida al chat lateral y permitir que la IA edite el documento.

· Paso 6.1: Conectar el SidePanelIA de Flutter con el servidor HTTP local de Rust.
· Paso 6.2: Implementar el streaming de respuestas: Rust envía chunks de texto a Flutter, que los muestra en el chat en tiempo real.
· Paso 6.3: Implementar el Tool Calling. La IA puede devolver una respuesta como:
  ```json
  {"tool": "format.bold", "args": {"paragraph": 1}}
  ```
  Rust intercepta esto, lo ejecuta en el engine y actualiza el documento.
· Paso 6.4: Añadir un botón de "Aplicar corrección ortográfica" que envíe el texto a la IA y reciba el texto corregido.
· Criterio de Éxito: Le pides a la IA "Pon el primer párrafo en negrita" y el cambio se refleja en el DocumentCanvas.

Fase 7: Tipografías y Empaquetado Final

Meta: Dejar la app lista para el usuario final.

· Paso 7.1: Descargar las fuentes de código abierto (Arial y Times New Roman no son de código abierto, pero existen alternativas libres como Liberation Sans y Liberation Serif que son métricamente compatibles).
· Paso 7.2: Incluir los archivos .ttf en la carpeta assets/fonts/ de Flutter y declararlos en el pubspec.yaml.
· Paso 7.3: Configurar el crate wordcraft-fonts de Rust para que cargue estas fuentes desde los assets de la app en tiempo de ejecución. WordCraft recomienda usar la variable de entorno CRAFT_I18N_DIR para los diccionarios.
· Paso 7.4: Compilar el APK de release: flutter build apk --release. Instalar en la tablet y verificar que todo funciona sin conexión a internet (excepto la IA).
· Criterio de Éxito: La app es estable, las fuentes se ven correctamente y el chat de IA responde.

---

🚑 4. Guía de Solución de Problemas (PRoot y Android)

Problema Causa Solución
Failed to lookup symbol 'init_app' Los bindings de flutter_rust_bridge no se regeneraron. Ejecutar flutter_rust_bridge_codegen generate y luego cargo build en la carpeta de Rust.
error: failed to get ... as a git dependency PRoot renombra los archivos .lock de Git. export CARGO_NET_OFFLINE=true y find ~/.cargo/git/db -name "*.l2s.*" -delete.
Timeout o error 429 de NVIDIA Límite de 40 RPM o red inestable. El servidor Rust debe implementar Rate Limiter, Backoff y rotación multikey.
La app se congela al compilar Rust RAM insuficiente (8 GB). Usar cargo check antes de cargo build. Compilar con -j 2 o -j 1.
El teclado virtual tapa el cursor GameActivity no ajusta el viewport. En el shell de Android, interceptar el evento de apertura del teclado (IME) y ajustar el padding del DocumentCanvas.

---

🎯 5. Prompt de Arranque para Mistral Large 4

"Socio, eres el arquitecto principal de UrquiOffice. Lee este archivo URQUIOFFICE_MASTER_BLUEPRINT.md y síguelo al pie de la letra. No te desvíes.

Hoy empezamos la Fase 1: El Núcleo de WordCraft en Rust Puro. No quiero ver ni una línea de Flutter todavía.

1. Crea el crate urquioffice-core con cargo new --lib urquioffice-core.
2. Añade las dependencias de WordCraft y craft-proof al Cargo.toml.
3. Escribe un test unitario que cree un documento, inserte el texto "Hola UrquiOffice", aplique negrita con el comando format.bold y genere la lista de dibujo. Imprime las posiciones en consola.
4. Ejecuta cargo check y cargo test. Si falla la descarga de Git por PRoot, aplica las soluciones del apartado 4.

Demuéstrame que el motor funciona antes de pasar a la Fase 2. Responde de forma concisa, sin explicaciones largas. ¡Dale!"

```

---
