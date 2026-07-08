use serde::{de::DeserializeOwned, Serialize};

// Minimal URL percent-encode/decode so the cached JSON is always a safe
// cookie value ([A-Za-z0-9%-] only). Works on both wasm and desktop targets.
pub fn url_encode(s: &str) -> String {
    const SAFE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_.~";
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        if SAFE.contains(&b) {
            out.push(b as char);
        } else {
            out.push('%');
            out.push(char::from_digit((b >> 4) as u32, 16).unwrap().to_ascii_uppercase());
            out.push(char::from_digit((b & 0x0f) as u32, 16).unwrap().to_ascii_uppercase());
        }
    }
    out
}

pub fn url_decode(s: &str) -> Option<String> {
    let bytes = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(s.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hi = char::from(bytes[i + 1]).to_digit(16)?;
            let lo = char::from(bytes[i + 2]).to_digit(16)?;
            out.push(((hi << 4) | lo) as u8);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

pub fn set_json_cookie<T: Serialize>(name: &str, value: &T, max_age: u32) {
    if let Ok(json) = serde_json::to_string(value) {
        let enc = url_encode(&json);
        crate::set_cookie!(name, enc, max_age);
    }
}

pub fn get_json_cookie<T: DeserializeOwned>(name: &str) -> Option<T> {
    let raw = crate::get_cookie!(name)?;
    if raw.is_empty() {
        return None;
    }
    let json = url_decode(&raw)?;
    serde_json::from_str(&json).ok()
}
