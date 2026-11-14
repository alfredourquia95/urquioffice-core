//! Capa FFI de UrquiOffice: expone el núcleo Rust a Flutter vía flutter_rust_bridge.

/// Inicializa la app: ejecuta la demo de la Fase 1 (crear doc, negrita, layout)
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

/// Ejecuta un comando del engine de WordCraft por nombre (Fase 5).
/// Por ahora solo devuelve eco hasta cablear la Session persistente.
pub fn execute_command(command_name: String, args: String) -> Result<String, String> {
    // TODO Fase 5: mantener Session en un estado global y ejecutar aquí.
    let _ = (command_name, args);
    Ok("comando recibido (pendiente Fase 5)".to_string())
}
