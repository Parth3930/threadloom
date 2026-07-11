use leptos::*;

#[derive(Clone, PartialEq)]
struct RowData {
    id: usize,
    label: RwSignal<String>,
}

#[component]
fn App() -> impl IntoView {
    let (rows, set_rows) = create_signal(Vec::<RowData>::new());
    let next_id = create_rw_signal(1);

    let create_rows = move |count: usize| {
        let start = next_id.get();
        let mut new_rows = vec![];
        for i in 0..count {
            new_rows.push(RowData {
                id: start + i,
                label: create_rw_signal(format!("Row {}", start + i)),
            });
        }
        next_id.set(start + count);
        set_rows.set(new_rows);
    };

    let append_rows = move |count: usize| {
        let start = next_id.get();
        set_rows.update(|rows| {
            for i in 0..count {
                rows.push(RowData {
                    id: start + i,
                    label: create_rw_signal(format!("Row {}", start + i)),
                });
            }
        });
        next_id.set(start + count);
    };

    let update_rows = move || {
        set_rows.update(|rows| {
            for i in (0..rows.len()).step_by(10) {
                rows[i].label.update(|l| l.push_str(" !!!"));
            }
        });
    };

    let clear_rows = move || {
        set_rows.set(vec![]);
    };

    let swap_rows = move || {
        set_rows.update(|rows| {
            if rows.len() > 998 {
                rows.swap(1, 998);
            }
        });
    };

    let remove_row = move |id: usize| {
        set_rows.update(|rows| {
            if let Some(pos) = rows.iter().position(|r| r.id == id) {
                rows.remove(pos);
            }
        });
    };

    view! {
        <div class="container">
            <div class="jumbotron">
                <div class="row">
                    <div class="col-md-6">
                        <h1>"Leptos benchmark"</h1>
                    </div>
                    <div class="col-md-6">
                        <div class="row">
                            <div class="col-sm-6 smallpad">
                                <button type="button" class="btn btn-primary btn-block" id="run" on:click=move |_| create_rows(1000)>
                                    "Create 1,000 rows"
                                </button>
                            </div>
                            <div class="col-sm-6 smallpad">
                                <button type="button" class="btn btn-primary btn-block" id="runlots" on:click=move |_| create_rows(10000)>
                                    "Create 10,000 rows"
                                </button>
                            </div>
                            <div class="col-sm-6 smallpad">
                                <button type="button" class="btn btn-primary btn-block" id="add" on:click=move |_| append_rows(1000)>
                                    "Append 1,000 rows"
                                </button>
                            </div>
                            <div class="col-sm-6 smallpad">
                                <button type="button" class="btn btn-primary btn-block" id="update" on:click=move |_| update_rows()>
                                    "Update every 10th row"
                                </button>
                            </div>
                            <div class="col-sm-6 smallpad">
                                <button type="button" class="btn btn-primary btn-block" id="clear" on:click=move |_| clear_rows()>
                                    "Clear"
                                </button>
                            </div>
                            <div class="col-sm-6 smallpad">
                                <button type="button" class="btn btn-primary btn-block" id="swaprows" on:click=move |_| swap_rows()>
                                    "Swap Rows"
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
            <table class="table table-hover table-striped test-data">
                <tbody>
                    <For
                        each=move || rows.get()
                        key=|row| row.id
                        children=move |row| {
                            let id = row.id;
                            view! {
                                <tr>
                                    <td class="col-md-1">{id}</td>
                                    <td class="col-md-4">
                                        <a>{row.label}</a>
                                    </td>
                                    <td class="col-md-1">
                                        <a on:click=move |_| remove_row(id)>
                                            <span class="glyphicon glyphicon-remove" aria-hidden="true"></span>
                                        </a>
                                    </td>
                                    <td class="col-md-6"></td>
                                </tr>
                            }
                        }
                    />
                </tbody>
            </table>
            <span class="preloadicon glyphicon glyphicon-remove" aria-hidden="true"></span>
        </div>
    }
}

pub fn main() {
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();
    mount_to_body(App);
}
