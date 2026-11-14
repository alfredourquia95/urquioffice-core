//! UrquiOffice Core - Fase 1: Núcleo de WordCraft en Rust puro.
//!
//! Reglas del blueprint: sin `panic!`, `unwrap()`, `expect()`, `todo!()` ni `unsafe`.
//! Todo error se maneja con `Result<T, E>`.

pub mod ai_server;
pub mod db;

use wordcraft_engine::Session;
use wordcraft_engine::layout::Placed;
use wordcraft_engine::doc::Document;

/// Error propio del núcleo: nunca entramos en panic.
#[derive(Debug)]
pub enum CoreError {
    /// Un comando del engine devolvió error.
    Command(String),
    /// La lista de dibujo está vacía cuando no debería.
    EmptyLayout,
}

impl std::fmt::Display for CoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoreError::Command(m) => write!(f, "error de comando: {m}"),
            CoreError::EmptyLayout => write!(f, "la lista de dibujo está vacía"),
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
    let layout = session.layout();
    let mut lines = Vec::new();
    for page in &layout.pages {
        for item in &page.items {
            if let Placed::Lines { para, l0, x, y, .. } = item {
                let line = para.lines.get(*l0);
                if let Some(line) = line {
                    // Reconstruir el texto de la línea desde los clusters.
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
                    // ¿Es negrita? Mirar el estilo del primer cluster de la línea.
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
}
