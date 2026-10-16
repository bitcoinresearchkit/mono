//! Blocking HTTP/1.1 transport for Bitcoin Core's JSON-RPC. Keep-alive sockets are pooled, so
//! concurrent callers never queue behind one another's exchanges, and every read ends: on data,
//! EOF (an error mid-response) or the I/O timeout.

use std::{
    error, fmt,
    io::{self, BufRead, BufReader, ErrorKind, Read, Write},
    net::{TcpStream, ToSocketAddrs},
    path::PathBuf,
    result::Result as StdResult,
    str,
    time::Duration,
};

use base64::{Engine, engine::general_purpose::STANDARD};
use brk_error::{Error, Result};
use corepc_jsonrpc::{Request, Response, Transport, error::Error as JsonRpcError};
use parking_lot::{Mutex, RwLock};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{from_slice, to_vec};

use crate::{Auth, auth::read_cookie};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);
/// Per read or write: a long response keeps flowing, a silent node fails the exchange.
const IO_TIMEOUT: Duration = Duration::from_secs(60);
/// Idle sockets kept for reuse; more concurrent callers open short-lived extra ones.
const MAX_IDLE: usize = 8;
/// Status line plus headers. Bitcoin Core sends a few hundred bytes.
const MAX_HEAD_BYTES: usize = 16 * 1024;
/// One chunk-size or trailer line.
const MAX_LINE_BYTES: usize = 1024;
/// Allocated up front for a declared body; larger bodies grow as they arrive.
const MAX_PREALLOC: usize = 16 * 1024 * 1024;

/// Why an exchange failed below JSON-RPC.
#[derive(Debug)]
pub enum HttpError {
    /// The node answered this status without a JSON-RPC body.
    Status(u16),
    /// The response broke HTTP/1.1 framing.
    Malformed(&'static str),
    /// The cookie could not be re-read after the node rejected the current one.
    Cookie(io::Error),
    Io(io::Error),
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Status(code) => write!(f, "node answered HTTP {code}"),
            Self::Malformed(what) => write!(f, "malformed node response: {what}"),
            Self::Cookie(error) | Self::Io(error) => error.fmt(f),
        }
    }
}

impl error::Error for HttpError {}

impl From<io::Error> for HttpError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

pub struct Http {
    host: String,
    port: u16,
    /// `Host` header value.
    authority: String,
    path: String,
    cookie: Option<PathBuf>,
    /// `Authorization` header value, refreshed when the node's cookie changes.
    authorization: RwLock<Option<String>>,
    idle: Mutex<Vec<Connection>>,
}

impl Http {
    /// Parses `http://host[:port][/path]` (port 80 when missing, IPv6 hosts in brackets) and
    /// reads the cookie, without connecting.
    pub fn new(url: &str, auth: &Auth) -> Result<Self> {
        let invalid = || Error::Internal("invalid node RPC URL");
        let rest = url
            .strip_prefix("http://")
            .ok_or(Error::Internal("node RPC requires an HTTP URL"))?;
        let (authority, path) = rest.find('/').map_or((rest, "/"), |at| rest.split_at(at));
        let (host, port) = match authority.strip_prefix('[') {
            Some(bracketed) => {
                let (host, rest) = bracketed.split_once(']').ok_or_else(invalid)?;
                let port = match rest {
                    "" => None,
                    rest => Some(rest.strip_prefix(':').ok_or_else(invalid)?),
                };
                (host, port)
            }
            None => match authority.split_once(':') {
                Some((host, port)) => (host, Some(port)),
                None => (authority, None),
            },
        };
        let port = port.map_or(Ok(80), str::parse).map_err(|_| invalid())?;
        if host.is_empty() {
            return Err(invalid());
        }
        let (cookie, authorization) = match auth {
            Auth::None => (None, None),
            Auth::UserPass(user, password) => (None, Some(basic(&format!("{user}:{password}")))),
            Auth::CookieFile(path) => (Some(path.clone()), Some(basic(&read_cookie(path)?))),
        };
        Ok(Self {
            host: host.to_owned(),
            port,
            authority: authority.to_owned(),
            path: path.to_owned(),
            cookie,
            authorization: RwLock::new(authorization),
            idle: Mutex::new(Vec::new()),
        })
    }

    fn post<T: DeserializeOwned>(&self, payload: &impl Serialize) -> StdResult<T, JsonRpcError> {
        let body = to_vec(payload)?;
        let (status, bytes) = self.exchange(&body).map_err(transport)?;
        match from_slice(&bytes) {
            Ok(response) => Ok(response),
            Err(_) if !(200..300).contains(&status) => Err(transport(HttpError::Status(status))),
            Err(error) => Err(error.into()),
        }
    }

    /// One request and its complete response. A 401 replays once when the node has written a
    /// new cookie since the rejected request was built: Bitcoin Core writes it before serving
    /// RPC, so an unchanged cookie means the rejection is real.
    fn exchange(&self, body: &[u8]) -> StdResult<(u16, Vec<u8>), HttpError> {
        let sent = self.authorization.read().clone();
        let answer = self.round_trip(&self.request(body, sent.as_deref()))?;
        if answer.0 != 401 {
            return Ok(answer);
        }
        let Some(path) = &self.cookie else {
            return Ok(answer);
        };
        let current = Some(basic(&read_cookie(path).map_err(HttpError::Cookie)?));
        if current == sent {
            return Ok(answer);
        }
        *self.authorization.write() = current.clone();
        self.round_trip(&self.request(body, current.as_deref()))
    }

    fn request(&self, body: &[u8], authorization: Option<&str>) -> Vec<u8> {
        let mut request = Vec::with_capacity(body.len() + 256);
        // Writing to a Vec cannot fail.
        let _ = write!(
            request,
            "POST {} HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n",
            self.path,
            self.authority,
            body.len()
        );
        if let Some(authorization) = authorization {
            let _ = write!(request, "Authorization: {authorization}\r\n");
        }
        request.extend_from_slice(b"\r\n");
        request.extend_from_slice(body);
        request
    }

    /// The node closes idle keep-alive sockets: a pooled socket that dies before any response
    /// byte replays once on a fresh one. Every call is read-only, so a replay is always safe.
    fn round_trip(&self, request: &[u8]) -> StdResult<(u16, Vec<u8>), HttpError> {
        let pooled = self.idle.lock().pop();
        if let Some(mut connection) = pooled {
            match connection.exchange(request) {
                Ok(answer) => return Ok(self.release(connection, answer)),
                Err(failure) if !failure.replayable => return Err(failure.error),
                Err(_) => {}
            }
        }
        let mut connection = self.connect()?;
        let answer = connection
            .exchange(request)
            .map_err(|failure| failure.error)?;
        Ok(self.release(connection, answer))
    }

    fn release(&self, connection: Connection, answer: Answer) -> (u16, Vec<u8>) {
        if answer.keep_alive {
            let mut idle = self.idle.lock();
            if idle.len() < MAX_IDLE {
                idle.push(connection);
            }
        }
        (answer.status, answer.body)
    }

    fn connect(&self) -> StdResult<Connection, HttpError> {
        let mut last = None;
        for addr in (self.host.as_str(), self.port).to_socket_addrs()? {
            match TcpStream::connect_timeout(&addr, CONNECT_TIMEOUT) {
                Ok(stream) => {
                    stream.set_nodelay(true)?;
                    stream.set_read_timeout(Some(IO_TIMEOUT))?;
                    stream.set_write_timeout(Some(IO_TIMEOUT))?;
                    return Ok(Connection(BufReader::new(stream)));
                }
                Err(error) => last = Some(error),
            }
        }
        Err(last
            .unwrap_or_else(|| io::Error::new(ErrorKind::NotFound, "node RPC host has no address"))
            .into())
    }
}

impl Transport for Http {
    fn send_request(&self, request: Request) -> StdResult<Response, JsonRpcError> {
        self.post(&request)
    }

    fn send_batch(&self, requests: &[Request]) -> StdResult<Vec<Response>, JsonRpcError> {
        self.post(&requests)
    }

    fn fmt_target(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "http://{}{}", self.authority, self.path)
    }
}

fn basic(credentials: &str) -> String {
    format!("Basic {}", STANDARD.encode(credentials))
}

fn transport(error: HttpError) -> JsonRpcError {
    JsonRpcError::Transport(Box::new(error))
}

struct Connection(BufReader<TcpStream>);

struct Answer {
    status: u16,
    body: Vec<u8>,
    keep_alive: bool,
}

struct Failure {
    error: HttpError,
    /// The socket was dead before the node answered (a write error, or EOF or a reset at the
    /// first read), so the node never handled the request. A timeout is not: it may be working.
    replayable: bool,
}

impl Connection {
    fn exchange(&mut self, request: &[u8]) -> StdResult<Answer, Failure> {
        let dead = |error: io::Error| Failure {
            replayable: !matches!(error.kind(), ErrorKind::TimedOut | ErrorKind::WouldBlock),
            error: error.into(),
        };
        self.0.get_mut().write_all(request).map_err(dead)?;
        let first = loop {
            match self.0.fill_buf() {
                Err(error) if error.kind() == ErrorKind::Interrupted => {}
                first => break first.map_err(dead)?,
            }
        };
        if first.is_empty() {
            return Err(dead(ErrorKind::UnexpectedEof.into()));
        }
        self.read_response().map_err(|error| Failure {
            error,
            replayable: false,
        })
    }

    fn read_response(&mut self) -> StdResult<Answer, HttpError> {
        let mut budget = MAX_HEAD_BYTES;
        let mut line = Vec::new();

        let status_line = self.read_line(&mut line, &mut budget)?;
        let (version, status) = status_line
            .strip_prefix(b"HTTP/1.")
            .and_then(|rest| rest.split_first())
            .filter(|(_, rest)| rest.first() == Some(&b' '))
            .and_then(|(&minor, rest)| Some((minor, parse_status(rest.get(1..4)?)?)))
            .ok_or(HttpError::Malformed("status line"))?;
        let mut keep_alive = version == b'1';
        let mut content_length = None;
        let mut chunked = false;

        loop {
            let header = self.read_line(&mut line, &mut budget)?;
            if header.is_empty() {
                break;
            }
            let colon = header
                .iter()
                .position(|&byte| byte == b':')
                .ok_or(HttpError::Malformed("header line"))?;
            let (name, value) = (&header[..colon], header[colon + 1..].trim_ascii());
            if name.eq_ignore_ascii_case(b"content-length") {
                let length = str::from_utf8(value)
                    .ok()
                    .and_then(|value| value.parse::<usize>().ok())
                    .ok_or(HttpError::Malformed("content-length"))?;
                if content_length.is_some_and(|known| known != length) {
                    return Err(HttpError::Malformed("conflicting content-length"));
                }
                content_length = Some(length);
            } else if name.eq_ignore_ascii_case(b"transfer-encoding") {
                if !value.eq_ignore_ascii_case(b"chunked") {
                    return Err(HttpError::Malformed("unsupported transfer-encoding"));
                }
                chunked = true;
            } else if name.eq_ignore_ascii_case(b"connection") {
                if value.eq_ignore_ascii_case(b"close") {
                    keep_alive = false;
                } else if value.eq_ignore_ascii_case(b"keep-alive") {
                    keep_alive = true;
                }
            }
        }

        let body = if chunked {
            self.read_chunked()?
        } else if let Some(length) = content_length {
            let mut body = Vec::with_capacity(length.min(MAX_PREALLOC));
            (&mut self.0).take(length as u64).read_to_end(&mut body)?;
            if body.len() != length {
                return Err(io::Error::from(ErrorKind::UnexpectedEof).into());
            }
            body
        } else {
            // Close-delimited: the body ends with the connection.
            keep_alive = false;
            let mut body = Vec::new();
            self.0.read_to_end(&mut body)?;
            body
        };

        Ok(Answer {
            status,
            body,
            keep_alive,
        })
    }

    fn read_chunked(&mut self) -> StdResult<Vec<u8>, HttpError> {
        let mut body = Vec::new();
        let mut line = Vec::new();
        loop {
            let mut budget = MAX_LINE_BYTES;
            let size_line = self.read_line(&mut line, &mut budget)?;
            let digits = size_line
                .split(|&byte| byte == b';')
                .next()
                .unwrap_or_default()
                .trim_ascii();
            let size = str::from_utf8(digits)
                .ok()
                .and_then(|digits| usize::from_str_radix(digits, 16).ok())
                .ok_or(HttpError::Malformed("chunk size"))?;
            if size == 0 {
                // Trailer fields, then the final empty line.
                let mut budget = MAX_HEAD_BYTES;
                while !self.read_line(&mut line, &mut budget)?.is_empty() {}
                return Ok(body);
            }
            let start = body.len();
            (&mut self.0).take(size as u64).read_to_end(&mut body)?;
            if body.len() - start != size {
                return Err(io::Error::from(ErrorKind::UnexpectedEof).into());
            }
            let mut budget = 2;
            if !self.read_line(&mut line, &mut budget)?.is_empty() {
                return Err(HttpError::Malformed("chunk terminator"));
            }
        }
    }

    /// One line without its CRLF (or bare LF), within `budget` bytes. EOF before the line ends is
    /// an error: the response is incomplete.
    fn read_line<'a>(
        &mut self,
        line: &'a mut Vec<u8>,
        budget: &mut usize,
    ) -> StdResult<&'a [u8], HttpError> {
        line.clear();
        let read = (&mut self.0).take(*budget as u64).read_until(b'\n', line)?;
        *budget -= read;
        if line.last() != Some(&b'\n') {
            return Err(if *budget == 0 {
                HttpError::Malformed("line too long")
            } else {
                io::Error::from(ErrorKind::UnexpectedEof).into()
            });
        }
        line.pop();
        if line.last() == Some(&b'\r') {
            line.pop();
        }
        Ok(line)
    }
}

fn parse_status(digits: &[u8]) -> Option<u16> {
    digits.iter().try_fold(0, |status: u16, &digit| {
        digit
            .is_ascii_digit()
            .then(|| status * 10 + u16::from(digit - b'0'))
    })
}
