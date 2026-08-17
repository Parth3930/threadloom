
#[macro_export]
macro_rules! get_value {
    ($id:expr) => {{
        let mut val = String::new();
        if let Some(w) = $crate::web_sys::window() {
            if let Some(d) = w.document() {
                if let Some(el) = d.get_element_by_id($id) {
                    use $crate::wasm_bindgen::JsCast;
                    if let Ok(input_el) = el.clone().dyn_into::<$crate::web_sys::HtmlInputElement>()
                    {
                        val = input_el.value();
                    } else if let Ok(textarea_el) = el
                        .clone()
                        .dyn_into::<$crate::web_sys::HtmlTextAreaElement>()
                    {
                        val = textarea_el.value();
                    } else if let Ok(select_el) =
                        el.dyn_into::<$crate::web_sys::HtmlSelectElement>()
                    {
                        val = select_el.value();
                    }
                }
            }
        }
        val
    }};
}

#[macro_export]
macro_rules! spawn {
    ($fut:expr) => {
        $crate::wasm_bindgen_futures::spawn_local(async move {
            $fut.await;
            let _ = $crate::tick();
        });
    };
}

#[macro_export]
macro_rules! fetch {
    // With body
    ($method:ident $url:expr, $body:expr => |$text:ident| $success:block) => {
        $crate::wasm_bindgen_futures::spawn_local(async move {
            if let Ok(resp) = $crate::reqwasm::http::Request::$method($url).header("Content-Type", "application/json").body($body).send().await {
                if let Ok($text) = resp.text().await {
                    $success
                    let _ = $crate::tick();
                }
            }
        });
    };
    ($method:ident $url:expr, $body:expr => |$text:ident| $success:block, |$err:ident| $error:block) => {
        $crate::wasm_bindgen_futures::spawn_local(async move {
            match $crate::reqwasm::http::Request::$method($url).header("Content-Type", "application/json").body($body).send().await {
                Ok(resp) => {
                    match resp.text().await {
                        Ok($text) => {
                            $success
                            let _ = $crate::tick();
                        }
                        Err(e) => {
                            let $err = format!("Parse error: {:?}", e);
                            $error
                            let _ = $crate::tick();
                        }
                    }
                }
                Err(e) => {
                    let $err = format!("Fetch error: {:?}", e);
                    $error
                    let _ = $crate::tick();
                }
            }
        });
    };

    // Without body
    ($method:ident $url:expr => |$text:ident| $success:block) => {
        $crate::wasm_bindgen_futures::spawn_local(async move {
            if let Ok(resp) = $crate::reqwasm::http::Request::$method($url).send().await {
                if let Ok($text) = resp.text().await {
                    $success
                    let _ = $crate::tick();
                }
            }
        });
    };
    ($method:ident $url:expr => |$text:ident| $success:block, |$err:ident| $error:block) => {
        $crate::wasm_bindgen_futures::spawn_local(async move {
            match $crate::reqwasm::http::Request::$method($url).send().await {
                Ok(resp) => {
                    match resp.text().await {
                        Ok($text) => {
                            $success
                            let _ = $crate::tick();
                        }
                        Err(e) => {
                            let $err = format!("Parse error: {:?}", e);
                            $error
                            let _ = $crate::tick();
                        }
                    }
                }
                Err(e) => {
                    let $err = format!("Fetch error: {:?}", e);
                    $error
                    let _ = $crate::tick();
                }
            }
        });
    };

    // Default GET
    ($url:expr => |$text:ident| $success:block) => {
        $crate::fetch!(get $url => |$text| $success)
    };
    ($url:expr => |$text:ident| $success:block, |$err:ident| $error:block) => {
        $crate::fetch!(get $url => |$text| $success, |$err| $error)
    };
}

#[macro_export]
macro_rules! rpc {
    ($call:expr => |$ok:ident| $success:block) => {
        $crate::spawn!(async move {
            if let Ok($ok) = $call.await {
                $success
            }
        });
    };
    ($call:expr => |$ok:ident| $success:block, |$err:ident| $error:block) => {
        $crate::spawn!(async move {
            match $call.await {
                Ok($ok) => $success,
                Err($err) => $error,
            }
        });
    };
}

#[macro_export]
macro_rules! alert {
    ($msg:expr) => {
        if let Some(window) = $crate::web_sys::window() {
            let _ = window.alert_with_message($msg);
        }
    };
}

#[macro_export]
macro_rules! log {
    ($($t:tt)*) => {
        $crate::web_sys::console::log_1(&format!($($t)*).into());
    }
}

// ponytail: keep it simple. use max-age for expiration, no complex date parsing.
#[macro_export]
macro_rules! get_cookie {
    () => {{
        let mut cookie_string = String::new();
        if let Some(window) = $crate::web_sys::window() {
            if let Some(document) = window.document() {
                use $crate::wasm_bindgen::JsCast;
                if let Ok(html_doc) = document.dyn_into::<$crate::web_sys::HtmlDocument>() {
                    if let Ok(c) = html_doc.cookie() {
                        cookie_string = c;
                    }
                }
            }
        }
        cookie_string
    }};
    ($name:expr) => {{
        let cookies = $crate::get_cookie!();
        let name = $name;
        let mut result = None;
        for c in cookies.split(';') {
            let c = c.trim();
            if c.starts_with(name) && c[name.len()..].starts_with('=') {
                result = Some(c[name.len() + 1..].to_string());
                break;
            }
        }
        result
    }};
}

#[macro_export]
macro_rules! set_cookie {
    ($name:expr, $value:expr) => {
        if let Some(window) = $crate::web_sys::window() {
            if let Some(document) = window.document() {
                use $crate::wasm_bindgen::JsCast;
                if let Ok(html_doc) = document.dyn_into::<$crate::web_sys::HtmlDocument>() {
                    let cookie_str = format!("{}={}; path=/", $name, $value);
                    let _ = html_doc.set_cookie(&cookie_str);
                }
            }
        }
    };
    ($name:expr, $value:expr, $max_age:expr) => {
        if let Some(window) = $crate::web_sys::window() {
            if let Some(document) = window.document() {
                use $crate::wasm_bindgen::JsCast;
                if let Ok(html_doc) = document.dyn_into::<$crate::web_sys::HtmlDocument>() {
                    let cookie_str = format!("{}={}; max-age={}; path=/", $name, $value, $max_age);
                    let _ = html_doc.set_cookie(&cookie_str);
                }
            }
        }
    };
}

#[macro_export]
macro_rules! navigate {
    ($path:expr) => {
        if let Some(window) = $crate::web_sys::window() {
            let _ = window.history().unwrap().push_state_with_url(
                &$crate::wasm_bindgen::JsValue::NULL,
                "",
                Some($path),
            );
            let path_str = $path;
            let route = path_str.split(['?', '#']).next().unwrap_or(path_str);
            $crate::ROUTER_SETTER.with(|s| {
                if let Some(setter) = *s.borrow() {
                    setter.set(route.to_string());
                }
            });
            let _ = $crate::tick();
            window.scroll_to_with_x_and_y(0.0, 0.0);
        }
    };
}

#[macro_export]
macro_rules! animate {
    ($selector:expr, $config:expr) => {
        if let Some(_) = $crate::web_sys::window() {
            let script = format!(
                "if (window.gsap) {{ gsap.to('{}', {}) }}",
                $selector, $config
            );
            if let Err(e) = $crate::js_sys::eval(&script) {
                $crate::web_sys::console::error_2(&"GSAP animate! error:".into(), &e);
            }
        }
    };
    (from $selector:expr, $config:expr) => {
        if let Some(_) = $crate::web_sys::window() {
            let script = format!(
                "if (window.gsap) {{ gsap.from('{}', {}) }}",
                $selector, $config
            );
            if let Err(e) = $crate::js_sys::eval(&script) {
                $crate::web_sys::console::error_2(&"GSAP animate! from error:".into(), &e);
            }
        }
    };
    (fromTo $selector:expr, $from:expr, $to:expr) => {
        if let Some(_) = $crate::web_sys::window() {
            let script = format!(
                "if (window.gsap) {{ gsap.fromTo('{}', {}, {}) }}",
                $selector, $from, $to
            );
            if let Err(e) = $crate::js_sys::eval(&script) {
                $crate::web_sys::console::error_2(&"GSAP animate! fromTo error:".into(), &e);
            }
        }
    };
    (timeline $script:expr) => {
        if let Some(_) = $crate::web_sys::window() {
            let script = format!(
                "if (window.gsap) {{ let tl = gsap.timeline(); {} }}",
                $script
            );
            if let Err(e) = $crate::js_sys::eval(&script) {
                $crate::web_sys::console::error_2(&"GSAP animate! timeline error:".into(), &e);
            }
        }
    };
}

#[macro_export]
macro_rules! redirect {
    ($url:expr) => {
        {
            let target: &str = $url;
            if target.starts_with('/') && !target.starts_with("//") {
                $crate::navigate!(target);
            } else if let Some(w) = $crate::web_sys::window() {
                let _ = w.location().assign(target);
            }
        }
    };
}

#[macro_export]
macro_rules! back {
    () => {
        if let Some(w) = $crate::web_sys::window() {
            if let Ok(h) = w.history() {
                let _ = h.back();
            }
        }
    };
}
