//! UrquiOffice Core - Fase 1: Núcleo de WordCraft en Rust puro.
//!
//! Reglas del blueprint: sin `panic!`, `unwrap()`, `expect()`, `todo!()` ni `unsafe`.
//! Todo error se maneja con `Result<T, E>`.

pub mod ai_server;
pub mod db;

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
}
