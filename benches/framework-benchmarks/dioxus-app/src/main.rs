#![allow(non_snake_case)]
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
struct RowData {
    id: usize,
    label: String,
}

fn main() {
    launch(App);
}

fn App() -> Element {
    let mut rows = use_signal(|| Vec::<RowData>::new());
    let mut next_id = use_signal(|| 1);

    let mut create_rows = move |count: usize| {
        let start = next_id();
        let mut new_rows = Vec::with_capacity(count);
        for i in 0..count {
            new_rows.push(RowData {
                id: start + i,
                label: format!("Row {}", start + i),
            });
        }
        next_id.set(start + count);
        rows.set(new_rows);
    };

    let mut append_rows = move |count: usize| {
        let start = next_id();
        let mut new_rows = rows().clone();
        for i in 0..count {
            new_rows.push(RowData {
                id: start + i,
                label: format!("Row {}", start + i),
            });
        }
        next_id.set(start + count);
        rows.set(new_rows);
    };

    let mut update_rows = move || {
        let mut new_rows = rows().clone();
        for (i, row) in new_rows.iter_mut().enumerate() {
            if i % 10 == 0 {
                row.label.push_str(" !!!");
            }
        }
        rows.set(new_rows);
    };

    let mut clear_rows = move || {
        rows.set(vec![]);
    };

    let mut swap_rows = move || {
        let mut new_rows = rows().clone();
        if new_rows.len() > 998 {
            new_rows.swap(1, 998);
        }
        rows.set(new_rows);
    };

    let mut remove_row = move |id: usize| {
        let mut new_rows = rows().clone();
        new_rows.retain(|r| r.id != id);
        rows.set(new_rows);
    };

    rsx! {
        div { class: "container",
            div { class: "jumbotron",
                div { class: "row",
                    div { class: "col-md-6",
                        h1 { "Dioxus Benchmark" }
                    }
                    div { class: "col-md-6",
                        div { class: "row",
                            div { class: "col-sm-6 smallpad",
                                button { r#type: "button", class: "btn btn-primary btn-block", id: "run", onclick: move |_| create_rows(1000), "Create 1,000 rows" }
                            }
                            div { class: "col-sm-6 smallpad",
                                button { r#type: "button", class: "btn btn-primary btn-block", id: "runlots", onclick: move |_| create_rows(10000), "Create 10,000 rows" }
                            }
                            div { class: "col-sm-6 smallpad",
                                button { r#type: "button", class: "btn btn-primary btn-block", id: "add", onclick: move |_| append_rows(1000), "Append 1,000 rows" }
                            }
                            div { class: "col-sm-6 smallpad",
                                button { r#type: "button", class: "btn btn-primary btn-block", id: "update", onclick: move |_| update_rows(), "Update every 10th row" }
                            }
                            div { class: "col-sm-6 smallpad",
                                button { r#type: "button", class: "btn btn-primary btn-block", id: "clear", onclick: move |_| clear_rows(), "Clear" }
                            }
                            div { class: "col-sm-6 smallpad",
                                button { r#type: "button", class: "btn btn-primary btn-block", id: "swaprows", onclick: move |_| swap_rows(), "Swap Rows" }
                            }
                        }
                    }
                }
            }
            table { class: "table table-hover table-striped test-data",
                tbody {
                    for row in rows().into_iter() {
                        tr { key: "{row.id}",
                            td { class: "col-md-1", "{row.id}" }
                            td { class: "col-md-4",
                                a { class: "lbl", "{row.label}" }
                            }
                            td { class: "col-md-1",
                                a { class: "remove", onclick: move |_| remove_row(row.id),
                                    span { class: "glyphicon glyphicon-remove", "aria-hidden": "true" }
                                }
                            }
                            td { class: "col-md-6" }
                        }
                    }
                }
            }
        }
    }
}
