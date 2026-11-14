//! Capa FFI de UrquiOffice: expone el núcleo Rust a Flutter vía flutter_rust_bridge.

/// Inicializa la app: crea el documento nuevo en la sesión global (Fase 5)
/// y devuelve un resumen con las primeras líneas de la lista de dibujo.
#[flutter_rust_bridge::frb(init)]
pub fn init_app() -> String {
    iniciar_nucleo()
}

/// Versión pública llamable desde Dart para mostrar el estado del motor.
pub fn estado_nucleo() -> String {
    iniciar_nucleo()
}

fn iniciar_nucleo() -> String {
    let nuevo = urquioffice_core::doc_nuevo();
    if let Err(e) = nuevo {
        return format!("ERROR: doc_nuevo: {e}");
    }
    match urquioffice_core::demo_fase1() {
        Ok(lineas) => {
            let mut out = String::from("OK: motor WordCraft inicializado. Lista de dibujo:\n");
            for l in lineas.iter().take(5) {
                out.push_str(&format!(
                    "  ({:.1}, {:.1}) bold={} {:?}\n",
                    l.x, l.y, l.bold, l.text
                ));
            }
            out
        }
        Err(e) => format!("ERROR: {e}"),
    }
}

/// Ejecuta un comando del engine de WordCraft sobre la sesión global (Fase 5).
/// Devuelve el resultado del comando como JSON en String.
pub fn execute_command(command_name: String, args: String) -> Result<String, String> {
    urquioffice_core::ejecutar_comando(&command_name, &args)
        .map(|v| v.to_string())
        .map_err(|e| e.to_string())
}

/// Devuelve la lista de dibujo actual como JSON: [{x, y, text, bold}, ...].
/// Flutter la usa para repintar el DocumentCanvas.
pub fn obtener_dibujo() -> Result<String, String> {
    urquioffice_core::obtener_dibujo()
        .map(|lineas| {
            let arr: Vec<serde_json::Value> = lineas
                .iter()
                .map(|l| {
                    serde_json::json!({
                        "x": l.x,
                        "y": l.y,
                        "text": l.text,
                        "bold": l.bold,
                    })
                })
                .collect();
            serde_json::to_string(&arr).unwrap_or_else(|_| "[]".to_string())
        })
        .map_err(|e| e.to_string())
}
