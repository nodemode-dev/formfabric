use std::collections::{BTreeMap, HashMap};
use std::io::{self, ErrorKind};
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use tungstenite::handshake::server::{ErrorResponse, Request, Response};
use tungstenite::http::StatusCode;
use tungstenite::{Error as WsError, Message};

use crate::frame;

const AUTH_WINDOW: Duration = Duration::from_secs(5);
const POLL: Duration = Duration::from_millis(4);
const SEND_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_UNAUTH: usize = 8;
const MAX_CLIENTS: usize = 32;

type Bytes = Arc<Vec<u8>>;

#[derive(Default)]
struct Slot {
    topology: Vec<(u32, Bytes)>,
    positions: BTreeMap<u32, Bytes>,
    scalars: BTreeMap<u32, Bytes>,
}

#[derive(Default)]
struct Hub {
    clients: Vec<Arc<Mutex<Slot>>>,
    topology: BTreeMap<u32, Bytes>,
    positions: BTreeMap<u32, Bytes>,
    scalars: BTreeMap<u32, Bytes>,
    frames: HashMap<u32, u32>,
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

#[derive(Clone)]
pub struct Publisher {
    hub: Arc<Mutex<Hub>>,
}

impl Publisher {
    pub fn publish_positions(&self, object: u32, positions: &[f32]) {
        let mut hub = lock(&self.hub);
        let n = hub.frames.entry(object).or_insert(0);
        *n = n.wrapping_add(1);
        let bytes = Arc::new(frame::encode_positions(*n, object, positions));
        hub.positions.insert(object, bytes.clone());
        hub.clients.retain(|c| Arc::strong_count(c) > 1);
        for c in &hub.clients {
            lock(c).positions.insert(object, bytes.clone());
        }
    }

    pub fn publish_scalars(&self, object: u32, map: u32, component: u32, min: f32, max: f32, values: &[f32]) {
        let mut hub = lock(&self.hub);
        let n = hub.frames.entry(object).or_insert(0);
        *n = n.wrapping_add(1);
        let bytes = Arc::new(frame::encode_scalars(*n, object, map, component, min, max, values));
        if values.is_empty() {
            hub.scalars.remove(&object);
        } else {
            hub.scalars.insert(object, bytes.clone());
        }
        hub.clients.retain(|c| Arc::strong_count(c) > 1);
        for c in &hub.clients {
            lock(c).scalars.insert(object, bytes.clone());
        }
    }

    pub fn publish_topology(&self, object: u32, indices: &[u32], panel_ids: &[u16]) {
        let vertices = indices.iter().max().map_or(0, |m| m + 1);
        let mut hub = lock(&self.hub);
        let n = hub.frames.entry(object).or_insert(0);
        *n = n.wrapping_add(1);
        let bytes = Arc::new(frame::encode_topology(*n, object, vertices, indices, panel_ids));
        hub.topology.insert(object, bytes.clone());
        hub.positions.remove(&object);
        hub.scalars.remove(&object);
        hub.clients.retain(|c| Arc::strong_count(c) > 1);
        for c in &hub.clients {
            let mut slot = lock(c);
            slot.positions.remove(&object);
            slot.scalars.remove(&object);
            slot.topology.retain(|(o, _)| *o != object);
            slot.topology.push((object, bytes.clone()));
        }
    }

    fn join(&self) -> Arc<Mutex<Slot>> {
        let mut hub = lock(&self.hub);
        let slot = Slot {
            topology: hub.topology.iter().map(|(o, b)| (*o, b.clone())).collect(),
            positions: hub.positions.clone(),
            scalars: hub.scalars.clone(),
        };
        let slot = Arc::new(Mutex::new(slot));
        hub.clients.push(slot.clone());
        slot
    }
}

pub struct Server {
    port: u16,
    token: String,
    publisher: Publisher,
    stop: Arc<AtomicBool>,
}

fn make_token() -> io::Result<String> {
    let mut raw = [0u8; 24];
    getrandom::fill(&mut raw).map_err(|e| io::Error::other(e.to_string()))?;
    Ok(raw.iter().map(|b| format!("{b:02x}")).collect())
}

fn same(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    let mut diff = a.len() ^ b.len();
    for i in 0..a.len().min(b.len()) {
        diff |= (a[i] ^ b[i]) as usize;
    }
    diff == 0
}

#[derive(Clone, Default)]
struct Seats {
    total: Arc<AtomicUsize>,
    unauth: Arc<AtomicUsize>,
}

struct Seat {
    total: Arc<AtomicUsize>,
    unauth: Arc<AtomicUsize>,
    pending: bool,
}

impl Seats {
    fn take(&self) -> Option<Seat> {
        if self.total.fetch_add(1, Ordering::AcqRel) >= MAX_CLIENTS {
            self.total.fetch_sub(1, Ordering::AcqRel);
            return None;
        }
        if self.unauth.fetch_add(1, Ordering::AcqRel) >= MAX_UNAUTH {
            self.unauth.fetch_sub(1, Ordering::AcqRel);
            self.total.fetch_sub(1, Ordering::AcqRel);
            return None;
        }
        Some(Seat { total: self.total.clone(), unauth: self.unauth.clone(), pending: true })
    }
}

impl Seat {
    fn authenticated(&mut self) {
        if self.pending {
            self.pending = false;
            self.unauth.fetch_sub(1, Ordering::AcqRel);
        }
    }
}

impl Drop for Seat {
    fn drop(&mut self) {
        self.authenticated();
        self.total.fetch_sub(1, Ordering::AcqRel);
    }
}

fn query_token(query: &str) -> Option<&str> {
    query.split('&').find_map(|kv| kv.strip_prefix("token="))
}

impl Server {
    pub fn start() -> io::Result<Server> {
        let listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))?;
        let port = listener.local_addr()?.port();
        let token = make_token()?;
        let publisher = Publisher { hub: Arc::new(Mutex::new(Hub::default())) };
        let stop = Arc::new(AtomicBool::new(false));
        let accept_pub = publisher.clone();
        let accept_token = token.clone();
        let accept_stop = stop.clone();
        let seats = Seats::default();
        thread::Builder::new().name("ffb-stream-accept".into()).spawn(move || {
            for conn in listener.incoming() {
                if accept_stop.load(Ordering::Relaxed) {
                    break;
                }
                let Ok(stream) = conn else { continue };
                let Some(seat) = seats.take() else { continue };
                let publisher = accept_pub.clone();
                let token = accept_token.clone();
                let _ = thread::Builder::new()
                    .name("ffb-stream-client".into())
                    .spawn(move || serve(stream, publisher, &token, seat));
            }
        })?;
        Ok(Server { port, token, publisher, stop })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn publisher(&self) -> Publisher {
        self.publisher.clone()
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        let _ = TcpStream::connect(SocketAddr::from((Ipv4Addr::LOCALHOST, self.port)));
    }
}

#[allow(clippy::result_large_err)]
fn serve(stream: TcpStream, publisher: Publisher, token: &str, mut seat: Seat) {
    let loopback = stream.peer_addr().map(|a| a.ip().is_loopback()).unwrap_or(false);
    if !loopback {
        return;
    }
    let _ = stream.set_nodelay(true);
    let _ = stream.set_read_timeout(Some(AUTH_WINDOW));
    let _ = stream.set_write_timeout(Some(SEND_TIMEOUT));
    let mut query_ok = false;
    let mut bad_query = false;
    let callback = |req: &Request, resp: Response| -> Result<Response, ErrorResponse> {
        if let Some(t) = req.uri().query().and_then(query_token) {
            if same(t, token) {
                query_ok = true;
            } else {
                bad_query = true;
            }
        }
        if bad_query {
            let mut denied = ErrorResponse::new(Some("denied".into()));
            *denied.status_mut() = StatusCode::UNAUTHORIZED;
            return Err(denied);
        }
        Ok(resp)
    };
    let Ok(mut ws) = tungstenite::accept_hdr(stream, callback) else { return };

    if !query_ok {
        let started = Instant::now();
        loop {
            match ws.read() {
                Ok(Message::Text(t)) if same(t.as_str(), token) => break,
                Ok(Message::Ping(_)) | Ok(Message::Pong(_)) if started.elapsed() < AUTH_WINDOW => continue,
                _ => {
                    let _ = ws.close(None);
                    let _ = ws.flush();
                    return;
                }
            }
        }
    }

    seat.authenticated();
    let slot = publisher.join();
    let _ = ws.get_ref().set_read_timeout(Some(POLL));
    loop {
        match ws.read() {
            Ok(Message::Close(_)) => break,
            Ok(_) => {}
            Err(WsError::Io(e)) if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {}
            Err(_) => break,
        }
        let batch: Vec<Bytes> = {
            let mut s = lock(&slot);
            let mut out: Vec<Bytes> = s.topology.drain(..).map(|(_, b)| b).collect();
            out.extend(std::mem::take(&mut s.positions).into_values());
            out.extend(std::mem::take(&mut s.scalars).into_values());
            out
        };
        for bytes in batch {
            if ws.send(Message::Binary(bytes.as_slice().to_vec().into())).is_err() {
                return;
            }
        }
    }
}
