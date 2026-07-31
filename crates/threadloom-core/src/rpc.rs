#[cfg(target_arch = "wasm32")]
pub async fn client_rpc_call<T: serde::de::DeserializeOwned>(
    url: &str,
    body: serde_json::Value,
) -> Result<T, String> {
    use wasm_bindgen::JsCast;

    let mut opts = web_sys::RequestInit::new();
    opts.method("POST");
    opts.mode(web_sys::RequestMode::Cors);

    let js_body = wasm_bindgen::JsValue::from_str(&body.to_string());
    opts.body(Some(&js_body));

    let headers = web_sys::Headers::new().map_err(|e| format!("Headers::new failed: {:?}", e))?;
    headers
        .set("Content-Type", "application/json")
        .map_err(|e| format!("set Content-Type failed: {:?}", e))?;
    headers
        .set("x-threadloom-route", url)
        .map_err(|e| format!("set x-threadloom-route failed: {:?}", e))?;
    opts.headers(&headers);

    let request = web_sys::Request::new_with_str_and_init(url, &opts)
        .map_err(|e| format!("Request::new failed: {:?}", e))?;

    let window = web_sys::window().ok_or_else(|| "no window".to_string())?;
    let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("fetch failed: {:?}", e))?;
    let resp: web_sys::Response = resp_value
        .dyn_into()
        .map_err(|e| format!("cast to Response failed: {:?}", e))?;

    if !resp.ok() {
        let status = resp.status();
        let err_text = match resp.text() {
            Ok(promise) => match wasm_bindgen_futures::JsFuture::from(promise).await {
                Ok(js_val) => js_val.as_string().unwrap_or_default(),
                Err(_) => "Could not read response text".to_string(),
            },
            Err(_) => "Could not read response text promise".to_string(),
        };
        return Err(format!("server returned HTTP {}: {}", status, err_text));
    }

    let text_promise = resp
        .text()
        .map_err(|e| format!("resp.text() failed: {:?}", e))?;
    let text_val = wasm_bindgen_futures::JsFuture::from(text_promise)
        .await
        .map_err(|e| format!("reading body failed: {:?}", e))?;
    let text = text_val
        .as_string()
        .ok_or_else(|| "body is not a string".to_string())?;

    serde_json::from_str(&text)
        .map_err(|e| format!("deserialize failed: {} | body was: {}", e, text))
}
