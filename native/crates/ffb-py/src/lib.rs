use std::sync::{Arc, Mutex, OnceLock};

use ffb_commands::{FrameSink, Session};
use ffb_stream::{Publisher, Server};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

fn guarded(f: impl FnOnce() -> String) -> String {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(text) => text,
        Err(payload) => {
            let detail = payload.downcast_ref::<&str>().map(|s| s.to_string()).or_else(|| payload.downcast_ref::<String>().cloned()).unwrap_or_else(|| "unknown panic".into());
            serde_json::json!({"ok": false, "error": {"code": "internal", "message": format!("the engine hit an internal error: {detail}")}}).to_string()
        }
    }
}

struct PublisherSink(Publisher);

impl FrameSink for PublisherSink {
    fn topology(&self, object: u32, indices: &[u32], panel_ids: &[u16]) {
        self.0.publish_topology(object, indices, panel_ids);
    }

    fn positions(&self, object: u32, positions: &[f32]) {
        self.0.publish_positions(object, positions);
    }

    fn scalars(&self, object: u32, map: u32, component: u32, min: f32, max: f32, values: &[f32]) {
        self.0.publish_scalars(object, map, component, min, max, values);
    }
}

#[pyclass]
struct Engine {
    session: Mutex<Session>,
    stream: OnceLock<Server>,
}

impl Engine {
    fn server(&self) -> PyResult<&Server> {
        if let Some(s) = self.stream.get() {
            return Ok(s);
        }
        let started = Server::start().map_err(|e| PyValueError::new_err(format!("stream server failed to start: {e}")))?;
        let mut fresh = false;
        let server = self.stream.get_or_init(|| {
            fresh = true;
            started
        });
        if fresh {
            if let Ok(mut session) = self.session.lock() {
                session.sink = Some(Arc::new(PublisherSink(server.publisher())));
            }
        }
        Ok(server)
    }
}

#[pymethods]
impl Engine {
    #[new]
    fn new() -> Self {
        Engine { session: Mutex::new(Session::default()), stream: OnceLock::new() }
    }

    fn dispatch(&self, name: &str, args_json: &str) -> PyResult<String> {
        let mut session = self.session.lock().unwrap_or_else(|e| e.into_inner());
        Ok(guarded(|| ffb_commands::dispatch_json(&mut session, name, args_json).to_string()))
    }

    fn epoch(&self) -> PyResult<u64> {
        let session = self.session.lock().map_err(|_| PyValueError::new_err("engine session poisoned"))?;
        Ok(session.epoch)
    }

    fn stream_info(&self) -> PyResult<String> {
        let server = self.server()?;
        let mut session = self.session.lock().unwrap_or_else(|e| e.into_inner());
        Ok(guarded(|| {
            ffb_commands::publish_all(&mut session);
            serde_json::json!({"port": server.port(), "token": server.token()}).to_string()
        }))
    }

    fn stream_demo(&self, n_frames: u32) -> PyResult<String> {
        let server = self.server()?;
        Ok(guarded(|| {
            ffb_stream::demo::spawn(server.publisher(), n_frames);
            serde_json::json!({"ok": true}).to_string()
        }))
    }

    fn crash_test(&self, mode: &str) -> PyResult<String> {
        if std::env::var_os("FFB_CRASH_TEST").is_none() {
            return Err(PyValueError::new_err("crash_test is off"));
        }
        let _hold = self.session.lock().unwrap_or_else(|e| e.into_inner());
        Ok(guarded(|| {
            if mode == "panic" {
                panic!("forced panic for the hardening check");
            }
            serde_json::json!({"ok": true}).to_string()
        }))
    }
}

#[pyfunction]
fn catalogue() -> String {
    ffb_commands::catalogue().to_string()
}

#[pyfunction]
fn version() -> &'static str {
    ffb_core::ENGINE_VERSION
}

#[pymodule]
fn ffb_engine(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Engine>()?;
    m.add_function(wrap_pyfunction!(catalogue, m)?)?;
    m.add_function(wrap_pyfunction!(version, m)?)?;
    Ok(())
}
