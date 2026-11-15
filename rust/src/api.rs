//! Capa FFI de UrquiOffice: expone el núcleo Rust a Flutter vía flutter_rust_bridge.

/// Inicializa la app: crea el documento nuevo en la sesión global (Fase 5)
/// y arranca el servidor de IA local (Fase 6) en 127.0.0.1:31337.
/// Devuelve un resumen con las primeras líneas de la lista de dibujo.
#[flutter_rust_bridge::frb(init)]
pub fn init_app() -> String {
    iniciar_nucleo()
}

/// Versión pública llamable desde Dart para mostrar el estado del motor.
pub fn estado_nucleo() -> String {
    iniciar_nucleo()
}

fn iniciar_nucleo() -> String {
    let mut partes: Vec<String> = Vec::new();

    // Documento nuevo en la sesión global.
    match urquioffice_core::doc_nuevo() {
        Ok(()) => partes.push("documento: OK".to_string()),
        Err(e) => partes.push(format!("documento ERROR: {e}")),
    }

    // Servidor IA local en segundo plano. La DB vive en el dir de la app;
    // Flutter pasa la ruta real con arrancarServidorIa(rutaDb). Aquí usamos
    // una ruta por defecto bajo /data/data del paquete vía env HOME.
    let ruta_db = std::env::var("URQUIOFFICE_DB")
        .unwrap_or_else(|_| "/data/data/com.example.urquioffice/files/urquioffice.db".to_string());
    let secreto = std::env::var("URQUIOFFICE_DB_SECRET")
        .unwrap_or_else(|_| "urquioffice-local-secret".to_string());
    match urquioffice_core::arrancar_servidor_ia(&ruta_db, &secreto) {
        Ok(m) => partes.push(format!("IA: {m}")),
        Err(e) => partes.push(format!("IA ERROR: {e}")),
    }

    match urquioffice_core::demo_fase1() {
        Ok(lineas) => {
            partes.push("motor WordCraft OK. Lista de dibujo:".to_string());
            for l in lineas.iter().take(5) {
                partes.push(format!(
                    "  ({:.1}, {:.1}) bold={} {:?}",
                    l.x, l.y, l.bold, l.text
                ));
            }
        }
        Err(e) => partes.push(format!("motor ERROR: {e}")),
    }
    partes.join("\n")
}

/// Arranca (o confirma) el servidor de IA local. Flutter debe llamar a esta
/// función pasando la ruta real de la base de datos dentro del paquete Android.
pub fn arrancar_servidor_ia(ruta_db: String, secreto: String) -> Result<String, String> {
    urquioffice_core::arrancar_servidor_ia(&ruta_db, &secreto).map_err(|e| e.to_string())
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

/// Guarda el documento activo como .docx en `path` (Fase 5.4, ruta vía SAF).
pub fn file_guardar(path: String) -> Result<String, String> {
    urquioffice_core::file_guardar(&path)
        .map(|v| v.to_string())
        .map_err(|e| e.to_string())
}

/// Abre un .docx desde `path` y lo carga como documento activo (Fase 5.4).
pub fn file_abrir(path: String) -> Result<String, String> {
    urquioffice_core::file_abrir(&path)
        .map(|v| v.to_string())
        .map_err(|e| e.to_string())
}
