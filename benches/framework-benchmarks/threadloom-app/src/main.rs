use threadloom_core::{create_signal, element, IntoView, View};
use threadloom_macro::threadloom;

#[derive(Clone, PartialEq)]
struct RowData {
    id: usize,
    id_str: std::rc::Rc<String>,
    label: std::rc::Rc<String>,
}

pub fn app() -> threadloom_core::View {
    let (rows, set_rows) = create_signal(Vec::<RowData>::new());
    
    let remove_cb = std::rc::Rc::new({
        let rows = rows.clone();
        let set_rows = set_rows.clone();
        move |e: threadloom_dom::web_sys::Event| {
            use wasm_bindgen::JsCast;
            let target = e.target().unwrap().unchecked_into::<threadloom_dom::web_sys::Element>();
            if let Some(tr) = target.closest("tr").unwrap() {
                if let Some(row_id) = tr.get_attribute("data-row-id") {
                    if let Ok(id) = row_id.parse::<usize>() {
                        let mut new_rows = rows.get();
                        new_rows.retain(|r| r.id != id);
                        set_rows.set(new_rows);
                    }
                }
            }
        }
    }) as std::rc::Rc<dyn Fn(threadloom_dom::web_sys::Event)>;

    let (next_id, set_next_id) = create_signal(1);

    let create_rows = move |count: usize| {
        let start = next_id.get();
        let mut new_rows = Vec::with_capacity(count);
        for i in 0..count {
            new_rows.push(RowData {
                id: start + i,
                id_str: std::rc::Rc::new((start + i).to_string()),
                label: std::rc::Rc::new(format!("Row {}", start + i)),
            });
        }
        set_next_id.set(start + count);
        set_rows.set(new_rows);
    };

    let append_rows = move |count: usize| {
        let start = next_id.get();
        let mut new_rows = rows.get();
        for i in 0..count {
            new_rows.push(RowData {
                id: start + i,
                id_str: std::rc::Rc::new((start + i).to_string()),
                label: std::rc::Rc::new(format!("Row {}", start + i)),
            });
        }
        set_next_id.set(start + count);
        set_rows.set(new_rows);
    };

    let update_rows = move || {
        let mut new_rows = rows.get();
        for (i, row) in new_rows.iter_mut().enumerate() {
            if i % 10 == 0 {
                row.label = std::rc::Rc::new(format!("{} !!!", row.label));
            }
        }
        set_rows.set(new_rows);
    };

    let clear_rows = move || {
        set_rows.set(vec![]);
    };

    let swap_rows = move || {
        let mut new_rows = rows.get();
        if new_rows.len() > 998 {
            new_rows.swap(1, 998);
        }
        set_rows.set(new_rows);
    };

    threadloom! {
        div(class="container") {
            div(class="jumbotron") {
                div(class="row") {
                    div(class="col-md-6") {
                        h1() { "Threadloom Benchmark" }
                    }
                    div(class="col-md-6") {
                        div(class="row") {
                            div(class="col-sm-6 smallpad") {
                                button(type="button", class="btn btn-primary btn-block", id="run", on_click=move || create_rows(1000)) { "Create 1,000 rows" }
                            }
                            div(class="col-sm-6 smallpad") {
                                button(type="button", class="btn btn-primary btn-block", id="runlots", on_click=move || create_rows(10000)) { "Create 10,000 rows" }
                            }
                            div(class="col-sm-6 smallpad") {
                                button(type="button", class="btn btn-primary btn-block", id="add", on_click=move || append_rows(1000)) { "Append 1,000 rows" }
                            }
                            div(class="col-sm-6 smallpad") {
                                button(type="button", class="btn btn-primary btn-block", id="update", on_click=move || update_rows()) { "Update every 10th row" }
                            }
                            div(class="col-sm-6 smallpad") {
                                button(type="button", class="btn btn-primary btn-block", id="clear", on_click=move || clear_rows()) { "Clear" }
                            }
                            div(class="col-sm-6 smallpad") {
                                button(type="button", class="btn btn-primary btn-block", id="swaprows", on_click=move || swap_rows()) { "Swap Rows" }
                            }
                        }
                    }
                }
            }
            table(class="table table-hover table-striped test-data") {
                tbody() {
                    {
                        threadloom_core::map_keyed(
                            rows,
                            |row| row.id,
                            {
                                let remove_cb = remove_cb.clone();
                                move |row_sig| {
                                    let id_str = row_sig.get().id_str;
                                    element("tr").attr("key", id_str.clone()).attr("data-row-id", id_str.clone())
                                        .child(element("td").attr("class", "col-md-1").child(id_str.clone()))
                                        .child(element("td").attr("class", "col-md-4").child(element("a").attr("class", "lbl").child(
                                            threadloom_core::View::DynamicRcText(std::rc::Rc::new(move || row_sig.get().label.clone()))
                                        )))
                                        .child(element("td").attr("class", "col-md-1").child(
                                            element("a").attr("class", "remove").on_obj_rc("click", remove_cb.clone()).child(element("span").attr("class", "glyphicon glyphicon-remove").attr("aria-hidden", "true"))
                                        ))
                                        .child(element("td").attr("class", "col-md-6"))
                                }
                            }
                        )
                    }
                }
            }
        }
    }
}

pub fn main() {
    let window = threadloom_dom::web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();
    let _ = threadloom_dom::mount(app(), &body.into());
}
