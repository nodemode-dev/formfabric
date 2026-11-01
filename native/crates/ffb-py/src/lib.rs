use std::sync::{Mutex, OnceLock};

use ffb_commands::Session;
use ffb_stream::Server;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

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
        Ok(self.stream.get_or_init(|| started))
    }
}

#[pymethods]
impl Engine {
    #[new]
    fn new() -> Self {
        Engine { session: Mutex::new(Session::default()), stream: OnceLock::new() }
    }

    fn dispatch(&self, name: &str, args_json: &str) -> PyResult<String> {
        let mut session = self.session.lock().map_err(|_| PyValueError::new_err("engine session poisoned"))?;
        Ok(ffb_commands::dispatch_json(&mut session, name, args_json).to_string())
    }

    fn stream_info(&self) -> PyResult<String> {
        let server = self.server()?;
        Ok(serde_json::json!({"port": server.port(), "token": server.token()}).to_string())
    }

    fn stream_demo(&self, n_frames: u32) -> PyResult<()> {
        ffb_stream::demo::spawn(self.server()?.publisher(), n_frames);
        Ok(())
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
