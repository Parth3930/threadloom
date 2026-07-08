use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use threadloom_core::ReadSignal;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, MessageEvent, MouseEvent, WebSocket};


struct CursorAnimState {
    last_x: f64,
    last_y: f64,
    last_time: f64,
    prev_angle: f64,
    acc_rotation: f64,
}

fn get_or_create_cursor(uid: &str, is_local: bool, doc: &web_sys::Document) -> web_sys::Element {
    if let Some(el) = doc.get_element_by_id(&format!("cursor-{}", uid)) {
        return el;
    }
    
    let div = doc.create_element("div").unwrap();
    div.set_id(&format!("cursor-{}", uid));
    div.set_class_name("absolute pointer-events-none z-50");
    let _ = div.set_attribute("style", "top: 0; left: 0; margin-top: -3px; margin-left: -12.5px; opacity: 0;");
    
    let color = if is_local { "black" } else { "#3b82f6" };
    
    let svg_str = format!(r#"
    <svg xmlns="http://www.w3.org/2000/svg" width="25" height="27" viewBox="0 0 50 54" fill="none">
      <g filter="url(#filter0_d_91_{uid})">
        <path d="M42.6817 41.1495L27.5103 6.79925C26.7269 5.02557 24.2082 5.02558 23.3927 6.79925L7.59814 41.1495C6.75833 42.9759 8.52712 44.8902 10.4125 44.1954L24.3757 39.0496C24.8829 38.8627 25.4385 38.8627 25.9422 39.0496L39.8121 44.1954C41.6849 44.8902 43.4884 42.9759 42.6817 41.1495Z" fill="{color}" />
        <path d="M43.7146 40.6933L28.5431 6.34306C27.3556 3.65428 23.5772 3.69516 22.3668 6.32755L6.57226 40.6778C5.3134 43.4156 7.97238 46.298 10.803 45.2549L24.7662 40.109C25.0221 40.0147 25.2999 40.0156 25.5494 40.1082L39.4193 45.254C42.2261 46.2953 44.9254 43.4347 43.7146 40.6933Z" stroke="white" stroke-width="2.25825" />
      </g>
      <defs>
        <filter id="filter0_d_91_{uid}" x="0.602397" y="0.952444" width="49.0584" height="52.428" filterUnits="userSpaceOnUse" color-interpolation-filters="sRGB">
          <feFlood flood-opacity="0" result="BackgroundImageFix" />
          <feColorMatrix in="SourceAlpha" type="matrix" values="0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 127 0" result="hardAlpha" />
          <feOffset dy="2.25825" />
          <feGaussianBlur stdDeviation="2.25825" />
          <feComposite in2="hardAlpha" operator="out" />
          <feColorMatrix type="matrix" values="0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0.08 0" />
          <feBlend mode="normal" in2="BackgroundImageFix" result="effect1_dropShadow_91_7928" />
          <feBlend mode="normal" in="SourceGraphic" in2="effect1_dropShadow_91_7928" result="shape" />
        </filter>
      </defs>
    </svg>
    "#);
    div.set_inner_html(&svg_str);
    
    if let Some(parent) = doc.get_element_by_id("board-canvas").and_then(|el| el.parent_element()) {
        let _ = parent.append_child(&div);
    }
    
    let script = format!("if (window.gsap) {{ gsap.set('#cursor-{}', {{ transformOrigin: '12.5px 3px' }}); }}", uid);
    let _ = js_sys::eval(&script);
    
    div
}

pub fn init_board(tool_sig: ReadSignal<String>, color_sig: ReadSignal<String>) {
    wasm_bindgen_futures::spawn_local(async move {
        if let Err(e) = init_board_inner(tool_sig, color_sig).await {
            web_sys::console::error_1(&format!("Board init error: {:?}", e).into());
        }
    });
}

async fn init_board_inner(
    tool_sig: ReadSignal<String>,
    color_sig: ReadSignal<String>,
) -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let mut canvas_opt = None;
    for _ in 0..200 {
        if let Some(el) = document.get_element_by_id("board-canvas") {
            canvas_opt = Some(el.dyn_into::<HtmlCanvasElement>().unwrap());
            break;
        }
        let promise = js_sys::Promise::new(&mut |resolve, _| {
            window
                .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 50)
                .unwrap();
        });
        let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
    }

    let canvas = match canvas_opt {
        Some(c) => c,
        None => return Ok(()),
    };

    let mut w = canvas.client_width() as u32;
    let mut h = canvas.client_height() as u32;
    if w == 0 {
        w = window.inner_width().unwrap().as_f64().unwrap() as u32;
    }
    if h == 0 {
        h = window.inner_height().unwrap().as_f64().unwrap() as u32;
    }
    canvas.set_width(w);
    canvas.set_height(h);

    let ctx = canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()?;
    ctx.set_line_cap("round");
    ctx.set_line_join("round");

    let search = window.location().search().unwrap();
    let room_id = if let Some(pos) = search.find("room=") {
        search[pos + 5..]
            .split('&')
            .next()
            .unwrap_or("default")
            .to_string()
    } else {
        "default".to_string()
    };

    // Load from LocalStorage instantly!
    if let Ok(Some(ls)) = window.local_storage() {
        if let Ok(Some(cached)) = ls.get_item(&format!("board_{}", room_id)) {
            let img = web_sys::HtmlImageElement::new().unwrap();
            let ctx_clone = ctx.clone();
            let img_clone = img.clone();
            let onload = Closure::once(Box::new(move || {
                let _ = ctx_clone.draw_image_with_html_image_element(&img_clone, 0.0, 0.0);
            }));
            img.set_onload(Some(onload.as_ref().unchecked_ref()));
            onload.forget();
            img.set_src(&cached);
        }
    }

    let protocol = if window.location().protocol().unwrap() == "https:" {
        "wss:"
    } else {
        "ws:"
    };
    let host = window.location().host().unwrap();
    let ws = threadloom::ws::WsClient::new(&format!("{}//{}/api/ws?room={}", protocol, host, room_id)).unwrap();

    let is_drawing = Rc::new(RefCell::new(false));
    let start_x = Rc::new(RefCell::new(0.0));
    let start_y = Rc::new(RefCell::new(0.0));
    let last_x = Rc::new(RefCell::new(0.0));
    let last_y = Rc::new(RefCell::new(0.0));
    let cursor_states = Rc::new(RefCell::new(HashMap::<String, CursorAnimState>::new()));
    let doc_clone = document.clone();
    let update_cursor = Rc::new(move |uid: &str, x: f64, y: f64, is_local: bool| {
        let now = js_sys::Date::now();
        let mut states = cursor_states.borrow_mut();
        let state = states.entry(uid.to_string()).or_insert_with(|| {
            let _ = get_or_create_cursor(uid, is_local, &doc_clone);
            CursorAnimState {
                last_x: x,
                last_y: y,
                last_time: now,
                prev_angle: 0.0,
                acc_rotation: 0.0,
            }
        });

        let dt = now - state.last_time;
        let mut vx = 0.0;
        let mut vy = 0.0;
        if dt > 0.0 {
            vx = (x - state.last_x) / dt;
            vy = (y - state.last_y) / dt;
        }

        state.last_time = now;
        state.last_x = x;
        state.last_y = y;

        let speed = (vx * vx + vy * vy).sqrt();
        let mut scale_gsap = 1.0;

        if speed > 0.1 {
            let current_angle = vy.atan2(vx) * (180.0 / std::f64::consts::PI) + 90.0;
            let mut angle_diff = current_angle - state.prev_angle;
            if angle_diff > 180.0 { angle_diff -= 360.0; }
            if angle_diff < -180.0 { angle_diff += 360.0; }
            state.acc_rotation += angle_diff;
            state.prev_angle = current_angle;
            scale_gsap = 0.95;
        }

        let sel = format!("#cursor-{}", uid);
        let rot = state.acc_rotation;
        
        let script = format!(
            "if (window.gsap) {{ gsap.to('{}', {{ x: {}, y: {}, rotation: {}, scale: {}, opacity: 1, duration: 0.15, ease: 'power2.out', overwrite: 'auto' }}); gsap.to('{}', {{ scale: 1, duration: 0.15, delay: 0.15, overwrite: 'auto' }}); }}",
            sel, x, y, rot, scale_gsap, sel
        );
        let _ = js_sys::eval(&script);
    });

    let last_ws_send = Rc::new(RefCell::new(0.0));

    let ws_rc = Rc::new(ws);

    let send_snapshot = {
        let canvas = canvas.clone();
        let ws = ws_rc.clone();
        let room_id = room_id.clone();
        let window_ls = window.clone();
        Rc::new(move || {
            if let Ok(data_url) = canvas.to_data_url() {
                if let Ok(Some(ls)) = window_ls.local_storage() {
                    let _ = ls.set_item(&format!("board_{}", room_id), &data_url);
                }
                if ws.ws.ready_state() == WebSocket::OPEN {
                    let msg = serde_json::json!({
                        "type": "snapshot",
                        "data": data_url
                    })
                    .to_string();
                    let _ = ws.send(&msg);
                }
            }
        }) as Rc<dyn Fn()>
    };

    let cursor_canvas = document
        .create_element("canvas")?
        .dyn_into::<HtmlCanvasElement>()?;
    cursor_canvas.set_width(w);
    cursor_canvas.set_height(h);
    cursor_canvas.set_class_name("absolute inset-0 w-full h-full pointer-events-none z-10");
    canvas
        .parent_element()
        .unwrap()
        .append_child(&cursor_canvas)?;
    let cursor_ctx = cursor_canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()?;

    let ws_onmsg = {
        let ctx = ctx.clone();
        let canvas_ws = canvas.clone();
        let window_ws = window.clone();
        let room_id_ws = room_id.clone();
        let update_cursor_ws = update_cursor.clone();
        Closure::wrap(Box::new(move |e: MessageEvent| {
            if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                if let Ok(msg) = serde_json::from_str::<serde_json::Value>(&String::from(txt)) {
                    if msg["type"] == "snapshot" {
                        if let Some(base64) = msg["data"].as_str() {
                            let img = web_sys::HtmlImageElement::new().unwrap();
                            let ctx_clone = ctx.clone();
                            let img_clone = img.clone();
                            let onload = Closure::once(Box::new(move || {
                                let _ = ctx_clone
                                    .draw_image_with_html_image_element(&img_clone, 0.0, 0.0);
                            }));
                            img.set_onload(Some(onload.as_ref().unchecked_ref()));
                            onload.forget();
                            img.set_src(base64);
                        }
                    } else if msg["type"] == "room_deleted" {
                        // This room was deleted server-side. Wipe the canvas,
                        // drop the cached snapshot, and head back home.
                        ctx.clear_rect(
                            0.0,
                            0.0,
                            canvas_ws.width() as f64,
                            canvas_ws.height() as f64,
                        );
                        if let Ok(Some(ls)) = window_ws.local_storage() {
                            let _ = ls.remove_item(&format!("board_{}", room_id_ws));
                        }
                        web_sys::console::log_1(&"Room was deleted. Redirecting...".into());
                        let _ = window_ws.location().assign("/");
                    } else if msg["type"] == "stroke" {
                        if let Some(stroke) = msg.get("data") {
                            let tool = stroke["tool"].as_str().unwrap_or("brush");
                            let color = stroke["color"].as_str().unwrap_or("#000000");

                            if tool == "eraser" {
                                ctx.set_global_composite_operation("destination-out")
                                    .unwrap();
                                ctx.set_line_width(20.0);
                                ctx.set_stroke_style_str("rgba(0,0,0,1)");
                            } else {
                                ctx.set_global_composite_operation("source-over").unwrap();
                                ctx.set_stroke_style_str(color);
                            }

                            if tool == "shape" || tool == "rect" {
                                let x0 = stroke["x0"].as_f64().unwrap_or(0.0);
                                let y0 = stroke["y0"].as_f64().unwrap_or(0.0);
                                let x1 = stroke["x1"].as_f64().unwrap_or(0.0);
                                let y1 = stroke["y1"].as_f64().unwrap_or(0.0);
                                ctx.set_line_width(4.0);
                                ctx.stroke_rect(x0, y0, x1 - x0, y1 - y0);
                            } else if tool == "circle" {
                                let x0 = stroke["x0"].as_f64().unwrap_or(0.0);
                                let y0 = stroke["y0"].as_f64().unwrap_or(0.0);
                                let x1 = stroke["x1"].as_f64().unwrap_or(0.0);
                                let y1 = stroke["y1"].as_f64().unwrap_or(0.0);
                                let r = ((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt();
                                ctx.set_line_width(4.0);
                                ctx.begin_path();
                                let _ = ctx.arc(x0, y0, r, 0.0, std::f64::consts::PI * 2.0);
                                ctx.stroke();
                            } else if tool == "triangle" {
                                let x0 = stroke["x0"].as_f64().unwrap_or(0.0);
                                let y0 = stroke["y0"].as_f64().unwrap_or(0.0);
                                let x1 = stroke["x1"].as_f64().unwrap_or(0.0);
                                let y1 = stroke["y1"].as_f64().unwrap_or(0.0);
                                ctx.set_line_width(4.0);
                                ctx.begin_path();
                                ctx.move_to(x0 + (x1 - x0) / 2.0, y0);
                                ctx.line_to(x1, y1);
                                ctx.line_to(x0, y1);
                                ctx.close_path();
                                ctx.stroke();
                            } else if tool == "text" {
                                let x0 = stroke["x0"].as_f64().unwrap_or(0.0);
                                let y0 = stroke["y0"].as_f64().unwrap_or(0.0);
                                let text = stroke["text"].as_str().unwrap_or("");
                                ctx.set_fill_style_str(color);
                                ctx.set_font("20px sans-serif");
                                let _ = ctx.fill_text(text, x0, y0);
                            } else {
                                let x0 = stroke["x0"].as_f64().unwrap_or(0.0);
                                let y0 = stroke["y0"].as_f64().unwrap_or(0.0);
                                let x1 = stroke["x1"].as_f64().unwrap_or(0.0);
                                let y1 = stroke["y1"].as_f64().unwrap_or(0.0);
                                ctx.set_line_width(if tool == "eraser" { 20.0 } else { 4.0 });
                                ctx.begin_path();
                                ctx.move_to(x0, y0);
                                ctx.line_to(x1, y1);
                                ctx.stroke();
                            }
                        }
                    } else if msg["type"] == "cursor" {
                        if let Some(uid) = msg["user_id"].as_str() {
                            let x = msg["x"].as_f64().unwrap();
                            let y = msg["y"].as_f64().unwrap();
                            update_cursor_ws(uid, x, y, false);
                        }
                    } else if msg["type"] == "leave" {
                        if let Some(uid) = msg["user_id"].as_str() {
                            if let Some(el) = window_ws.document().unwrap().get_element_by_id(&format!("cursor-{}", uid)) {
                                el.remove();
                            }
                        }
                    }
                }
            }
        }) as Box<dyn FnMut(_)>)
    };
    ws_rc.ws.set_onmessage(Some(ws_onmsg.as_ref().unchecked_ref()));
    ws_onmsg.forget();

    let is_drawing_down = is_drawing.clone();
    let start_x_down = start_x.clone();
    let start_y_down = start_y.clone();
    let last_x_down = last_x.clone();
    let last_y_down = last_y.clone();
    let ctx_down = ctx.clone();
    let ws_down = ws_rc.clone();
    let tool_sig_down = tool_sig.clone();
    let color_sig_down = color_sig.clone();
    let window_down = window.clone();
    let send_snapshot_down = send_snapshot.clone();

    let on_down = Closure::wrap(Box::new(move |e: MouseEvent| {
        let x = e.offset_x() as f64;
        let y = e.offset_y() as f64;
        let tool = tool_sig_down.get();
        let color = color_sig_down.get();

        if tool == "text" {
            let input = window_down.document().unwrap().create_element("input").unwrap();
            let input_el = input.dyn_into::<web_sys::HtmlInputElement>().unwrap();
            input_el.set_type("text");
            input_el.set_class_name("absolute bg-transparent outline-none px-1 text-xl z-50");
            let _ = input_el.set_attribute("style", &format!("top: {}px; left: {}px; color: {}; font: 20px sans-serif; border: 1px dashed #ccc;", y - 10.0, x, color));
            
            let parent = window_down.document().unwrap().get_element_by_id("board-canvas").unwrap().parent_element().unwrap();
            let _ = parent.append_child(&input_el);
            
            let input_focus = input_el.clone();
            let focus_cb = Closure::once_into_js(move || {
                let _ = input_focus.focus();
            });
            window_down.set_timeout_with_callback_and_timeout_and_arguments_0(focus_cb.as_ref().unchecked_ref(), 10).unwrap();
            
            let color_clone = color.clone();
            let ctx_clone = ctx_down.clone();
            let ws_clone = ws_down.clone();
            let parent_clone = parent.clone();
            let input_clone = input_el.clone();
            let snapshot_clone = send_snapshot_down.clone();
            
            let on_blur = Closure::once(Box::new(move || {
                let text = input_clone.value();
                if !text.is_empty() {
                    ctx_clone.set_global_composite_operation("source-over").unwrap();
                    ctx_clone.set_fill_style_str(&color_clone);
                    ctx_clone.set_font("20px sans-serif");
                    let _ = ctx_clone.fill_text(&text, x, y + 10.0);
                    
                    if ws_clone.ws.ready_state() == WebSocket::OPEN {
                        let msg = serde_json::json!({
                            "type": "stroke",
                            "data": { "tool": "text", "color": color_clone, "x0": x, "y0": y + 10.0, "text": text }
                        }).to_string();
                        let _ = ws_clone.send(&msg);
                    }
                    snapshot_clone();
                }
                let _ = parent_clone.remove_child(&input_clone);
            }));
            
            input_el.set_onblur(Some(on_blur.as_ref().unchecked_ref()));
            
            let on_keydown = Closure::wrap(Box::new(move |e: web_sys::KeyboardEvent| {
                if e.key() == "Enter" {
                    let _ = e.target().unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap().blur();
                }
            }) as Box<dyn FnMut(_)>);
            input_el.set_onkeydown(Some(on_keydown.as_ref().unchecked_ref()));
            on_keydown.forget();
            
            on_blur.forget();
            return;
        }

        *is_drawing_down.borrow_mut() = true;
        *start_x_down.borrow_mut() = x;
        *start_y_down.borrow_mut() = y;
        *last_x_down.borrow_mut() = x;
        *last_y_down.borrow_mut() = y;
    }) as Box<dyn FnMut(_)>);
    canvas.add_event_listener_with_callback("mousedown", on_down.as_ref().unchecked_ref())?;
    on_down.forget();

    let is_drawing_up = is_drawing.clone();
    let start_x_up = start_x.clone();
    let start_y_up = start_y.clone();
    let ctx_up = ctx.clone();
    let ws_up = ws_rc.clone();
    let tool_sig_up = tool_sig.clone();
    let color_sig_up = color_sig.clone();
    let send_snapshot_up = send_snapshot.clone();

    let on_up = Closure::wrap(Box::new(move |e: MouseEvent| {
        if !*is_drawing_up.borrow() {
            return;
        }
        *is_drawing_up.borrow_mut() = false;

        let tool = tool_sig_up.get();
        let color = color_sig_up.get();
        if tool == "shape" || tool == "rect" || tool == "circle" || tool == "triangle" {
            let sx = *start_x_up.borrow();
            let sy = *start_y_up.borrow();
            let ex = e.offset_x() as f64;
            let ey = e.offset_y() as f64;

            ctx_up
                .set_global_composite_operation("source-over")
                .unwrap();
            ctx_up.set_stroke_style_str(&color);
            ctx_up.set_line_width(4.0);
            
            if tool == "shape" || tool == "rect" {
                ctx_up.stroke_rect(sx, sy, ex - sx, ey - sy);
            } else if tool == "circle" {
                let r = ((ex - sx).powi(2) + (ey - sy).powi(2)).sqrt();
                ctx_up.begin_path();
                let _ = ctx_up.arc(sx, sy, r, 0.0, std::f64::consts::PI * 2.0);
                ctx_up.stroke();
            } else if tool == "triangle" {
                ctx_up.begin_path();
                ctx_up.move_to(sx + (ex - sx) / 2.0, sy);
                ctx_up.line_to(ex, ey);
                ctx_up.line_to(sx, ey);
                ctx_up.close_path();
                ctx_up.stroke();
            }

            if ws_up.ws.ready_state() == WebSocket::OPEN {
                let msg = serde_json::json!({
                    "type": "stroke",
                    "data": { "tool": tool, "color": color, "x0": sx, "y0": sy, "x1": ex, "y1": ey }
                }).to_string();
                let _ = ws_up.send(&msg);
            }
        }

        // Take snapshot after drawing stroke/shape/eraser is complete
        send_snapshot_up();
    }) as Box<dyn FnMut(_)>);
    canvas.add_event_listener_with_callback("mouseup", on_up.as_ref().unchecked_ref())?;
    canvas.add_event_listener_with_callback("mouseleave", on_up.as_ref().unchecked_ref())?;
    on_up.forget();

    let is_drawing_move = is_drawing.clone();
    let last_x_move = last_x.clone();
    let last_y_move = last_y.clone();
    let ctx_move = ctx.clone();
    let ws_move = ws_rc.clone();
    let last_ws_send_move = last_ws_send.clone();
    let tool_sig_move = tool_sig.clone();
    let color_sig_move = color_sig.clone();

    let on_move = Closure::wrap(Box::new(move |e: MouseEvent| {
        let x = e.offset_x() as f64;
        let y = e.offset_y() as f64;

        let now = js_sys::Date::now();
        if now - *last_ws_send_move.borrow() > 30.0 {
            if ws_move.ws.ready_state() == WebSocket::OPEN {
                let msg = format!(r#"{{"type":"cursor","x":{},"y":{}}}"#, x, y);
                let _ = ws_move.send(&msg);
            }
            *last_ws_send_move.borrow_mut() = now;
        }

        if !*is_drawing_move.borrow() {
            *last_x_move.borrow_mut() = x;
            *last_y_move.borrow_mut() = y;
            return;
        }

        let tool = tool_sig_move.get();
        let color = color_sig_move.get();

        if tool == "brush" || tool == "eraser" {
            let lx = *last_x_move.borrow();
            let ly = *last_y_move.borrow();

            if tool == "eraser" {
                ctx_move
                    .set_global_composite_operation("destination-out")
                    .unwrap();
                ctx_move.set_line_width(20.0);
                ctx_move.set_stroke_style_str("rgba(0,0,0,1)");
            } else {
                ctx_move
                    .set_global_composite_operation("source-over")
                    .unwrap();
                ctx_move.set_line_width(4.0);
                ctx_move.set_stroke_style_str(&color);
            }

            ctx_move.begin_path();
            ctx_move.move_to(lx, ly);
            ctx_move.line_to(x, y);
            ctx_move.stroke();

            if ws_move.ws.ready_state() == WebSocket::OPEN {
                let msg = serde_json::json!({
                    "type": "stroke",
                    "data": { "tool": tool, "color": color, "x0": lx, "y0": ly, "x1": x, "y1": y }
                })
                .to_string();
                let _ = ws_move.send(&msg);
            }
        }

        *last_x_move.borrow_mut() = x;
        *last_y_move.borrow_mut() = y;
    }) as Box<dyn FnMut(_)>);
    canvas.add_event_listener_with_callback("mousemove", on_move.as_ref().unchecked_ref())?;
    on_move.forget();

    let f = Rc::new(RefCell::new(None as Option<Closure<dyn FnMut()>>));
    let g = f.clone();

    let w_raf = window.clone();
    let cursor_ctx_raf = cursor_ctx.clone();
    let cursor_canvas_raf = cursor_canvas.clone();
    let is_drawing_raf = is_drawing.clone();
    let tool_sig_raf = tool_sig.clone();
    let color_sig_raf = color_sig.clone();
    let start_x_raf = start_x.clone();
    let start_y_raf = start_y.clone();
    let last_x_raf = last_x.clone();
    let last_y_raf = last_y.clone();
    
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        cursor_ctx_raf.clear_rect(
            0.0,
            0.0,
            cursor_canvas_raf.width() as f64,
            cursor_canvas_raf.height() as f64,
        );

        let tool = tool_sig_raf.get();
        if *is_drawing_raf.borrow() && (tool == "shape" || tool == "rect" || tool == "circle" || tool == "triangle") {
            cursor_ctx_raf.set_stroke_style_str(&color_sig_raf.get());
            cursor_ctx_raf.set_line_width(4.0);
            let sx = *start_x_raf.borrow();
            let sy = *start_y_raf.borrow();
            let lx = *last_x_raf.borrow();
            let ly = *last_y_raf.borrow();
            
            if tool == "shape" || tool == "rect" {
                cursor_ctx_raf.stroke_rect(sx, sy, lx - sx, ly - sy);
            } else if tool == "circle" {
                let r = ((lx - sx).powi(2) + (ly - sy).powi(2)).sqrt();
                cursor_ctx_raf.begin_path();
                let _ = cursor_ctx_raf.arc(sx, sy, r, 0.0, std::f64::consts::PI * 2.0);
                cursor_ctx_raf.stroke();
            } else if tool == "triangle" {
                cursor_ctx_raf.begin_path();
                cursor_ctx_raf.move_to(sx + (lx - sx) / 2.0, sy);
                cursor_ctx_raf.line_to(lx, ly);
                cursor_ctx_raf.line_to(sx, ly);
                cursor_ctx_raf.close_path();
                cursor_ctx_raf.stroke();
            }
        }

        w_raf
            .request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .unwrap();
    }) as Box<dyn FnMut()>));

    window.request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())?;

    Ok(())
}
