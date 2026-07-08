use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{MessageEvent, WebSocket};

pub struct WsClient {
    pub ws: WebSocket,
}

impl WsClient {
    pub fn new(url: &str) -> Result<Self, JsValue> {
        let ws = WebSocket::new(url)?;
        Ok(Self { ws })
    }

    pub fn on_message<F>(&self, mut callback: F) 
    where
        F: FnMut(String) + 'static,
    {
        let closure = Closure::wrap(Box::new(move |e: MessageEvent| {
            if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                let s = String::from(txt);
                callback(s);
                let _ = crate::tick();
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        self.ws.set_onmessage(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    pub fn send(&self, msg: &str) -> Result<(), JsValue> {
        self.ws.send_with_str(msg)
    }
}
