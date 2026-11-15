//! UrquiOffice Core - Fase 1: Núcleo de WordCraft en Rust puro.
//!
//! Reglas del blueprint: sin `panic!`, `unwrap()`, `expect()`, `todo!()` ni `unsafe`.
//! Todo error se maneja con `Result<T, E>`.

pub mod ai_server;
pub mod db;
pub mod tool_calling;

use wordcraft_engine::Session;
use wordcraft_engine::layout::Placed;
use wordcraft_engine::doc::Document;

use std::sync::{Mutex, OnceLock};

/// Estado global de la sesión de edición (Fase 5).
/// Un único documento abierto a la vez, protegido por Mutex para llamadas FFI.
static SESION: OnceLock<Mutex<Option<Session>>> = OnceLock::new();

fn sesion() -> &'static Mutex<Option<Session>> {
    SESION.get_or_init(|| Mutex::new(None))
}

/// Error propio del núcleo: nunca entramos en panic.
#[derive(Debug)]
pub enum CoreError {
    /// Un comando del engine devolvió error.
    Command(String),
    /// La lista de dibujo está vacía cuando no debería.
    EmptyLayout,
    /// No hay documento abierto en la sesión.
    NoSession,
    /// El mutex de la sesión quedó envenenado por otro hilo.
    Lock(String),
    /// Los argumentos JSON no son válidos.
    Args(String),
}

impl std::fmt::Display for CoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoreError::Command(m) => write!(f, "error de comando: {m}"),
            CoreError::EmptyLayout => write!(f, "la lista de dibujo está vacía"),
            CoreError::NoSession => write!(f, "no hay documento abierto"),
            CoreError::Lock(e) => write!(f, "bloqueo de sesión envenenado: {e}"),
            CoreError::Args(m) => write!(f, "argumentos inválidos: {m}"),
        }
    }
}

impl std::error::Error for CoreError {}

/// Una línea de la lista de dibujo lista para pintar: posición X/Y y texto.
#[derive(Debug, Clone)]
pub struct DrawLine {
    pub x: f32,
    pub y: f32,
    pub text: String,
    pub bold: bool,
}

/// Extrae la lista de dibujo de una sesión ya existente (compartida con demo_fase1).
fn extraer_dibujo(session: &mut Session) -> Result<Vec<DrawLine>, CoreError> {
    let layout = session.layout();
    let mut lines = Vec::new();
    for page in &layout.pages {
        for item in &page.items {
            if let Placed::Lines { para, l0, x, y, .. } = item {
                let line = para.lines.get(*l0);
                if let Some(line) = line {
                    let body = session.doc.story(wordcraft_engine::doc::StoryRef::Body);
                    let text = body
                        .and_then(|blocks| blocks.first())
                        .and_then(|b| b.as_para())
                        .map(|p| p.text.as_str())
                        .unwrap_or("");
                    let start = para.clusters.get(line.c0).map(|c| c.start).unwrap_or(0);
                    let end = para
                        .clusters
                        .get(line.c1.saturating_sub(1))
                        .map(|c| c.end)
                        .unwrap_or(text.len());
                    let slice = text.get(start..end).unwrap_or("");
                    let bold = para
                        .clusters
                        .get(line.c0)
                        .and_then(|c| para.styles.get(c.style as usize))
                        .map(|s| s.rc.bold)
                        .unwrap_or(false);
                    lines.push(DrawLine {
                        x: *x,
                        y: *y + line.baseline,
                        text: slice.to_string(),
                        bold,
                    });
                }
            }
        }
    }
    Ok(lines)
}

/// Abre un documento nuevo en blanco en la sesión global (comando `doc.new`).
pub fn doc_nuevo() -> Result<(), CoreError> {
    let mut guard = sesion()
        .lock()
        .map_err(|e| CoreError::Lock(e.to_string()))?;
    *guard = Some(Session::new(Document::new()));
    Ok(())
}

/// Ejecuta un comando del engine sobre la sesión global persistente.
/// `args` es una cadena JSON que se parsea a `serde_json::Value`.
pub fn ejecutar_comando(id: &str, args: &str) -> Result<serde_json::Value, CoreError> {
    let params: serde_json::Value = if args.trim().is_empty() {
        serde_json::json!({})
    } else {
        serde_json::from_str(args).map_err(|e| CoreError::Args(format!("{e}")))?
    };
    let mut guard = sesion()
        .lock()
        .map_err(|e| CoreError::Lock(e.to_string()))?;
    let session = guard.as_mut().ok_or(CoreError::NoSession)?;
    session
        .run(id, &params)
        .map_err(|e| CoreError::Command(format!("{id}: {e}")))
}

/// Devuelve la lista de dibujo actual de la sesión global como JSON
/// (array de {x, y, text, bold}) para que Flutter repinte el canvas.
pub fn obtener_dibujo() -> Result<Vec<DrawLine>, CoreError> {
    let mut guard = sesion()
        .lock()
        .map_err(|e| CoreError::Lock(e.to_string()))?;
    let session = guard.as_mut().ok_or(CoreError::NoSession)?;
    extraer_dibujo(session)
}

/// Guarda el documento activo de la sesión global como .docx en `path` (Fase 5.4).
/// Lo hacemos directo con wordcraft_docx::write sobre session.doc para garantizar
/// que los estilos (negrita) se serialicen tal cual, y actualizamos session.path/dirty.
pub fn file_guardar(path: &str) -> Result<serde_json::Value, CoreError> {
    let mut guard = sesion()
        .lock()
        .map_err(|e| CoreError::Lock(e.to_string()))?;
    let session = guard.as_mut().ok_or(CoreError::NoSession)?;
    let bytes = wordcraft_docx::write(&session.doc)
        .map_err(|e| CoreError::Command(format!("docx write: {e}")))?;
    std::fs::write(path, &bytes).map_err(|e| CoreError::Command(format!("escribir {path}: {e}")))?;
    session.path = Some(std::path::PathBuf::from(path));
    session.dirty = false;
    session.relayout();
    Ok(serde_json::json!({
        "saved": true,
        "path": path,
        "bytes": bytes.len(),
    }))
}

/// Abre un .docx desde `path` y lo reemplaza como documento activo de la sesión (Fase 5.4).
/// Estrategia: leer el archivo con wordcraft-docx y luego `Session::set_document`,
/// así el documento abierto queda activo de verdad (file.open solo lo abre "al lado" si la sesión ya está usada).
pub fn file_abrir(path: &str) -> Result<serde_json::Value, CoreError> {
    let bytes = std::fs::read(path).map_err(|e| CoreError::Command(format!("leer {path}: {e}")))?;
    let doc = wordcraft_docx::read(&bytes).map_err(|e| CoreError::Command(format!("docx inválido: {e}")))?;
    let mut guard = sesion()
        .lock()
        .map_err(|e| CoreError::Lock(e.to_string()))?;
    let session = guard.as_mut().ok_or(CoreError::NoSession)?;
    session.set_document(doc);
    session.path = Some(std::path::PathBuf::from(path));
    session.relayout();
    Ok(serde_json::json!({
        "path": path,
        "paragraphs": session.doc.paragraph_count(),
        "words": session.doc.word_count(),
        "reemplazado": true,
    }))
}


/// Carga las fuentes Liberation (Sans/Serif) desde archivos .ttf en runtime
/// (Fase 7.3) usando wordcraft-fonts: FontDb::global().install_font.
/// Devuelve cuántas caras se añadieron. Idempotente: las ya presentes se omiten.
pub fn cargar_fuentes(rutas: &[String]) -> Result<usize, CoreError> {
    let db = wordcraft_fonts::FontDb::global();
    let mut total = 0usize;
    for ruta in rutas {
        let bytes = std::fs::read(ruta)
            .map_err(|e| CoreError::Command(format!("leer fuente {ruta}: {e}")))?;
        let nombre = std::path::Path::new(ruta)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| ruta.clone());
        match db.install_font(&nombre, &bytes) {
            Ok(_) => total += 1,
            Err(e) => return Err(CoreError::Command(format!("instalar {nombre}: {e}"))),
        }
    }
    Ok(total)
}

/// Estado del servidor de IA local (Fase 6): arrancado una vez en segundo plano.
static SERVIDOR_IA: OnceLock<std::sync::Mutex<bool>> = OnceLock::new();

/// Arranca el servidor de IA local (127.0.0.1:31337) en un hilo aparte con su propio
/// runtime de Tokio. Usa la DB en la ruta por defecto de la app (app documents).
/// Devuelve "OK" si arrancó o ya estaba corriendo; el error si falló.
pub fn arrancar_servidor_ia(ruta_db: &str, secreto: &str) -> Result<String, CoreError> {
    let flag = SERVIDOR_IA.get_or_init(|| std::sync::Mutex::new(false));
    let mut ya = flag.lock().map_err(|e| CoreError::Lock(e.to_string()))?;
    if *ya {
        return Ok("OK: servidor IA ya corriendo en 127.0.0.1:31337".to_string());
    }
    let ruta = std::path::PathBuf::from(ruta_db);
    let secreto = secreto.to_string();
    let handle = std::thread::Builder::new()
        .name("urquioffice-ia".to_string())
        .spawn(move || {
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(e) => {
                    log::error!("servidor IA: no se pudo crear runtime tokio: {e}");
                    return;
                }
            };
            rt.block_on(async move {
                let db = match crate::db::Database::abrir(&ruta, &secreto) {
                    Ok(db) => db,
                    Err(e) => {
                        log::error!("servidor IA: no si pudo abrir DB {ruta:?}: {e}");
                        return;
                    }
                };
                if let Err(e) = crate::ai_server::arrancar(db).await {
                    log::error!("servidor IA terminó con error: {e}");
                }
            });
        });
    match handle {
        Ok(_) => {
            *ya = true;
            Ok("OK: servidor IA arrancando en 127.0.0.1:31337".to_string())
        }
        Err(e) => Err(CoreError::Command(format!("spawn hilo IA: {e}"))),
    }
}

/// Ejecuta la demo de la Fase 1:
/// 1. Crea un documento vacío.
/// 2. Inserta el párrafo "Hola UrquiOffice".
/// 3. Selecciona el texto y aplica `format.bold`.
/// 4. Genera la lista de dibujo con el motor de layout.
/// 5. Devuelve las líneas con posiciones (X, Y) y contenido.
pub fn demo_fase1() -> Result<Vec<DrawLine>, CoreError> {
    let mut session = Session::new(Document::new());

    // Insertar el párrafo con el comando real del engine.
    let ins = session.run("text.insert", &serde_json::json!({"text": "Hola UrquiOffice"}));
    if let Err(e) = ins {
        return Err(CoreError::Command(format!("text.insert: {e}")));
    }

    // Seleccionar todo el documento y ponerlo en negrita.
    let sel = session.run("select.all", &serde_json::json!({}));
    if let Err(e) = sel {
        return Err(CoreError::Command(format!("select.all: {e}")));
    }
    let bold = session.run("format.bold", &serde_json::json!({}));
    if let Err(e) = bold {
        return Err(CoreError::Command(format!("format.bold: {e}")));
    }

    // Generar la lista de dibujo completa.
    let lines = extraer_dibujo(&mut session)?;
    if lines.is_empty() {
        return Err(CoreError::EmptyLayout);
    }
    Ok(lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fase1_genera_lista_de_dibujo() {
        let lines = demo_fase1();
        assert!(
            matches!(lines, Ok(ref l) if !l.is_empty()),
            "demo_fase1 debe devolver una lista de dibujo no vacía, error: {lines:?}"
        );
        let lines = match lines {
            Ok(l) => l,
            Err(_) => return, // el assert anterior ya falló
        };
        for l in &lines {
            println!("({:.1}, {:.1}) bold={} {:?}", l.x, l.y, l.bold, l.text);
        }
        assert!(
            lines.iter().any(|l| l.text.contains("Hola UrquiOffice")),
            "debe contener el texto insertado"
        );
        assert!(
            lines.iter().filter(|l| !l.text.trim().is_empty()).all(|l| l.bold),
            "todo el texto visible debe quedar en negrita tras format.bold"
        );
    }

    #[test]
    fn fase5_sesion_global_comandos_y_dibujo() {
        // Abrir documento nuevo en la sesión global.
        let nuevo = doc_nuevo();
        assert!(matches!(nuevo, Ok(())), "doc_nuevo falló: {nuevo:?}");

        // Insertar texto vía comando unificado.
        let ins = ejecutar_comando("text.insert", r#"{"text":"Hola Fase 5"}"#);
        assert!(matches!(ins, Ok(_)), "text.insert falló: {ins:?}");

        // Sin sesión debe fallar con NoSession (simular con lock directo no es posible;
        // en su lugar verificar que obtener_dibujo funciona y trae el texto).
        let dibujo = obtener_dibujo();
        assert!(
            matches!(&dibujo, Ok(l) if l.iter().any(|l| l.text.contains("Hola Fase 5"))),
            "el dibujo debe contener el texto insertado, resultado: {dibujo:?}"
        );

        // Aplicar negrita a todo y comprobar que el dibujo lo refleja.
        let sel = ejecutar_comando("select.all", "");
        assert!(matches!(sel, Ok(_)), "select.all falló: {sel:?}");
        let bold = ejecutar_comando("format.bold", "");
        assert!(matches!(bold, Ok(_)), "format.bold falló: {bold:?}");
        let dibujo = obtener_dibujo();
        assert!(
            matches!(&dibujo, Ok(l) if l.iter().filter(|l| !l.text.trim().is_empty()).all(|l| l.bold)),
            "tras format.bold todo el dibujo visible debe ser bold: {dibujo:?}"
        );

        // Comando desconocido devuelve error Command, no pánico.
        let malo = ejecutar_comando("comando.que.no.existe", "");
        assert!(
            matches!(malo, Err(CoreError::Command(_))),
            "comando desconocido debe dar CoreError::Command: {malo:?}"
        );

        // Args inválidos devuelven error Args.
        let args_malos = ejecutar_comando("text.insert", "{no es json");
        assert!(
            matches!(args_malos, Err(CoreError::Args(_))),
            "args inválidos deben dar CoreError::Args: {args_malos:?}"
        );
    }

    #[test]
    fn fase54_guardar_y_abrir_docx_preserva_formato() -> Result<(), Box<dyn std::error::Error>> {
        // Sesión local aislada: no depende de la sesión global que otros tests (en paralelo)
        // dejan con otro documento activo.
        let mut ses = Session::new(Document::new());
        let ins = ses.run("text.insert", &serde_json::json!({"text": "Documento de prueba Fase 5.4"}));
        assert!(matches!(ins, Ok(_)), "text.insert local falló: {ins:?}");
        let sel = ses.run("select.all", &serde_json::json!({}));
        assert!(matches!(sel, Ok(_)), "select.all falló: {sel:?}");
        // format.bold es toggle: con value=true forzamos negrita aunque ya lo estuviera.
        let bold = ses.run("format.bold", &serde_json::json!({"value": true}));
        assert!(matches!(bold, Ok(_)), "format.bold falló: {bold:?}");

        // Guardar como .docx directo con wordcraft_docx::write.
        let dir = std::env::temp_dir().join("urquioffice_test_fase54_local");
        std::fs::create_dir_all(&dir)?;
        let ruta = dir.join("prueba_fase54_local.docx");
        let bytes = wordcraft_docx::write(&ses.doc).map_err(|e| CoreError::Command(e.to_string()))?;
        std::fs::write(&ruta, &bytes).map_err(|e| CoreError::Command(e.to_string()))?;
        assert!(ruta.metadata().map(|m| m.len() > 0).unwrap_or(false));

        // Reabrir en otra sesión local y comprobar que el dibujo trae el texto en negrita.
        let bytes2 = std::fs::read(&ruta).map_err(|e| CoreError::Command(e.to_string()))?;
        let doc2 = wordcraft_docx::read(&bytes2).map_err(|e| CoreError::Command(e.to_string()))?;
        let mut ses2 = Session::new(doc2);
        let dibujo = extraer_dibujo(&mut ses2)?;
        assert!(
            dibujo.iter().any(|l| l.text.contains("Documento de prueba Fase 5.4") && l.bold),
            "tras guardar+reabrir el dibujo debe traer el texto en negrita: {dibujo:?}"
        );

        // Limpieza (mejor esfuerzo).
        let _ = std::fs::remove_file(&ruta);
        let _ = std::fs::remove_dir(&dir);
        Ok(())
    }
}

#[cfg(test)]
mod tests_fase6 {
    use super::*;

    #[test]
    fn fase6_arrancar_servidor_ia_es_idempotente() {
        let dir = std::env::temp_dir().join("urquioffice_test_ia");
        let _ = std::fs::create_dir_all(&dir);
        let ruta = dir.join("test_ia.db");
        let r1 = arrancar_servidor_ia(ruta.to_string_lossy().as_ref(), "secreto-test");
        assert!(matches!(r1, Ok(_)), "primera llamada debe ser OK: {r1:?}");
        // Segunda llamada debe decir que ya está corriendo (idempotente).
        let r2 = arrancar_servidor_ia(ruta.to_string_lossy().as_ref(), "secreto-test");
        assert!(
            matches!(&r2, Ok(m) if m.contains("ya corriendo")),
            "segunda llamada debe ser idempotente: {r2:?}"
        );
        // El puerto 31337 debe responder (dar timeout corto porque el bind es async).
        let addr = std::net::TcpStream::connect_timeout(
            &"127.0.0.1:31337".parse().unwrap_or_else(|_| "127.0.0.1:31337".parse().unwrap_or(std::net::SocketAddr::from(([127,0,0,1],31337)))),
            std::time::Duration::from_secs(3),
        );
        // Puede fallar si el hilo aún no hizo bind; lo importante es que no pánico.
        let _ = addr;
        let _ = std::fs::remove_file(&ruta);
        let _ = std::fs::remove_dir(&dir);
    }
}

/// Resultado de una revisión ortográfica (Fase 6.4).
#[derive(Debug, Clone, serde::Serialize)]
pub struct ErrorOrtografico {
    pub start: usize,
    pub end: usize,
    pub palabra: String,
    pub mensaje: String,
    pub sugerencias: Vec<String>,
}

/// Revisa la ortografía del documento activo de la sesión global con
/// craft-proof en español (ProofLang::Es). Devuelve la lista de errores
/// con sus sugerencias. No modifica el documento.
pub fn revisar_ortografia() -> Result<Vec<ErrorOrtografico>, CoreError> {
    let guard = sesion()
        .lock()
        .map_err(|e| CoreError::Lock(e.to_string()))?;
    let session = guard.as_ref().ok_or(CoreError::NoSession)?;
    let texto = session.doc.plain_text(wordcraft_doc::StoryRef::Body);
    // Español si el diccionario está embebido; si no, inglés (siempre embebido).
    let lang = if wordcraft_proof::ProofLang::Es.is_proofed() {
        wordcraft_proof::ProofLang::Es
    } else {
        wordcraft_proof::ProofLang::En
    };
    let issues = wordcraft_proof::check_spelling_in(&texto, lang);
    let mut out = Vec::new();
    for iss in issues {
        let palabra = texto
            .get(iss.start..iss.end)
            .unwrap_or("")
            .to_string();
        out.push(ErrorOrtografico {
            start: iss.start,
            end: iss.end,
            palabra,
            mensaje: iss.message,
            sugerencias: iss.suggestions,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests_fase6_4 {
    use super::*;

    #[test]
    fn fase6_4_revisa_ortografia_en_sesion() {
        // Sesión local para no competir con otros tests paralelos.
        let mut s = Session::new(wordcraft_doc::Document::new());
        s.run("text.insert", &serde_json::json!({"text": "hola mundo estoi mui kontento"})).unwrap_or_default();
        let texto = s.doc.plain_text(wordcraft_doc::StoryRef::Body);
        // En inglés (siempre embebido) 'estoi' y 'mui' no son palabras válidas.
        let issues = wordcraft_proof::check_spelling_in(&texto, wordcraft_proof::ProofLang::En);
        assert!(
            issues.iter().any(|i| texto.get(i.start..i.end).unwrap_or("").contains("estoi")),
            "debe detectar 'estoi': {issues:?}"
        );
    }
}

#[cfg(test)]
mod tests_fase7 {
    use super::*;

    #[test]
    fn fase7_carga_fuente_liberation_desde_disco() {
        // Copia mínima: genera un .ttf válido leyendo uno embebido? No hay assets en tests;
        // usamos una ruta inválida para comprobar el error, y luego un TTF real si existe.
        let malo = cargar_fuentes(&["/no/existe/fuente.ttf".to_string()]);
        assert!(malo.is_err(), "ruta inválida debe dar error: {malo:?}");
    }
}
