//! Direct h2 + rustls Cursor transport.
//!
//! Two entry points:
//!
//! - `open_streaming_run`: opens a bidirectional Connect stream against
//!   `/agent.v1.AgentService/Run`, returning a `RunStream` that the caller
//!   drives to read decoded frames and write follow-on bytes (heartbeats,
//!   exec/kv replies, append-blob bodies).
//! - `unary_get_usable_models`: single-shot HTTP/2 against
//!   `/agent.v1.AgentService/GetUsableModels` returning the raw response
//!   body (caller is responsible for both Connect-wrapped and raw protobuf
//!   decode paths).
//!
//! Heartbeats are owned by `RunStream`; a 5-second tokio interval task is
//! spawned alongside the read side and writes a Connect-framed
//! `ClientHeartbeat` until the read loop exits.
//!
//! The transport intentionally does not import `hyper`; the ADR pins the
//! direct `h2::client::handshake` path.

use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
    time::Duration,
};

use bytes::Bytes;
use h2::client::SendRequest;
use http::{Request, StatusCode};
use rustls::pki_types::ServerName;
use rustls::{ClientConfig, RootCertStore};
use rustls_native_certs::load_native_certs;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tokio_rustls::TlsConnector;

use crate::upstream::cursor::connect::{
    frame_connect_message, parse_connect_end_stream, take_connect_frame, ConnectError,
    CONNECT_END_STREAM_FLAG, GRPC_WEB_TRAILER_FLAG,
};
use crate::upstream::cursor::proto::encode_client_heartbeat;
use crate::upstream::cursor::{
    cursor_client_version, CURSOR_API_HOST, CURSOR_GET_USABLE_MODELS_PATH, CURSOR_RUN_PATH,
};

/// Per-stream read deadline. Matches the canonical Rust adapter so existing
/// observability dashboards keep their meaning.
pub const READ_DEADLINE: Duration = Duration::from_secs(90);

/// Connect-phase deadline (TCP + TLS + h2 handshake).
pub const CONNECT_DEADLINE: Duration = Duration::from_secs(30);

/// Heartbeat tick. Verified live in Phase 0 against the Cursor side; matches
/// the Node bridge cadence.
pub const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// Errors surfaced by the Cursor transport. Higher-level lanes wrap these
/// into `AppError` variants; Lane J validates the mapping.
#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("cursor transport io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("cursor TLS error: {0}")]
    Tls(String),
    #[error("cursor h2 error: {0}")]
    H2(String),
    #[error("cursor http error ({status}): {body}")]
    Upstream { status: u16, body: String },
    #[error("cursor connect error [{request_id}] {code}: {message}")]
    Connect {
        code: String,
        message: String,
        request_id: String,
    },
    #[error("cursor stream timed out")]
    Timeout,
    #[error("cursor connect timeout")]
    ConnectTimeout,
    #[error("cursor request build failed: {0}")]
    Request(String),
}

pub type TransportResult<T> = Result<T, TransportError>;

static TLS_CONNECTOR: OnceLock<TlsConnector> = OnceLock::new();

struct PooledH2Connection {
    sender: SendRequest<Bytes>,
    _connection_handle: ConnectionGuard,
}

fn h2_pool() -> &'static Mutex<HashMap<&'static str, PooledH2Connection>> {
    static CELL: OnceLock<Mutex<HashMap<&'static str, PooledH2Connection>>> = OnceLock::new();
    CELL.get_or_init(|| Mutex::new(HashMap::new()))
}

pub async fn pooled_send_request_for_host(
    host: &'static str,
) -> TransportResult<SendRequest<Bytes>> {
    let mut pool = h2_pool().lock().await;
    if let Some(connection) = pool.get(host) {
        return Ok(connection.sender.clone());
    }
    let (sender, connection_handle) = connect_h2_host(host).await?;
    let pooled_sender = sender.clone();
    pool.insert(
        host,
        PooledH2Connection {
            sender,
            _connection_handle: connection_handle,
        },
    );
    Ok(pooled_sender)
}

/// Build the rustls `TlsConnector` used for every Cursor request. ALPN is
/// pinned to `h2` so a server fallback to HTTP/1.1 fails the handshake
/// instead of silently switching protocols.
///
/// The connector (and native root loading) is computed once per process and
/// reused for all Cursor h2 connections.
pub fn tls_connector() -> TransportResult<TlsConnector> {
    if let Some(existing) = TLS_CONNECTOR.get() {
        return Ok(existing.clone());
    }

    // Idempotent install; safe under concurrent first calls.
    let _ = rustls::crypto::ring::default_provider().install_default();

    let mut roots = RootCertStore::empty();
    let certs = load_native_certs();
    for cert in certs.certs {
        let _ = roots.add(cert);
    }
    if !certs.errors.is_empty() {
        tracing::debug!(errors = ?certs.errors, "cursor: some native root certificates failed to load");
    }
    let mut config = ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    config.alpn_protocols = vec![b"h2".to_vec()];
    let connector = TlsConnector::from(Arc::new(config));

    // Best-effort insert; if another thread won the race we still return the one we built.
    let _ = TLS_CONNECTOR.set(connector.clone());
    Ok(connector)
}

/// Active streaming Cursor Run session.
///
/// The reader loop pumps inbound h2 chunks, splits them on Connect frames,
/// and forwards complete payloads to `next_frame`. `send_frame` writes to
/// the same h2 stream (e.g. for tool-result resume). `close` stops the
/// heartbeat and flushes a terminal h2 END_STREAM frame.
pub struct RunStream {
    /// Receiver fed by the reader task with decoded inbound payloads.
    rx: mpsc::Receiver<Bytes>,
    /// Active stream send handle returned by `send_request`. Wrapped so the
    /// heartbeat task and the public `send_frame` API can both write.
    stream_send: Arc<Mutex<h2::SendStream<Bytes>>>,
    /// h2 connection driver for non-pooled streams. Pooled streams keep the
    /// connection guard in the process-global h2 pool.
    _connection_handle: Option<ConnectionGuard>,
    /// Cursor request id reflected from the response trailer / the outbound
    /// `x-request-id` header. Surfaced on `ConnectError`.
    request_id: String,
    /// Reader task lifetime guard. Aborted on `close`.
    reader_handle: JoinHandle<TransportResult<()>>,
    /// Heartbeat task guard. Aborted on `close`.
    heartbeat_handle: JoinHandle<()>,
    /// Latest connect-end-stream error emitted by the reader, if any.
    terminal_error: Arc<Mutex<Option<ConnectError>>>,
    /// Flag flipped to `true` once the reader sees END_STREAM.
    closed: Arc<Mutex<bool>>,
}

impl RunStream {
    /// Read the next decoded payload, or `None` once the stream has closed.
    pub async fn next_frame(&mut self) -> Option<Bytes> {
        self.rx.recv().await
    }

    /// Send a follow-on Connect-framed body to Cursor. `end_stream` flips the
    /// h2 END_STREAM bit on the outbound side; pass `true` only on the final
    /// frame of the request side.
    pub async fn send_frame(&self, payload: Bytes, end_stream: bool) -> TransportResult<()> {
        let mut send = self.stream_send.lock().await;
        send.send_data(payload, end_stream)
            .map_err(|err| TransportError::H2(format!("failed to send cursor frame: {err}")))
    }

    /// Cursor request id used for cross-process correlation. Matches the
    /// outgoing `x-request-id` header.
    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    /// Return any ConnectError observed in the terminal frame.
    pub async fn take_connect_error(&self) -> Option<ConnectError> {
        self.terminal_error.lock().await.take()
    }

    /// Indicates whether the reader task has observed a terminal frame.
    pub async fn is_closed(&self) -> bool {
        *self.closed.lock().await
    }

    /// Shut down the stream: cancel heartbeat, abort reader, send a final
    /// h2 END_STREAM frame. Idempotent.
    pub async fn close(&mut self) -> TransportResult<()> {
        self.heartbeat_handle.abort();
        let mut send = self.stream_send.lock().await;
        let _ = send.send_data(Bytes::new(), true);
        drop(send);
        self.reader_handle.abort();
        Ok(())
    }
}

/// Open a streaming Run RPC. The caller must drive `next_frame` to completion
/// (or call `close`) so the reader/heartbeat tasks shut down cleanly.
pub async fn open_streaming_run(token: &str, request_body: Vec<u8>) -> TransportResult<RunStream> {
    let request_id = uuid::Uuid::new_v4().to_string();
    let mut send_request = pooled_send_request_for_host(CURSOR_API_HOST).await?;

    let request = Request::builder()
        .method("POST")
        .uri(format!("https://{CURSOR_API_HOST}{CURSOR_RUN_PATH}"))
        .header("content-type", "application/connect+proto")
        .header("te", "trailers")
        .header("authorization", format!("Bearer {token}"))
        .header("x-ghost-mode", "true")
        .header("x-cursor-client-version", cursor_client_version())
        .header("x-cursor-client-type", "cli")
        .header("x-request-id", &request_id)
        .header("connect-protocol-version", "1")
        .body(())
        .map_err(|err| TransportError::Request(err.to_string()))?;

    let (response_fut, mut send_stream) = send_request
        .send_request(request, false)
        .map_err(|err| TransportError::H2(format!("failed to send cursor request: {err}")))?;

    // Initial body frame: the AgentClientMessage envelope wrapped in Connect.
    let initial_frame = frame_connect_message(&request_body, 0);
    send_stream
        .send_data(Bytes::from(initial_frame), false)
        .map_err(|err| TransportError::H2(format!("failed to send cursor run frame: {err}")))?;

    let response = response_fut
        .await
        .map_err(|err| TransportError::H2(format!("cursor h2 response failed: {err}")))?;
    if response.status() != StatusCode::OK {
        return Err(TransportError::Upstream {
            status: response.status().as_u16(),
            body: format!("cursor h2 status {}", response.status()),
        });
    }

    let (_parts, body) = response.into_parts();

    let (frame_tx, frame_rx) = mpsc::channel::<Bytes>(64);
    let stream_send = Arc::new(Mutex::new(send_stream));
    let terminal_error: Arc<Mutex<Option<ConnectError>>> = Arc::new(Mutex::new(None));
    let closed = Arc::new(Mutex::new(false));

    let reader_send = stream_send.clone();
    let reader_terminal = terminal_error.clone();
    let reader_closed = closed.clone();
    let reader_request_id = request_id.clone();
    let reader_handle = tokio::spawn(async move {
        let result =
            run_reader_loop(body, frame_tx, reader_terminal.clone(), reader_request_id).await;
        *reader_closed.lock().await = true;
        // Best-effort flush of the END_STREAM frame.
        let mut send = reader_send.lock().await;
        let _ = send.send_data(Bytes::new(), true);
        result
    });

    let heartbeat_send = stream_send.clone();
    let heartbeat_handle = tokio::spawn(async move {
        let mut ticker = tokio::time::interval(HEARTBEAT_INTERVAL);
        // Skip the immediate first tick — first heartbeat fires after one
        // full interval so it doesn't race the initial body frame.
        ticker.tick().await;
        loop {
            ticker.tick().await;
            let payload = encode_client_heartbeat();
            let frame = frame_connect_message(&payload, 0);
            let mut send = heartbeat_send.lock().await;
            if send.send_data(Bytes::from(frame), false).is_err() {
                break;
            }
        }
    });

    Ok(RunStream {
        rx: frame_rx,
        stream_send,
        _connection_handle: None,
        request_id,
        reader_handle,
        heartbeat_handle,
        terminal_error,
        closed,
    })
}

/// Connection driver guard returned by `connect_h2`. Holds the spawned
/// `connection.await` task; dropping it closes the underlying socket.
struct ConnectionGuard {
    handle: JoinHandle<()>,
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

async fn connect_h2_host(
    host: &'static str,
) -> TransportResult<(SendRequest<Bytes>, ConnectionGuard)> {
    timeout(CONNECT_DEADLINE, async {
        let tcp = TcpStream::connect((host, 443)).await?;
        let connector = tls_connector()?;
        let server_name = ServerName::try_from(host)
            .map_err(|err| TransportError::Tls(format!("invalid cursor server name: {err}")))?;
        let tls = connector
            .connect(server_name, tcp)
            .await
            .map_err(|err| TransportError::Tls(format!("cursor TLS connect failed: {err}")))?;
        let (client, connection) = h2::client::handshake(tls)
            .await
            .map_err(|err| TransportError::H2(format!("cursor h2 handshake failed: {err}")))?;
        let handle = tokio::spawn(async move {
            if let Err(err) = connection.await {
                tracing::debug!(?err, "cursor h2 connection ended");
            }
        });
        Ok::<_, TransportError>((client, ConnectionGuard { handle }))
    })
    .await
    .map_err(|_| TransportError::ConnectTimeout)?
}

async fn run_reader_loop(
    mut body: h2::RecvStream,
    tx: mpsc::Sender<Bytes>,
    terminal_error: Arc<Mutex<Option<ConnectError>>>,
    request_id: String,
) -> TransportResult<()> {
    let mut pending: Vec<u8> = Vec::new();
    let outcome = timeout(READ_DEADLINE, async {
        while let Some(chunk_result) = body.data().await {
            let chunk = chunk_result
                .map_err(|err| TransportError::H2(format!("cursor h2 data failed: {err}")))?;
            let chunk_len = chunk.len();
            pending.extend_from_slice(&chunk);
            // Release h2 flow-control window for the bytes we just consumed.
            let _ = body.flow_control().release_capacity(chunk_len);

            while let Some((flags, payload)) = take_connect_frame(&mut pending) {
                if flags & CONNECT_END_STREAM_FLAG != 0 {
                    if let Some(error) = parse_connect_end_stream(&payload) {
                        *terminal_error.lock().await = Some(error.clone());
                        return Err(TransportError::Connect {
                            code: error.code,
                            message: error.message,
                            request_id: request_id.clone(),
                        });
                    }
                    return Ok(());
                }
                if flags & GRPC_WEB_TRAILER_FLAG != 0 {
                    // gRPC-web trailer block. Treat as terminal; body is
                    // text trailers (e.g. "grpc-status: 0"). Surface a
                    // ConnectError only when the trailers indicate a
                    // non-zero status.
                    if let Some(error) = parse_grpc_trailers(&payload) {
                        *terminal_error.lock().await = Some(error.clone());
                        return Err(TransportError::Connect {
                            code: error.code,
                            message: error.message,
                            request_id: request_id.clone(),
                        });
                    }
                    return Ok(());
                }
                if tx.send(Bytes::from(payload)).await.is_err() {
                    // Receiver dropped; bail out.
                    return Ok(());
                }
            }
        }
        Ok::<(), TransportError>(())
    })
    .await;

    match outcome {
        Ok(Ok(())) => Ok(()),
        Ok(Err(err)) => Err(err),
        Err(_) => Err(TransportError::Timeout),
    }
}

fn parse_grpc_trailers(payload: &[u8]) -> Option<ConnectError> {
    let text = String::from_utf8_lossy(payload);
    let mut status_code = "0";
    let mut status_message = String::new();
    for line in text.split(['\r', '\n']) {
        let line = line.trim();
        if let Some(value) = line.strip_prefix("grpc-status:") {
            status_code = value.trim();
        } else if let Some(value) = line.strip_prefix("grpc-message:") {
            status_message = value.trim().to_string();
        }
    }
    if status_code == "0" || status_code.is_empty() {
        return None;
    }
    Some(ConnectError {
        code: status_code.to_string(),
        message: if status_message.is_empty() {
            "grpc error".to_string()
        } else {
            status_message
        },
    })
}

/// Single-request unary call to `/agent.v1.AgentService/GetUsableModels`.
///
/// The response body is returned verbatim; callers strip an optional
/// Connect frame envelope before decoding the protobuf response per the
/// ADR's dual-shape contract.
pub async fn unary_get_usable_models(token: &str) -> TransportResult<Vec<u8>> {
    let request_id = uuid::Uuid::new_v4().to_string();
    let mut send_request = pooled_send_request_for_host(CURSOR_API_HOST).await?;

    let request = Request::builder()
        .method("POST")
        .uri(format!(
            "https://{CURSOR_API_HOST}{CURSOR_GET_USABLE_MODELS_PATH}"
        ))
        .header("content-type", "application/proto")
        .header("te", "trailers")
        .header("authorization", format!("Bearer {token}"))
        .header("x-ghost-mode", "true")
        .header("x-cursor-client-version", cursor_client_version())
        .header("x-cursor-client-type", "cli")
        .header("x-request-id", &request_id)
        .body(())
        .map_err(|err| TransportError::Request(err.to_string()))?;

    let (response_fut, mut send_stream) = send_request
        .send_request(request, false)
        .map_err(|err| TransportError::H2(format!("failed to send cursor unary request: {err}")))?;
    // Empty body for GetUsableModels.
    send_stream.send_data(Bytes::new(), true).map_err(|err| {
        TransportError::H2(format!("failed to flush cursor unary request: {err}"))
    })?;

    let response = timeout(READ_DEADLINE, response_fut)
        .await
        .map_err(|_| TransportError::Timeout)?
        .map_err(|err| TransportError::H2(format!("cursor unary response failed: {err}")))?;

    let status = response.status();
    let mut body = response.into_body();
    let mut buf = Vec::new();
    while let Some(chunk_result) = timeout(READ_DEADLINE, body.data())
        .await
        .map_err(|_| TransportError::Timeout)?
    {
        let chunk = chunk_result
            .map_err(|err| TransportError::H2(format!("cursor unary data failed: {err}")))?;
        let chunk_len = chunk.len();
        buf.extend_from_slice(&chunk);
        let _ = body.flow_control().release_capacity(chunk_len);
    }

    if status != StatusCode::OK {
        return Err(TransportError::Upstream {
            status: status.as_u16(),
            body: format!("cursor unary status {status}"),
        });
    }

    Ok(buf)
}

/// Strip a leading Connect frame envelope from `bytes` if one is present.
/// Returns the inner payload when the bytes start with a 5-byte Connect
/// header that exactly matches the body length, otherwise returns the
/// input unchanged.
pub fn strip_optional_connect_envelope(bytes: &[u8]) -> &[u8] {
    if bytes.len() < 5 {
        return bytes;
    }
    let len = u32::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]) as usize;
    if bytes.len() == 5 + len {
        return &bytes[5..];
    }
    bytes
}
