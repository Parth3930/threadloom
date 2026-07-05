use yew::prelude::*;

#[derive(Clone, PartialEq)]
struct RowData {
    id: usize,
    label: String,
}

#[function_component(App)]
fn app() -> Html {
    let rows = use_state(Vec::<RowData>::new);
    let next_id = use_state(|| 1);

    let create_rows = {
        let rows = rows.clone();
        let next_id = next_id.clone();
        move |count: usize| {
            let start = *next_id;
            let mut new_rows = vec![];
            for i in 0..count {
                new_rows.push(RowData {
                    id: start + i,
                    label: format!("Row {}", start + i),
                });
            }
            next_id.set(start + count);
            rows.set(new_rows);
        }
    };

    let append_rows = {
        let rows = rows.clone();
        let next_id = next_id.clone();
        move |count: usize| {
            let start = *next_id;
            let mut new_rows = (*rows).clone();
            for i in 0..count {
                new_rows.push(RowData {
                    id: start + i,
                    label: format!("Row {}", start + i),
                });
            }
            next_id.set(start + count);
            rows.set(new_rows);
        }
    };

    let update_rows = {
        let rows = rows.clone();
        move || {
            let mut new_rows = (*rows).clone();
            for (i, row) in new_rows.iter_mut().enumerate() {
                if i % 10 == 0 {
                    row.label.push_str(" !!!");
                }
            }
            rows.set(new_rows);
        }
    };

    let clear_rows = {
        let rows = rows.clone();
        move || {
            rows.set(vec![]);
        }
    };

    let swap_rows = {
        let rows = rows.clone();
        move || {
            let mut new_rows = (*rows).clone();
            if new_rows.len() > 998 {
                new_rows.swap(1, 998);
            }
            rows.set(new_rows);
        }
    };

    let remove_row = {
        let rows = rows.clone();
        move |id: usize| {
            let mut new_rows = (*rows).clone();
            new_rows.retain(|r| r.id != id);
            rows.set(new_rows);
        }
    };

    html! {
        <div class="container">
            <div class="jumbotron">
                <div class="row">
                    <div class="col-md-6">
                        <h1>{"Yew Benchmark"}</h1>
                    </div>
                    <div class="col-md-6">
                        <div class="row">
                            <div class="col-sm-6 smallpad">
                                <button type="button" class="btn btn-primary btn-block" id="run" onclick={
                                    let create_rows = create_rows.clone();
                                    Callback::from(move |_| create_rows(1000))
                                }>{"Create 1,000 rows"}</button>
                            </div>
                            <div class="col-sm-6 smallpad">
                                <button type="button" class="btn btn-primary btn-block" id="runlots" onclick={
                                    let create_rows = create_rows.clone();
                                    Callback::from(move |_| create_rows(10000))
                                }>{"Create 10,000 rows"}</button>
                            </div>
                            <div class="col-sm-6 smallpad">
                                <button type="button" class="btn btn-primary btn-block" id="add" onclick={
                                    let append_rows = append_rows.clone();
                                    Callback::from(move |_| append_rows(1000))
                                }>{"Append 1,000 rows"}</button>
                            </div>
                            <div class="col-sm-6 smallpad">
                                <button type="button" class="btn btn-primary btn-block" id="update" onclick={
                                    let update_rows = update_rows.clone();
                                    Callback::from(move |_| update_rows())
                                }>{"Update every 10th row"}</button>
                            </div>
                            <div class="col-sm-6 smallpad">
                                <button type="button" class="btn btn-primary btn-block" id="clear" onclick={
                                    let clear_rows = clear_rows.clone();
                                    Callback::from(move |_| clear_rows())
                                }>{"Clear"}</button>
                            </div>
                            <div class="col-sm-6 smallpad">
                                <button type="button" class="btn btn-primary btn-block" id="swaprows" onclick={
                                    let swap_rows = swap_rows.clone();
                                    Callback::from(move |_| swap_rows())
                                }>{"Swap Rows"}</button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
            <table class="table table-hover table-striped test-data">
                <tbody>
                    {
                        for rows.iter().map(|row| {
                            let id = row.id;
                            let remove = {
                                let remove_row = remove_row.clone();
                                Callback::from(move |_| remove_row(id))
                            };
                            html! {
                                <tr key={row.id}>
                                    <td class="col-md-1">{row.id}</td>
                                    <td class="col-md-4">
                                        <a class="lbl">{&row.label}</a>
                                    </td>
                                    <td class="col-md-1">
                                        <a class="remove" onclick={remove}>
                                            <span class="glyphicon glyphicon-remove" aria-hidden="true"></span>
                                        </a>
                                    </td>
                                    <td class="col-md-6"></td>
                                </tr>
                            }
                        })
                    }
                </tbody>
            </table>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
