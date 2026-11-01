use std::cell::RefCell;

use ffb_commands::{CommandError, Session};
use ffb_pattern::Garment;
use serde_json::{json, Value};

thread_local! {
    static SESSION: RefCell<Session> = RefCell::new(Session::default());
}

#[no_mangle]
pub extern "C" fn ffb_alloc(len: usize) -> *mut u8 {
    Box::into_raw(vec![0u8; len.max(1)].into_boxed_slice()) as *mut u8
}

/// # Safety
/// `ptr` and `len` must be a pair returned by `ffb_alloc`, or a reply pointer with its 4 byte length prefix counted, and not yet freed.
#[no_mangle]
pub unsafe extern "C" fn ffb_free(ptr: *mut u8, len: usize) {
    drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(ptr, len.max(1))));
}

unsafe fn text<'a>(ptr: *const u8, len: usize) -> &'a str {
    if len == 0 {
        return "";
    }
    std::str::from_utf8(std::slice::from_raw_parts(ptr, len)).unwrap_or("")
}

fn emit(value: Value) -> *mut u8 {
    let body = value.to_string().into_bytes();
    let mut out = Vec::with_capacity(body.len() + 4);
    out.extend_from_slice(&(body.len() as u32).to_le_bytes());
    out.extend_from_slice(&body);
    Box::into_raw(out.into_boxed_slice()) as *mut u8
}

fn bad(message: String) -> Value {
    ffb_commands::reply(Err(CommandError::new("bad_args", message)))
}

/// # Safety
/// `ptr` and `len` must describe UTF-8 JSON the page wrote into memory from `ffb_alloc`.
#[no_mangle]
pub unsafe extern "C" fn ffb_dispatch(ptr: *const u8, len: usize) -> *mut u8 {
    let request: Result<Value, _> = serde_json::from_str(text(ptr, len));
    let out = match request {
        Ok(request) => {
            let name = request.get("name").and_then(Value::as_str).unwrap_or("");
            let args = request.get("args").cloned().unwrap_or(Value::Null);
            SESSION.with(|s| ffb_commands::reply(ffb_commands::dispatch(&mut s.borrow_mut(), name, &args)))
        }
        Err(e) => bad(format!("request is not JSON: {e}")),
    };
    emit(out)
}

/// # Safety
/// As `ffb_dispatch`; the JSON is a full `pattern.get` result, bare or inside its reply envelope.
#[no_mangle]
pub unsafe extern "C" fn ffb_load(ptr: *const u8, len: usize) -> *mut u8 {
    let parsed: Result<Value, _> = serde_json::from_str(text(ptr, len));
    let out = match parsed {
        Ok(mut doc) => {
            if let Some(inner) = doc.get_mut("result").map(Value::take) {
                doc = inner;
            }
            match serde_json::from_value::<Garment>(doc) {
                Ok(garment) => {
                    let panels = garment.panels.len();
                    SESSION.with(|s| s.borrow_mut().reset(garment));
                    ffb_commands::reply(Ok(json!({"panels": panels})))
                }
                Err(e) => bad(format!("not a garment: {e}")),
            }
        }
        Err(e) => bad(format!("document is not JSON: {e}")),
    };
    emit(out)
}
