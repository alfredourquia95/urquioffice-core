//! Stub de ureq 3.x compatible en API con lo que usa `craft-font-store` (Agent, get, call,
//! body, body_mut, Error) pero SIN TLS (ring no compila con el NDK en PRoot).
//! En Android las descargas de fuentes/language packs quedan desactivadas: cualquier
//! llamada real devuelve `Error::Timeout` simulado. El HTTP local (127.0.0.1) de la app
//! no usa ureq, usa reqwest.

use std::io::Read;
use std::time::Duration;

#[derive(Debug)]
pub enum Error {
    StatusCode(u16),
    Timeout(&'static str),
    Io(std::io::Error),
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::StatusCode(c) => write!(f, "status {c}"),
            Error::Timeout(_) => write!(f, "timed out"),
            Error::Io(e) => write!(f, "io: {e}"),
            Error::Other(m) => write!(f, "{m}"),
        }
    }
}
impl std::error::Error for Error {}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

pub struct AgentConfig;
pub struct AgentConfigBuilder;

impl AgentConfigBuilder {
    pub fn https_only(self, _v: bool) -> Self { self }
    pub fn user_agent(self, _v: &str) -> Self { self }
    pub fn timeout_connect(self, _v: Option<Duration>) -> Self { self }
    pub fn timeout_recv_response(self, _v: Option<Duration>) -> Self { self }
    pub fn timeout_global(self, _v: Option<Duration>) -> Self { self }
    pub fn max_redirects(self, _v: usize) -> Self { self }
    pub fn http_status_as_error(self, _v: bool) -> Self { self }
    pub fn build(self) -> AgentConfig { AgentConfig }
}

#[derive(Clone)]
pub struct Agent;

impl Agent {
    pub fn config_builder() -> AgentConfigBuilder { AgentConfigBuilder }
    pub fn new_with_config(_c: AgentConfig) -> Self { Agent }
    pub fn new() -> Self { Agent }
    pub fn get(&self, _url: &str) -> RequestBuilder { RequestBuilder }
}

pub struct RequestBuilder;
impl RequestBuilder {
    pub fn call(self) -> Result<Response, Error> {
        // En Android (stub sin TLS) las descargas externas no están disponibles.
        Err(Error::Other("ureq-no-tls stub: descargas externas desactivadas en Android".into()))
    }
}

pub struct Response {
    body: Body,
}

impl Response {
    pub fn status(&self) -> u16 { 200 }
    pub fn body(&self) -> &Body { &self.body }
    pub fn body_mut(&mut self) -> &mut Body { &mut self.body }
}

pub struct Body;
pub struct BodyConfig;
pub struct BodyConfigBuilder;

impl Body {
    pub fn content_length(&self) -> Option<u64> { Some(0) }
    pub fn with_config(&mut self) -> BodyConfigBuilder { BodyConfigBuilder }
}

impl BodyConfigBuilder {
    pub fn limit(self, _n: u64) -> BodyLimit { BodyLimit }
}

pub struct BodyLimit;
impl BodyLimit {
    pub fn reader(self) -> StubReader { StubReader }
}

pub struct StubReader;
impl Read for StubReader {
    fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
        Ok(0) // EOF inmediato
    }
}
