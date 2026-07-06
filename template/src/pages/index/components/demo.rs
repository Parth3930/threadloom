use crate::api::hello::route::{hello, HelloArgs};
use threadloom_core::{create_signal, Action, GlobalSignal, Signal, View};
use threadloom_macro::threadloom;
use threadloom_ui::*;

static USER_SCORE: GlobalSignal<i32> = GlobalSignal::new(|| 0);
use threadloom_ui::*;

pub fn demo_component() -> View {
    let (terms_agreed, set_terms_agreed) = create_signal(false);
    let (selected_option, set_selected_option) = create_signal(1);
    let (accordion_open, set_accordion_open) = create_signal(false);
    let (dropdown_open, set_dropdown_open) = create_signal(false);
    let (dialog_open, set_dialog_open) = create_signal(false);
    let (active_tab, set_active_tab) = create_signal(0);
    let (hamburger_open, set_hamburger_open) = create_signal(false);
    let (select_val, set_select_val) = create_signal("1".to_string());

    let (api_response, set_api_response) = create_signal("Click to fetch...".to_string());
    let (counter, set_counter) = create_signal(0);
    let (cookie_val, set_cookie_val) =
        create_signal(threadloom_dom::get_cookie!("demo_cookie").unwrap_or_default());

    let (count, set_count) = create_signal(1);
    let doubled = Signal::computed(move || count.get() * 2);

    let submit_action = Action::new(|name: String| async move {
        let _ = reqwasm::http::Request::get("https://httpbin.org/delay/1")
            .send()
            .await;
        format!("Action response: {}", name)
    });
    let action_clone = submit_action.clone();
    let (action_result, set_action_result) = create_signal(String::new());

    threadloom! {
        Column(gap=8) {
            Grid(cols=1, md_cols=2, gap=8) {

                // Forms & Inputs
                Card(title="Forms & Inputs") {
                    Heading(level=4, mt=2, mb=1, weight="light") { "Buttons" }
                    Row(gap=4, items="center") {
                        Button(label="Primary Button", primary=true, on_click={|| {
                            threadloom_dom::alert!("Primary Button Clicked!");
                        }})
                        Button(label="Secondary Button", primary=false, on_click={|| {
                            threadloom_dom::alert!("Secondary Button Clicked!");
                        }})
                    }
                }

                // Text Inputs
                Card(title="Text Inputs") {
                    Column(gap=2) {
                        Column(gap=1) {
                            Label(text="Username", for="user")
                            Input(value="", placeholder="Enter your username...", on_input={|| {}})
                        }
                        Textarea(value="", placeholder="Write a message...", on_input={|| {}})
                    }
                }

                // Selections
                Card(title="Selections") {
                    Row(items="center", gap=2) {
                        { move || threadloom! { Checkbox(checked={terms_agreed.get()}, id="terms", on_change={move || { set_terms_agreed.set(!terms_agreed.get()); }}) } }
                        Label(text="Accept Terms & Conditions", for="terms")
                    }
                    { move || threadloom! { RadioGroup(
                        options=vec![
                            ("1".to_string(), "Option 1".to_string()),
                            ("2".to_string(), "Option 2".to_string()),
                        ],
                        selected_value={selected_option.get().to_string()},
                        name="options",
                        on_change={move |val: String| {
                            if let Ok(num) = val.parse::<i32>() {
                                set_selected_option.set(num);
                            }
                        }}
                    ) } }
                }

                // Interactive
                Card(title="Interactive") {
                    { move || threadloom! {
                        Accordion(
                            title="Click to Expand",
                            open={accordion_open.get()},
                            on_toggle={move || set_accordion_open.set(!accordion_open.get())}
                        ) {
                            Text(variant="p", class="text-gray-600 dark:text-gray-400 p-4") {
                                "This content is revealed with a smooth height transition. It's built into the Accordion component."
                            }
                        }
                    } }
                    Column(mt=4, gap=4) {
                        { move || threadloom! {
                            Dropdown(
                                label="Actions",
                                open={dropdown_open.get()},
                                on_toggle={move || set_dropdown_open.set(!dropdown_open.get())},
                                items=vec![
                                    threadloom! { Button(label="Edit", primary=false, class="tl-dropdown-item", on_click={move || set_dropdown_open.set(false)}) },
                                    threadloom! { Button(label="Duplicate", primary=false, class="tl-dropdown-item", on_click={move || set_dropdown_open.set(false)}) },
                                    threadloom! { Divider(class="tl-dropdown-divider", my=1) },
                                    threadloom! { Button(label="Delete", primary=false, class="tl-dropdown-item text-red-600 dark:text-red-400 border-none shadow-none", on_click={move || set_dropdown_open.set(false)}) },
                                ]
                            )
                        } }

                        Row(mt=2) {
                            Button(label="Open Modal", primary=false, on_click={move || set_dialog_open.set(true)}) {}
                            { move || threadloom! { Tooltip(tooltip_text="This explains the icon") {
                                Button(label="ℹ️", primary=false, class="p-2 rounded-full hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors text-gray-600 dark:text-gray-300 border-none shadow-none focus-visible:outline-none focus-visible:ring-0", on_click={|| {}})
                            } } }
                        }
                        { move || threadloom! {
                            Dialog(
                                open={dialog_open.get()},
                                title="Example Modal",
                                on_close={move || set_dialog_open.set(false)},
                                footer={
                                    threadloom! {
                                        Row(gap=2, mt=4) {
                                            Button(label="Cancel", primary=false, on_click={move || set_dialog_open.set(false)})
                                            Button(label="Confirm Action", primary=true, on_click={move || set_dialog_open.set(false)})
                                        }
                                    }
                                }
                            ) {
                                Text(variant="p", class="text-gray-600 dark:text-gray-400 py-4") {
                                    "This is a premium modal built with Threadloom UI. You can fully customize its footer actions!"
                                }
                            }
                        } }
                    }
                }

                // More Interactive
                Card(title="More Interactive") {

                    Heading(level=4, mt=2, mb=1, class="text-sm font-medium text-gray-700 dark:text-gray-300") { "Select & Tabs" }
                    { move || threadloom! { Select(
                        options=vec![
                            ("1".to_string(), "Option 1".to_string()),
                            ("2".to_string(), "Option 2".to_string()),
                        ],
                        selected_value={select_val.get()},
                        on_change={move || {
                            let next = if select_val.get() == "1" { "2" } else { "1" };
                            set_select_val.set(next.to_string());
                        }}
                    ) } }
                    Row(mt=6) {
                        { move || threadloom! {
                            Tabs(
                                tab_labels=vec!["Profile".to_string(), "Settings".to_string(), "Notifications".to_string()],
                                active_index={active_tab.get()},
                                on_tab_click={move |i: usize| set_active_tab.set(i)},
                                panels=vec![
                                    threadloom! { p(class="p-4 text-gray-700 dark:text-gray-300") { "Your profile information." } },
                                    threadloom! { p(class="p-4 text-gray-700 dark:text-gray-300") { "Update your settings here." } },
                                    threadloom! { p(class="p-4 text-gray-700 dark:text-gray-300") { "You have 3 unread messages." } },
                                ]
                            )
                        } }
                    }
                }

                // Data & Misc
                Card(title="Data & Misc", wide=true) {

                    Row(gap=4, items="center", mb=4) {
                        { move || threadloom! { Hamburger(open={hamburger_open.get()}, on_toggle={move || set_hamburger_open.set(!hamburger_open.get())}) } }
                        { threadloom! { Tooltip(
                            tooltip_text="This is a tooltip!"
                        ) {
                            Text(variant="span", class="text-blue-500 underline cursor-help dark:text-blue-400 font-medium") { "Hover me" }
                        } } }
                    }

                    {
                        threadloom! { DataTable(
                            headers=vec!["Name".to_string(), "Status".to_string(), "Role".to_string()],
                            rows=vec![
                                vec![
                                    threadloom! { Text(variant="span", class="font-medium") { "Alice" } },
                                    threadloom! { Text(variant="span", class="px-2 py-1 rounded-full text-xs bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200") { "Active" } },
                                    threadloom! { "Admin" }
                                ],
                                vec![
                                    threadloom! { Text(variant="span", class="font-medium") { "Bob" } },
                                    threadloom! { Text(variant="span", class="px-2 py-1 rounded-full text-xs bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-300") { "Offline" } },
                                    threadloom! { "Editor" }
                                ],
                                vec![
                                    threadloom! { Text(variant="span", class="font-medium") { "Charlie" } },
                                    threadloom! { Text(variant="span", class="px-2 py-1 rounded-full text-xs bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200") { "Away" } },
                                    threadloom! { "Viewer" }
                                ],
                            ]
                        ) }
                    }

                    Column(mt=6, class="relative") {
                        Heading(level=4, mb=4, class="text-sm font-medium text-gray-700 dark:text-gray-300") { "Toasts" }
                        {
                            threadloom! { Button(label="Show Toast", primary=false, on_click={move || {}}) }
                        }
                        { toast_container(vec![
                            toast("Action completed successfully!")
                        ]) }
                    }
                }

                // New Features Demo
                Card(title="New Reactivity Features", wide=true) {

                    Grid(cols=1, md_cols=3, gap=8) {
                        // Global Signal
                        Column(gap=4) {
                            Heading(level=4, class="text-sm font-medium text-gray-700 dark:text-gray-300") { "Global Signals" }
                            Text(variant="div", class="text-2xl font-bold text-indigo-600 dark:text-indigo-400") { { move || USER_SCORE.get() } }
                            Button(label="Add 10 to Global", primary=false, on_click={move || USER_SCORE.update(|s| *s += 10)})
                        }

                        // Computed Signal
                        Column(gap=4) {
                            Heading(level=4, class="text-sm font-medium text-gray-700 dark:text-gray-300") { "Computed Signals" }
                            Row(gap=2, items="center") {
                                Text(variant="span", class="text-sm") { "Base: " }
                                Text(variant="strong") { { move || count.get() } }
                            }
                            Row(gap=2, items="center") {
                                Text(variant="span", class="text-sm") { "Doubled: " }
                                Text(variant="strong", class="text-emerald-600") { { move || doubled.get() } }
                            }
                            Button(label="Increment Base", primary=true, on_click={move || set_count.set(count.get() + 1)})
                        }

                        // Actions
                        Column(gap=4) {
                            Heading(level=4, class="text-sm font-medium text-gray-700 dark:text-gray-300") { "Actions & Loading State" }
                            {
                                let sa = submit_action.clone();
                                move || {
                                    if sa.is_loading() {
                                        threadloom! { Button(label="Loading...", primary=true, class="bg-gray-400 cursor-not-allowed border-gray-400", on_click={|| {}}) }
                                    } else {
                                        let a = action_clone.clone();
                                        let s = set_action_result.clone();
                                        threadloom! { Button(label="Run Action", primary=true, class="bg-emerald-600 hover:bg-emerald-700 border-emerald-600 text-white transition-all", on_click={
                                            move || {
                                                let a = a.clone();
                                                let s = s.clone();
                                                threadloom_dom::spawn!(async move {
                                                    let res = a.execute("Worked".to_string()).await;
                                                    s.set(res);
                                                });
                                            }
                                        }) }
                                    }
                                }
                            }
                            Text(variant="div", class="text-sm mt-2 min-h-[1.5rem]") { { move || action_result.get() } }
                        }
                    }
                }

                // Counter & API Demo
                Card(title="State & API", wide=true) {

                    Grid(cols=1, md_cols=2, gap=8) {
                        // Counter
                        Column(items="center", gap=4, p=6, border=1, border_color="gray-100", class="bg-gray-50 dark:bg-gray-900 rounded-lg dark:border-gray-800") {
                            Heading(level=4, align="center", class="text-lg font-medium text-gray-800 dark:text-gray-200") { "Counter Component" }
                            Text(variant="div", class="text-4xl font-bold text-blue-600 dark:text-blue-400 tabular-nums") {
                                { move || counter.get() }
                            }
                            Row(gap=2, mt=2) {
                                Button(label="-1", primary=false, on_click={move || { set_counter.set(counter.get() - 1); }})
                                Button(label="+1", primary=true, on_click={move || { set_counter.set(counter.get() + 1); }})
                            }
                        }

                        // API Fetch
                        Column(items="center", gap=4, p=6, border=1, border_color="gray-100", class="bg-gray-50 dark:bg-gray-900 rounded-lg dark:border-gray-800") {
                            Heading(level=4, align="center", class="text-lg font-medium text-gray-800 dark:text-gray-200") { "API Integration" }
                            Text(variant="p", class="text-sm text-gray-600 dark:text-gray-400 text-center min-h-[2.5rem] flex items-center italic") {
                                { move || api_response.get() }
                            }
                            Button(label="Fetch from Backend", primary=true, on_click={
                                let s1 = set_api_response.clone();
                                let s2 = set_api_response.clone();
                                move || {
                                    s1.set("Fetching...".to_string());
                                    let s_ok = s1.clone();
                                    let s_err = s2.clone();
                                    threadloom_dom::rpc!(hello(HelloArgs {
                                        name: "Developer".to_string(),
                                    }) => |msg| {
                                        s_ok.set(msg);
                                    }, |err| {
                                        s_err.set(format!("Error: {}", err));
                                    });
                                }
                            })
                        }
                    }
                }

                // Global Store Demo
                Card(title="Global Store Demo", wide=true) {
                    Column(gap=4) {
                        Label(text="Enter your name to test global state persistence:", r#for="name_input")
                        Input(
                            id="name_input",
                            class="tl-input border border-gray-300 dark:border-gray-700 rounded-md p-2 bg-transparent text-gray-900 dark:text-gray-100",
                            placeholder="Your Name",
                            value="",
                            on_input={move || {
                                let val = threadloom_dom::get_value!("name_input");
                                crate::store::GlobalState::set(val);
                            }}
                        )
                        Row {
                            Button(label="Go to /name", primary=true, on_click={|| {
                                crate::store::navigate("/name");
                            }})
                        }
                    }
                }

                // Cookie Management Demo
                Card(title="Cookie Management", wide=true) {
                    Column(gap=4) {
                        Text(variant="div", class="text-sm text-gray-600 dark:text-gray-400") {
                            "Current demo_cookie value: " { move || cookie_val.get() }
                        }
                        Input(
                            id="cookie_input",
                            class="tl-input border border-gray-300 dark:border-gray-700 rounded-md p-2 bg-transparent text-gray-900 dark:text-gray-100",
                            placeholder="Set cookie value",
                            value={threadloom_dom::get_cookie!("demo_cookie").unwrap_or_default()},
                            on_input={move || {
                                let val = threadloom_dom::get_value!("cookie_input");
                                set_cookie_val.set(val);
                            }}
                        )
                        Row(gap=2, mb=30) {
                            Button(label="Save Cookie", primary=true, on_click={move || {
                                threadloom_dom::set_cookie!("demo_cookie", cookie_val.get(), 3600);
                                threadloom_dom::alert!("Cookie saved for 1 hour!");
                            }})
                            Button(label="Clear Cookie", primary=false, on_click={move || {
                                set_cookie_val.set("".to_string());
                                threadloom_dom::set_cookie!("demo_cookie", "", 0);
                                threadloom_dom::alert!("Cookie cleared!");
                            }})
                        }
                    }
                }
            }
        }
    }
}
