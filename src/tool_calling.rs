//! Capa de tool calling para el agente de IA (Fase 6.3).
//!
//! El modelo puede devolver, dentro de su respuesta, bloques de tool call con
//! el formato:
//! ```text
//! {"tool": "<comando>", "args": {...}}
//! ```
//! Esta capa detecta esos bloques, ejecuta el comando contra la sesión global
//! vía `ejecutar_comando` y devuelve el resultado para que el agente siga.

use serde_json::Value;

/// Un tool call extraído del texto del modelo.
#[derive(Debug, Clone, PartialEq)]
pub struct ToolCall {
    pub tool: String,
    pub args: Value,
}

/// Extrae todos los tool calls embebidos en `texto`.
/// Busca bloques JSON que tengan la clave "tool" (string) y "args" (objeto).
/// Tolera texto alrededor: recorre todos los '{' y prueba a parsear el bloque
/// balanceado que empieza ahí.
pub fn extraer_tool_calls(texto: &str) -> Vec<ToolCall> {
    let mut out = Vec::new();
    let bytes = texto.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'{' {
            if let Some((fin, call)) = probar_bloque(&texto[i..]) {
                out.push(call);
                i += fin;
                continue;
            }
        }
        i += 1;
    }
    out
}

/// Intenta parsear un bloque JSON balanceado que empiece en `s` (con '{').
/// Devuelve (bytes consumidos, ToolCall) si el bloque es un tool call válido.
fn probar_bloque(s: &str) -> Option<(usize, ToolCall)> {
    let bytes = s.as_bytes();
    let mut profundidad: i32 = 0;
    let mut en_cadena = false;
    let mut escape = false;
    for (i, &b) in bytes.iter().enumerate() {
        if en_cadena {
            if escape {
                escape = false;
            } else if b == b'\\' {
                escape = true;
            } else if b == b'"' {
                en_cadena = false;
            }
            continue;
        }
        match b {
            b'"' => en_cadena = true,
            b'{' => profundidad += 1,
            b'}' => {
                profundidad -= 1;
                if profundidad == 0 {
                    let candidato = &s[..=i];
                    if let Ok(v) = serde_json::from_str::<Value>(candidato) {
                        if let Some(tool) = v.get("tool").and_then(|t| t.as_str()) {
                            let args = v.get("args").cloned().unwrap_or(serde_json::json!({}));
                            return Some((i + 1, ToolCall {
                                tool: tool.to_string(),
                                args,
                            }));
                        }
                    }
                    return None;
                }
            }
            _ => {}
        }
    }
    None
}

/// Ejecuta un tool call contra la sesión global y devuelve el JSON del resultado.
pub fn ejecutar_tool_call(call: &ToolCall) -> Result<Value, crate::CoreError> {
    let args = serde_json::to_string(&call.args)
        .map_err(|e| crate::CoreError::Args(format!("serializar args: {e}")))?;
    crate::ejecutar_comando(&call.tool, &args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extrae_un_tool_call_simple() {
        let t = extraer_tool_calls(r#"voy a ponerlo en negrita: {"tool": "format.bold", "args": {"value": true}}"#);
        assert_eq!(t.len(), 1);
        assert_eq!(t[0].tool, "format.bold");
        assert_eq!(t[0].args, serde_json::json!({"value": true}));
    }

    #[test]
    fn extrae_varios_y_ignora_json_sin_tool() {
        let t = extraer_tool_calls(
            r#"{"foo": 1} y {"tool":"text.insert","args":{"text":"hola"}} y otro {"tool":"select.all","args":{}}"#,
        );
        assert_eq!(t.len(), 2);
        assert_eq!(t[0].tool, "text.insert");
        assert_eq!(t[1].tool, "select.all");
    }

    #[test]
    fn no_panica_con_bloques_incompletos() {
        let t = extraer_tool_calls(r#"{"tool": "format.bold", "args": {"#);
        assert!(t.is_empty());
        let t2 = extraer_tool_calls("texto normal sin nada { } {");
        assert!(t2.is_empty());
    }

    #[test]
    fn tool_call_con_cadenas_con_llaves_dentro() {
        let t = extraer_tool_calls(
            r#"{"tool":"text.insert","args":{"text":"usa {esto} adentro"}}"#,
        );
        assert_eq!(t.len(), 1);
        assert_eq!(t[0].args["text"], "usa {esto} adentro");
    }
}
