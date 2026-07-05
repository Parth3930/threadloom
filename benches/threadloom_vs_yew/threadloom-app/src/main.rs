use std::rc::Rc;
use threadloom_core::{create_signal, create_memo, View, IntoView};
use threadloom_macro::threadloom;
use threadloom_dom::mount;
use web_sys::window;
use wasm_bindgen::prelude::*;

#[derive(Clone, PartialEq)]
struct RowData {
    id: usize,
    label: String,
}

fn app() -> View {
    let (rows_read, rows_write) = create_signal(Vec::<RowData>::new());
    let (next_id_read, next_id_write) = create_signal(1);

    let create_rows = {
        let rows_write = rows_write.clone();
        let next_id_read = next_id_read.clone();
        let next_id_write = next_id_write.clone();
        Rc::new(move |count: usize| {
            let start = next_id_read.get();
            let mut new_rows = Vec::with_capacity(count);
            for i in 0..count {
                new_rows.push(RowData {
                    id: start + i,
                    label: format!("Row {}", start + i),
                });
            }
            next_id_write.set(start + count);
            rows_write.set(new_rows);
        })
    };

    let append_rows = {
        let rows_read = rows_read.clone();
        let rows_write = rows_write.clone();
        let next_id_read = next_id_read.clone();
        let next_id_write = next_id_write.clone();
        Rc::new(move |count: usize| {
            let start = next_id_read.get();
            let mut new_rows = rows_read.get();
            new_rows.reserve(count);
            for i in 0..count {
                new_rows.push(RowData {
                    id: start + i,
                    label: format!("Row {}", start + i),
                });
            }
            next_id_write.set(start + count);
            rows_write.set(new_rows);
        })
    };

    let update_rows = {
        let rows_read = rows_read.clone();
        let rows_write = rows_write.clone();
        Rc::new(move || {
            let mut new_rows = rows_read.get();
            for (i, row) in new_rows.iter_mut().enumerate() {
                if i % 10 == 0 {
                    row.label.push_str(" !!!");
                }
            }
            rows_write.set(new_rows);
        })
    };

    let clear_rows = {
        let rows_write = rows_write.clone();
        Rc::new(move || {
            rows_write.set(Vec::new());
        })
    };

    let swap_rows = {
        let rows_read = rows_read.clone();
        let rows_write = rows_write.clone();
        Rc::new(move || {
            let mut new_rows = rows_read.get();
            if new_rows.len() > 998 {
                new_rows.swap(1, 998);
            }
            rows_write.set(new_rows);
        })
    };

    let remove_row = {
        let rows_read = rows_read.clone();
        let rows_write = rows_write.clone();
        Rc::new(move |id: usize| {
            let mut new_rows = rows_read.get();
            new_rows.retain(|r| r.id != id);
            rows_write.set(new_rows);
        })
    };

    let rows_read_render = rows_read.clone();
    
    threadloom! {
        div(class="container") {
            div(class="jumbotron") {
                div(class="row") {
                    div(class="col-md-6") {
                        h1 { "Threadloom Benchmark" }
                    }
                    div(class="col-md-6") {
                        div(class="row") {
                            div(class="col-sm-6 smallpad") {
                                button(type="button", class="btn btn-primary btn-block", id="run", on_click={
                                    let cb = create_rows.clone();
                                    move || cb(1000)
                                }) { "Create 1,000 rows" }
                            }
                            div(class="col-sm-6 smallpad") {
                                button(type="button", class="btn btn-primary btn-block", id="runlots", on_click={
                                    let cb = create_rows.clone();
                                    move || cb(10000)
                                }) { "Create 10,000 rows" }
                            }
                            div(class="col-sm-6 smallpad") {
                                button(type="button", class="btn btn-primary btn-block", id="add", on_click={
                                    let cb = append_rows.clone();
                                    move || cb(1000)
                                }) { "Append 1,000 rows" }
                            }
                            div(class="col-sm-6 smallpad") {
                                button(type="button", class="btn btn-primary btn-block", id="update", on_click={
                                    let cb = update_rows.clone();
                                    move || cb()
                                }) { "Update every 10th row" }
                            }
                            div(class="col-sm-6 smallpad") {
                                button(type="button", class="btn btn-primary btn-block", id="clear", on_click={
                                    let cb = clear_rows.clone();
                                    move || cb()
                                }) { "Clear" }
                            }
                            div(class="col-sm-6 smallpad") {
                                button(type="button", class="btn btn-primary btn-block", id="swaprows", on_click={
                                    let cb = swap_rows.clone();
                                    move || cb()
                                }) { "Swap Rows" }
                            }
                        }
                    }
                }
            }
            table(class="table table-hover table-striped test-data") {
                tbody {
                    { move || {
                        let current_rows = rows_read_render.get();
                        let children: Vec<View> = current_rows.into_iter().map(|row| {
                            let id = row.id;
                            let remove_cb = {
                                let r = remove_row.clone();
                                move || r(id)
                            };
                            let lbl = row.label.clone();
                            let id_str = id.to_string();
                            threadloom! {
                                tr(key=id_str.clone()) {
                                    td(class="col-md-1") { { id_str } }
                                    td(class="col-md-4") {
                                        a(class="lbl") { { lbl } }
                                    }
                                    td(class="col-md-1") {
                                        a(class="remove", on_click=remove_cb) {
                                            span(class="glyphicon glyphicon-remove", aria_hidden="true") {}
                                        }
                                    }
                                    td(class="col-md-6") {}
                                }
                            }
                        }).collect();
                        threadloom_core::fragment(children)
                    } }
                }
            }
        }
    }
}

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    let window = window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");
    
    // Clear the body
    body.set_inner_html("");
    
    mount(app(), &body)?;
    Ok(())
}

fn main() {
    // entry point handled by wasm_bindgen(start)
}
