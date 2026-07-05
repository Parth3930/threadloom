use super::components::demo::demo_component;
use threadloom_core::{create_effect, create_signal, IntoView, View};
use threadloom_macro::threadloom;

pub fn page() -> View {
    let (is_dark, set_dark) = create_signal(false);
    let dark_signal = is_dark.clone();
    let is_dark_click = is_dark.clone();

    create_effect(move || {
        let is_dark = dark_signal.get();
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(html) = document.document_element() {
                    if is_dark {
                        let _ = html.set_attribute("class", "dark");
                    } else {
                        let _ = html.remove_attribute("class");
                    }
                }
            }
        }
    });

    threadloom! {
        div(class="min-h-screen bg-white dark:bg-gray-950 text-gray-900 dark:text-gray-50 font-sans transition-colors duration-300") {
            header(class="sticky top-0 z-40 border-b border-gray-200 dark:border-gray-800/50 bg-white/80 dark:bg-gray-950/80 backdrop-blur-md p-4 flex justify-between items-center transition-colors duration-300") {
                div(class="flex items-center gap-2") {
                    h1(class="text-2xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-blue-600 to-purple-600 dark:from-blue-400 dark:to-purple-400") { "Threadloom" }
                    span(class="px-2 py-0.5 rounded-full bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 text-xs font-semibold") { "UI" }
                }
                div(class="flex gap-4 items-center") {
                    a(href="https://github.com/Parth3930/threadloom", class="hover:text-blue-500 font-medium") { "GitHub" }
                    a(href="#", class="hover:text-blue-500 font-medium") { "Get Started" }
                    button(
                        class="p-2 rounded-full bg-gray-200 dark:bg-gray-800 hover:bg-gray-300 dark:hover:bg-gray-700 transition-colors text-gray-800 dark:text-gray-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500",
                        on_click=move || { set_dark.set(!is_dark_click.get()); }
                    ) {
                        {
                            let dark_svg = is_dark.clone();
                            move || {
                                if dark_svg.get() {
                                    threadloom! {
                                        svg(xmlns="http://www.w3.org/2000/svg", viewBox="0 0 24 24", class="svg-icon w-5 h-5") {
                                            circle(cx="12", cy="12", r="4") {}
                                            path(d="M12 2v2") {}
                                            path(d="M12 20v2") {}
                                            path(d="m4.93 4.93 1.41 1.41") {}
                                            path(d="m17.66 17.66 1.41 1.41") {}
                                            path(d="M2 12h2") {}
                                            path(d="M20 12h2") {}
                                            path(d="m6.34 17.66-1.41 1.41") {}
                                            path(d="m19.07 4.93-1.41 1.41") {}
                                        }
                                    }
                                } else {
                                    threadloom! {
                                        svg(xmlns="http://www.w3.org/2000/svg", viewBox="0 0 24 24", class="svg-icon w-5 h-5") {
                                            path(d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z") {}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            main(class="container mx-auto p-8 max-w-5xl flex flex-col gap-12") {
                div(class="py-16 md:py-24 flex flex-col items-center text-center gap-6") {
                    h2(class="text-5xl md:text-7xl font-extrabold tracking-tight text-gray-900 dark:text-white") {
                        "Build faster with "
                        span(class="text-transparent bg-clip-text bg-gradient-to-r from-blue-600 to-indigo-600 dark:from-blue-400 dark:to-indigo-400") { "Threadloom" }
                    }
                    p(class="text-xl md:text-2xl text-gray-600 dark:text-gray-400 max-w-2xl mx-auto font-light") {
                        "A premium, fast, and ergonomic UI framework for Rust and WebAssembly."
                    }
                    div(class="flex gap-4 mt-4") {
                        button(class="tl-btn tl-btn-primary px-8 text-lg rounded-full") { "Get Started" }
                        button(class="tl-btn tl-btn-secondary px-8 text-lg rounded-full") { "Documentation" }
                    }
                }
                div(class="relative") {
                    div(class="absolute inset-0 bg-gradient-to-tr from-blue-100/50 to-purple-100/50 dark:from-blue-900/20 dark:to-purple-900/20 rounded-3xl blur-3xl -z-10") {}
                    { demo_component() }
                }
            }
        }
    }
}
