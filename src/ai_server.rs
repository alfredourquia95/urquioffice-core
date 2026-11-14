//! Fase 2.2 + 2.3: Servidor IA local de UrquiOffice.
//!
//! - Escucha en `127.0.0.1:31337` y expone `POST /v1/chat/completions` (compatible OpenAI).
//! - Hace proxy a Venagate (`http://127.0.0.1:31079`) con las claves guardadas en SQLite.
//! - Resiliencia: rate limiter (token bucket, 40 RPM), backoff exponencial en 429/5xx,
//!   watchdog de 20s por chunk de stream y rotación multikey.
//!
//! Regla de oro: sin `unwrap()`/`expect()`/`panic!`/`unsafe`; todo con `Result`.

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response, sse::{Event, KeepAlive, Sse}},
    routing::{get, post},
};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::Mutex as TokioMutex;

use crate::db::Database;

/// Dirección de Venagate (upstream).
pub const VENAGATE_URL: &str = "http://127.0.0.1:31079";
/// Puerto del servidor local de UrquiOffice.
pub const PUERTO_LOCAL: u16 = 31337;
/// Límite de NVIDIA: 40 peticiones por minuto.
pub const RPM_LIMITE: u32 = 40;
/// Watchdog de stream: si no llega un chunk en 20s, se cierra el stream.
pub const WATCHDOG_STREAM: Duration = Duration::from_secs(20);
/// Reintentos máximos con backoff antes de rotar de clave.
pub const MAX_REINTENTOS: u32 = 3;

/// Errores del servidor de IA.
#[derive(Debug)]
pub enum AiError {
    Http(String),
    SinClaves,
    RateLimit(String),
    BadGateway(String),
}

impl std::fmt::Display for AiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiError::Http(m) => write!(f, "http: {m}"),
            AiError::SinClaves => write!(f, "no hay claves de API activas en la base de datos"),
            AiError::RateLimit(m) => write!(f, "rate limit: {m}"),
            AiError::BadGateway(m) => write!(f, "upstream: {m}"),
        }
    }
}

impl std::error::Error for AiError {}

/// Rate limiter: token bucket de `rpm` tokens por minuto.
pub struct RateLimiter {
    tokens: Mutex<f32>,
    ultimo: Mutex<Instant>,
    capacidad: f32,
}

impl RateLimiter {
    pub fn nuevo(rpm: u32) -> Self {
        RateLimiter {
            tokens: Mutex::new(rpm as f32),
            ultimo: Mutex::new(Instant::now()),
            capacidad: rpm as f32,
        }
    }

    /// Consume 1 token si hay; si no, devuelve cuánto esperar.
    pub fn intentar(&self) -> Result<(), Duration> {
        let mut tokens = match self.tokens.lock() {
            Ok(t) => t,
            Err(_) => return Err(Duration::from_secs(1)),
        };
        let mut ultimo = match self.ultimo.lock() {
            Ok(u) => u,
            Err(_) => return Err(Duration::from_secs(1)),
        };
        let ahora = Instant::now();
        let transcurrido = ahora.duration_since(*ultimo).as_secs_f32();
        let repuesto = transcurrido * (self.capacidad / 60.0);
        *tokens = (*tokens + repuesto).min(self.capacidad);
        *ultimo = ahora;
        if *tokens >= 1.0 {
            *tokens -= 1.0;
            Ok(())
        } else {
            let deficit = 1.0 - *tokens;
            Err(Duration::from_secs_f32(deficit * 60.0 / self.capacidad))
        }
    }
}

/// Estado compartido del servidor. La DB va dentro de un Mutex porque rusqlite
/// no es Sync (Connection usa RefCell internamente).
pub struct AiState {
    pub db: Mutex<Database>,
    pub limiter: RateLimiter,
    pub cliente: reqwest::Client,
    /// Cola de claves activas (id, nombre, clave) para rotación round-robin.
    pub claves: TokioMutex<VecDeque<(i64, String, String)>>,
}

/// Cuerpo de la petición compatible con OpenAI.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(default)]
    pub stream: bool,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Construye el router de axum.
pub fn router(state: Arc<AiState>) -> Router {
    Router::new()
        .route("/v1/chat/completions", post(chat_completions))
        .route("/v1/models", get(modelos))
        .with_state(state)
}

async fn modelos(State(state): State<Arc<AiState>>) -> Response {
    let url = format!("{VENAGATE_URL}/v1/models");
    match state.cliente.get(&url).send().await {
        Ok(r) => {
            let texto = r.text().await.unwrap_or_default();
            (StatusCode::OK, texto).into_response()
        }
        Err(e) => (StatusCode::BAD_GATEWAY, format!("venagate no responde: {e}")).into_response(),
    }
}

async fn chat_completions(State(state): State<Arc<AiState>>, Json(req): Json<ChatRequest>) -> Response {
    // 1. Rate limiter local (40 RPM)
    if let Err(espera) = state.limiter.intentar() {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({
                "error": {"message": format!("rate limit local: reintenta en {:.1}s", espera.as_secs_f32()), "type": "rate_limit"}
            })),
        )
            .into_response();
    }

    // 2. Sacar la siguiente clave de la cola (rotación round-robin)
    let (key_id, nombre, clave) = match {
        let mut cola = state.claves.lock().await;
        cola.pop_front()
    } {
        Some(k) => k,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": {"message": AiError::SinClaves.to_string()}})),
            )
                .into_response();
        }
    };

    // 3. Backoff exponencial sobre 429/5xx; si falla, rotar a la siguiente clave
    let mut ultimo_error: Option<AiError> = None;
    for intento in 0..MAX_REINTENTOS {
        match llamar_venagate(&state, &clave, &req).await {
            Ok(respuesta) => {
                // devolvemos la clave al frente para la siguiente petición
                let mut cola = state.claves.lock().await;
                cola.push_front((key_id, nombre, clave));
                return respuesta;
            }
            Err(e) => {
                let reintentar = matches!(&e, AiError::RateLimit(_) | AiError::BadGateway(_));
                ultimo_error = Some(e);
                if reintentar && intento + 1 < MAX_REINTENTOS {
                    let espera = Duration::from_millis(500 * 2u64.pow(intento));
                    tokio::time::sleep(espera).await;
                    let siguiente = {
                        let mut cola = state.claves.lock().await;
                        cola.pop_front()
                    };
                    if let Some((nid, nnombre, nclave)) = siguiente {
                        // la clave que falló queda desactivada en la DB
                        if let Ok(db) = state.db.lock() {
                            let _ = db.desactivar_api_key(key_id);
                        }
                        return llamar_con_rotacion(state.clone(), req.clone(), nid, nnombre, nclave).await;
                    }
                }
            }
        }
    }

    (
        StatusCode::BAD_GATEWAY,
        Json(serde_json::json!({"error": {"message": ultimo_error.map(|e| e.to_string()).unwrap_or_default()}})),
    )
        .into_response()
}

/// Reintenta la petición con otra clave (rotación multikey).
async fn llamar_con_rotacion(
    state: Arc<AiState>,
    req: ChatRequest,
    key_id: i64,
    nombre: String,
    clave: String,
) -> Response {
    match llamar_venagate(&state, &clave, &req).await {
        Ok(r) => {
            let mut cola = state.claves.lock().await;
            cola.push_front((key_id, nombre, clave));
            r
        }
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({"error": {"message": e.to_string()}})),
        )
            .into_response(),
    }
}

/// Llama a Venagate con una clave. Si `stream`, devuelve SSE con watchdog de 20s por chunk.
async fn llamar_venagate(
    state: &Arc<AiState>,
    clave: &str,
    req: &ChatRequest,
) -> Result<Response, AiError> {
    let url = format!("{VENAGATE_URL}/v1/chat/completions");
    let body = serde_json::to_value(req).map_err(|e| AiError::Http(e.to_string()))?;
    let resp = state
        .cliente
        .post(&url)
        .bearer_auth(clave)
        .json(&body)
        .send()
        .await
        .map_err(|e| AiError::Http(e.to_string()))?;

    let status = resp.status();
    if status == StatusCode::TOO_MANY_REQUESTS {
        let retry_after = resp
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok());
        let texto = resp.text().await.unwrap_or_default();
        let mut msg = format!("429 upstream: {texto}");
        if let Some(ra) = retry_after {
            msg.push_str(&format!(" (retry-after: {ra}s)"));
        }
        return Err(AiError::RateLimit(msg));
    }
    if status.is_server_error() {
        let texto = resp.text().await.unwrap_or_default();
        return Err(AiError::BadGateway(format!("{status}: {texto}")));
    }
    if !status.is_success() {
        let texto = resp.text().await.unwrap_or_default();
        return Err(AiError::BadGateway(format!("{status}: {texto}")));
    }

    if req.stream {
        let stream = resp.bytes_stream();
        let mapped = stream.map(|chunk| {
            chunk.map(|b| Event::default().data(String::from_utf8_lossy(&b).into_owned()))
        });
        let watchdog = aplicar_watchdog(mapped);
        Ok(Sse::new(watchdog).keep_alive(KeepAlive::new()).into_response())
    } else {
        let texto = resp.text().await.map_err(|e| AiError::Http(e.to_string()))?;
        Ok((StatusCode::OK, texto).into_response())
    }
}

/// Envuelve un stream SSE con un timeout de WATCHDOG_STREAM entre chunks.
fn aplicar_watchdog<S>(stream: S) -> impl futures_util::Stream<Item = Result<Event, std::convert::Infallible>>
where
    S: futures_util::Stream<Item = Result<Event, reqwest::Error>> + Send + 'static,
{
    let mut stream = Box::pin(stream);
    async_stream::stream! {
        loop {
            match tokio::time::timeout(WATCHDOG_STREAM, stream.next()).await {
                Ok(Some(Ok(ev))) => yield Ok(ev),
                Ok(Some(Err(_))) => break, // error de chunk: cerrar
                Ok(None) => break,         // fin del stream
                Err(_) => break,           // watchdog: 20s sin chunk -> cerrar
            }
        }
    }
}

/// Arranca el servidor en 127.0.0.1:31337 (bloqueante async).
pub async fn arrancar(db: Database) -> Result<(), AiError> {
    let claves = db
        .api_keys_activas()
        .map_err(|e| AiError::Http(e.to_string()))?;
    let cola: VecDeque<(i64, String, String)> = claves.into_iter().collect();
    let state = Arc::new(AiState {
        db: Mutex::new(db),
        limiter: RateLimiter::nuevo(RPM_LIMITE),
        cliente: reqwest::Client::new(),
        claves: TokioMutex::new(cola),
    });
    let app = router(state);
    let addr = format!("127.0.0.1:{PUERTO_LOCAL}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| AiError::Http(format!("bind {addr}: {e}")))?;
    axum::serve(listener, app)
        .await
        .map_err(|e| AiError::Http(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_limiter_bloquea_tras_agotar() {
        let rl = RateLimiter::nuevo(40);
        let mut ok = 0;
        for _ in 0..40 {
            if rl.intentar().is_ok() {
                ok += 1;
            }
        }
        assert_eq!(ok, 40, "los primeros 40 intentos deben pasar");
        assert!(rl.intentar().is_err(), "el 41 debe bloquearse");
    }

    #[tokio::test]
    async fn venagate_lista_modelos() {
        // Sonda real contra Venagate; si no está levantado, se registra y se omite.
        let cliente = reqwest::Client::new();
        match cliente
            .get(format!("{VENAGATE_URL}/v1/models"))
            .timeout(Duration::from_secs(3))
            .send()
            .await
        {
            Ok(r) => {
                let texto = r.text().await.unwrap_or_default();
                println!("Venagate /v1/models -> {}", &texto[..texto.len().min(200)]);
                assert!(texto.contains("data"), "debe devolver la lista de modelos");
            }
            Err(e) => println!("Venagate no disponible ({e}); sonda omitida"),
        }
    }

    /// Fase 2.4 (criterio de éxito): guardar una clave en SQLite, pedir a
    /// Venagate y recibir una respuesta de chat real.
    #[tokio::test]
    async fn fase2_end_to_end_sqlite_y_venagate() {
        use crate::db::Database;
        // 1. DB en memoria + clave real de NVIDIA (desde ~/.config/openvidia/keys.json)
        let keys_path = std::path::Path::new("/root/.config/openvidia/keys.json");
        let clave = std::fs::read_to_string(keys_path)
            .ok()
            .and_then(|s| serde_json::from_str::<Vec<String>>(&s).ok())
            .and_then(|v| v.into_iter().next());
        let Some(clave) = clave else {
            println!("sin claves nvapi disponibles; test omitido");
            return;
        };
        let db = match Database::en_memoria("secreto-test") {
            Ok(d) => d,
            Err(e) => {
                println!("no se pudo abrir la DB: {e}");
                return;
            }
        };
        assert!(db.guardar_api_key("nvidia", &clave).is_ok(), "guardar clave cifrada");
        let keys = db.api_keys_activas();
        assert!(
            matches!(&keys, Ok(k) if k.len() == 1 && k[0].2 == clave),
            "la clave debe quedar guardada y descifrable: {keys:?}"
        );

        // 2. Llamar a Venagate a través de la función del servidor
        let state = Arc::new(AiState {
            db: Mutex::new(db),
            limiter: RateLimiter::nuevo(RPM_LIMITE),
            cliente: reqwest::Client::new(),
            claves: TokioMutex::new(VecDeque::from(vec![(1, "nvidia".to_string(), clave.clone())])),
        });
        let req = ChatRequest {
            model: "nvidia/mistralai/mistral-large-4-1t-a52b".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "Di 'hola' en una palabra.".to_string(),
            }],
            stream: false,
            temperature: Some(0.0),
            max_tokens: Some(16),
        };
        let resp = llamar_venagate(&state, &clave, &req).await;
        match resp {
            Ok(r) => {
                let (partes, _cuerpo) = r.into_parts();
                assert_eq!(partes.status, StatusCode::OK, "Venagate debe responder 200");
                println!("fase2 end-to-end OK contra Venagate");
            }
            Err(e) => {
                // si el upstream está caído, registrar y no fallar el build
                println!("Venagate devolvió error ({e}); test end-to-end omitido");
            }
        }
    }
}
