use threadloom_core::{View, create_signal};
use threadloom_macro::threadloom;
use threadloom_ui::{button, input, checkbox, radio, accordion, textarea, label, dropdown, select, toast, toast_container, tabs, data_table, tooltip, hamburger, dialog};

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
    
    threadloom! {
        div(class="flex flex-col gap-8") {
            div(class="grid grid-cols-1 md:grid-cols-2 gap-8") {
                
                // Forms & Inputs
                div(class="flex flex-col gap-6 bg-white dark:bg-gray-800 p-6 shadow-sm border border-gray-200 dark:border-gray-800 rounded-xl transition-colors duration-300 tl-card") {
                    h3(class="text-xl font-medium text-gray-800 dark:text-gray-100 border-b dark:border-gray-800 pb-2") { "Forms & Inputs" }
                    h4(class="text-sm font-medium text-gray-700 dark:text-gray-300 mt-2 mb-1") { "Buttons" }
                    div(class="flex gap-4 items-center") {
                        { button("Primary Button", true, || { 
                            let _ = web_sys::window().unwrap().alert_with_message("Primary Button Clicked!"); 
                        }) }
                        { button("Secondary Button", false, || {
                            let _ = web_sys::window().unwrap().alert_with_message("Secondary Button Clicked!"); 
                        }) }
                    }
                }

                // Text Inputs
                div(class="flex flex-col gap-6 bg-white dark:bg-gray-800 p-6 shadow-sm border border-gray-200 dark:border-gray-800 rounded-xl transition-colors duration-300 tl-card") {
                    h3(class="text-xl font-medium text-gray-800 dark:text-gray-100 border-b dark:border-gray-800 pb-2") { "Text Inputs" }
                    div(class="flex flex-col gap-2") {
                        div(class="flex flex-col gap-1") {
                            { label("Username", "user") }
                            { input("", "Enter your username...", || {}) }
                        }
                        { textarea("", "Write a message...", || {}) }
                    }
                }

                // Selections
                div(class="flex flex-col gap-6 bg-white dark:bg-gray-800 p-6 shadow-sm border border-gray-200 dark:border-gray-800 rounded-xl transition-colors duration-300 tl-card") {
                    h3(class="text-xl font-medium text-gray-800 dark:text-gray-100 border-b dark:border-gray-800 pb-2") { "Selections" }
                    div(class="flex items-center gap-2") {
                        { 
                            let terms = terms_agreed.clone();
                            let set_terms = set_terms_agreed.clone();
                            move || {
                                let t = terms.clone();
                                let st = set_terms.clone();
                                checkbox(t.get(), "terms", move || { st.set(!t.get()); })
                            }
                        }
                        { label("Accept Terms & Conditions", "terms") }
                    }
                    div(class="flex gap-4") {
                        div(class="flex items-center gap-2") {
                            {
                                let opt = selected_option.clone();
                                let set_opt = set_selected_option.clone();
                                move || {
                                    let o = opt.clone();
                                    let so = set_opt.clone();
                                    radio(o.get() == 1, "opt1", "options", move || { so.set(1); })
                                }
                            }
                            { label("Option 1", "opt1") }
                        }
                        div(class="flex items-center gap-2") {
                            {
                                let opt = selected_option.clone();
                                let set_opt = set_selected_option.clone();
                                move || {
                                    let o = opt.clone();
                                    let so = set_opt.clone();
                                    radio(o.get() == 2, "opt2", "options", move || { so.set(2); })
                                }
                            }
                            { label("Option 2", "opt2") }
                        }
                    }
                }
                
                // Interactive
                div(class="flex flex-col gap-6 bg-white dark:bg-gray-800 p-6 shadow-sm border border-gray-200 dark:border-gray-800 rounded-xl transition-colors duration-300 tl-card") {
                    h3(class="text-xl font-medium text-gray-800 dark:text-gray-100 border-b dark:border-gray-800 pb-2") { "Interactive" }
                    { 
                        let acc = accordion_open.clone();
                        let set_acc = set_accordion_open.clone();
                        move || {
                            let a = acc.clone();
                            let sa = set_acc.clone();
                            accordion(
                                "Click to Expand Accordion",
                                a.get(),
                                threadloom! { p(class="p-4 text-gray-600 dark:text-gray-400") { "Accordion content here! Very premium and smooth." } },
                                move || { sa.set(!a.get()); },
                                ()
                            )
                        }
                    }
                    div(class="mt-4 flex flex-col gap-4") {
                        { 
                            let drop = dropdown_open.clone();
                            let set_drop = set_dropdown_open.clone();
                            move || {
                                let d = drop.clone();
                                let sd = set_drop.clone();
                                dropdown(
                                    "Open Menu Dropdown",
                                    d.get(),
                                    vec![
                                        threadloom! { button(class="block px-4 py-2 hover:bg-gray-100 dark:hover:bg-gray-700 w-full text-left dark:text-gray-200") { "Profile" } },
                                        threadloom! { button(class="block px-4 py-2 hover:bg-gray-100 dark:hover:bg-gray-700 w-full text-left dark:text-gray-200") { "Settings" } }
                                    ],
                                    move || { sd.set(!d.get()); }
                                )
                            }
                        }
                        
                        div(class="mt-2") {
                            { 
                                let set_dlg = set_dialog_open.clone();
                                button("Open Modal", false, move || {
                                    set_dlg.set(true); 
                                }) 
                            }
                        }
                        { 
                            let dlg = dialog_open.clone();
                            let set_dlg2 = set_dialog_open.clone();
                            move || {
                                let d = dlg.clone();
                                let sd = set_dlg2.clone();
                                dialog(
                                    d.get(),
                                    "Example Modal",
                                    threadloom! {
                                        p(class="text-gray-600 dark:text-gray-400 py-4") {
                                            "This is a premium modal built with Threadloom UI."
                                        }
                                    },
                                    move || { sd.set(false); }
                                )
                            }
                        }
                    }
                }
                
                // More Interactive
                div(class="flex flex-col gap-6 bg-white dark:bg-gray-800 p-6 shadow-sm border border-gray-200 dark:border-gray-800 rounded-xl transition-colors duration-300 tl-card") {
                    h3(class="text-xl font-medium text-gray-800 dark:text-gray-100 border-b dark:border-gray-800 pb-2") { "More Interactive" }
                    
                    h4(class="text-sm font-medium text-gray-700 dark:text-gray-300 mt-2 mb-1") { "Select & Tabs" }
                    { 
                        let sel = select_val.clone();
                        let set_sel = set_select_val.clone();
                        move || {
                            let s = sel.clone();
                            let ss = set_sel.clone();
                            select(
                                vec![
                                    ("1".to_string(), "Option 1".to_string()),
                                    ("2".to_string(), "Option 2".to_string()),
                                ],
                                s.get(),
                                move || { 
                                    let next = if s.get() == "1" { "2" } else { "1" };
                                    ss.set(next.to_string()); 
                                } 
                            )
                        }
                    }
                    div(class="mt-6") {
                        { 
                            let tab = active_tab.clone();
                            let set_tab = set_active_tab.clone();
                            move || {
                                let t = tab.clone();
                                let st = set_tab.clone();
                                tabs(
                                    vec!["Tab 1".to_string(), "Tab 2".to_string()],
                                    t.get(),
                                    move |idx| { st.set(idx); },
                                    vec![
                                        threadloom! { p(class="p-4 text-gray-600 dark:text-gray-400") { "This is the content for Tab 1. Very clean." } },
                                        threadloom! { p(class="p-4 text-gray-600 dark:text-gray-400") { "This is the content for Tab 2. Much wow." } }
                                    ]
                                )
                            }
                        }
                    }
                }

                // Data & Misc
                div(class="flex flex-col gap-6 bg-white dark:bg-gray-800 p-6 shadow-sm border border-gray-200 dark:border-gray-800 rounded-xl transition-colors duration-300 tl-card md:col-span-2") {
                    h3(class="text-xl font-medium text-gray-800 dark:text-gray-100 border-b dark:border-gray-800 pb-2") { "Data & Misc" }
                    
                    div(class="flex gap-4 items-center mb-4") {
                        { 
                            let ham = hamburger_open.clone();
                            let set_ham = set_hamburger_open.clone();
                            move || {
                                let h = ham.clone();
                                let sh = set_ham.clone();
                                hamburger(h.get(), move || { sh.set(!h.get()); }, ()) 
                            }
                        }
                        { tooltip(
                            threadloom! { span(class="text-blue-500 underline cursor-help dark:text-blue-400 font-medium") { "Hover me" } },
                            "This is a tooltip!"
                        ) }
                    }

                    { 
                        data_table(
                            vec!["ID".to_string(), "Name".to_string(), "Role".to_string()],
                            vec![
                                vec![threadloom! { span(class="dark:text-gray-300") { "1" } }, threadloom! { span(class="dark:text-gray-300 font-medium") { "Alice" } }, threadloom! { span(class="dark:text-gray-300") { "Admin" } }],
                                vec![threadloom! { span(class="dark:text-gray-300") { "2" } }, threadloom! { span(class="dark:text-gray-300 font-medium") { "Bob" } }, threadloom! { span(class="dark:text-gray-300") { "User" } }]
                            ]
                        ) 
                    }

                    div(class="mt-6 relative") {
                        h4(class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-4") { "Toasts" }
                        { 
                            button("Show Toast", false, move || {
                                // Normally we'd spawn a toast here, but for demo we just show the static container
                            }) 
                        }
                        { toast_container(vec![
                            toast("Action completed successfully!")
                        ]) }
                    }
                }

                // Counter & API Demo
                div(class="flex flex-col gap-6 bg-white dark:bg-gray-800 p-6 shadow-sm border border-gray-200 dark:border-gray-800 rounded-xl transition-colors duration-300 tl-card md:col-span-2") {
                    h3(class="text-xl font-medium text-gray-800 dark:text-gray-100 border-b dark:border-gray-800 pb-2") { "State & API" }
                    
                    div(class="grid grid-cols-1 md:grid-cols-2 gap-8") {
                        // Counter
                        div(class="flex flex-col items-center gap-4 p-6 bg-gray-50 dark:bg-gray-900 rounded-lg border border-gray-100 dark:border-gray-800") {
                            h4(class="text-lg font-medium text-gray-800 dark:text-gray-200") { "Counter Component" }
                            div(class="text-4xl font-bold text-blue-600 dark:text-blue-400 tabular-nums") {
                                { 
                                    let cnt = counter.clone();
                                    move || threadloom! { span() { { cnt.get().to_string() } } }
                                }
                            }
                            div(class="flex gap-2 mt-2") {
                                {
                                    let cnt = counter.clone();
                                    let set_cnt = set_counter.clone();
                                    button("-1", false, move || { set_cnt.set(cnt.get() - 1); })
                                }
                                {
                                    let cnt = counter.clone();
                                    let set_cnt = set_counter.clone();
                                    button("+1", true, move || { set_cnt.set(cnt.get() + 1); })
                                }
                            }
                        }

                        // API Fetch
                        div(class="flex flex-col items-center gap-4 p-6 bg-gray-50 dark:bg-gray-900 rounded-lg border border-gray-100 dark:border-gray-800") {
                            h4(class="text-lg font-medium text-gray-800 dark:text-gray-200") { "API Integration" }
                            p(class="text-sm text-gray-600 dark:text-gray-400 text-center min-h-[2.5rem] flex items-center italic") {
                                {
                                    let resp = api_response.clone();
                                    move || threadloom! { span() { { resp.get() } } }
                                }
                            }
                            {
                                let set_resp = set_api_response.clone();
                                button("Fetch from Backend", true, move || {
                                    set_resp.set("Fetching...".to_string());
                                    let sr = set_resp.clone();
                                    wasm_bindgen_futures::spawn_local(async move {
                                        match reqwasm::http::Request::get("/api/hello").send().await {
                                            Ok(resp) => {
                                                web_sys::console::log_1(&format!("Response status: {}", resp.status()).into());
                                                match resp.text().await {
                                                    Ok(text) => {
                                                        web_sys::console::log_1(&format!("Response text: {}", text).into());
                                                        sr.set(text);
                                                        let _ = threadloom_dom::tick();
                                                    }
                                                    Err(e) => {
                                                        web_sys::console::log_1(&format!("Error parsing text: {:?}", e).into());
                                                        sr.set("Error reading response".to_string());
                                                        let _ = threadloom_dom::tick();
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                web_sys::console::log_1(&format!("Fetch error: {:?}", e).into());
                                                sr.set(format!("Error: {:?}", e));
                                                let _ = threadloom_dom::tick();
                                            }
                                        }
                                    });
                                })
                            }
                        }
                    }
                }
            }
        }
    }
}
