use super::components::demo::demo_component;
use threadloom_core::{create_effect, create_signal, IntoView, View};
use threadloom_macro::threadloom;
use threadloom_ui::*;

pub fn page() -> View {
    let (is_dark, set_dark) = create_signal(true);

    create_effect(move || {
        threadloom_dom::toggle_html_class("dark", is_dark.get());
    });

    threadloom! {
        Section(row=false, height="screen", class="bg-white dark:bg-gray-950 text-gray-900 dark:text-gray-50 font-sans transition-colors duration-300") {
            Row(p=4, justify="between", items="center", width="full", class="sticky top-0 z-40 border-b border-gray-200 dark:border-gray-800/50 bg-white/80 dark:bg-gray-950/80 backdrop-blur-md transition-colors duration-300") {
                Row(items="center", gap=2) {
                    Heading(level=1, weight="bold", class="text-2xl bg-clip-text text-transparent bg-gradient-to-r from-blue-600 to-purple-600 dark:from-blue-400 dark:to-purple-400") { "Threadloom" }
                    Text(variant="span", weight="semibold", class="px-2 py-1 rounded-full bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 text-xs") { "UI" }
                }
                Row(items="center", gap=4) {
                    Text(variant="a", class="hover:text-blue-500 font-medium") { "GitHub" }
                    Text(variant="a", class="hover:text-blue-500 font-medium") { "Get Started" }
                    Button(
                        label="",
                        primary=false,
                        class="p-2 rounded-full bg-gray-200 dark:bg-gray-800 hover:bg-gray-300 dark:hover:bg-gray-700 transition-colors text-gray-800 dark:text-gray-200 border-none shadow-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500",
                        on_click=move || { set_dark.set(!is_dark.get()); }
                    ) {
                        {
                            move || {
                                if is_dark.get() {
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
            Section(width="full", p=8, gap=12, class="mx-auto max-w-5xl") {
                Section(row=false, py=16, items="center", gap=6, class="md:py-24 text-center") {
                    Heading(level=2, weight="extrabold", class="text-5xl md:text-7xl tracking-tight text-gray-900 dark:text-white") {
                        "Build faster with "
                        Text(variant="span", class="text-transparent bg-clip-text bg-gradient-to-r from-blue-600 to-indigo-600 dark:from-blue-400 dark:to-indigo-400") { "Threadloom" }
                    }
                    Text(variant="p", weight="light", class="mx-auto text-xl md:text-2xl text-gray-600 dark:text-gray-400 max-w-2xl") {
                        "A premium, fast, and ergonomic full-stack UI framework for Rust and WebAssembly."
                    }
                    Row(gap=4, mt=4) {
                        Button(label="Get Started", primary=true, class="px-8 text-lg rounded-full")
                        Button(label="Documentation", primary=false, class="px-8 text-lg rounded-full")
                    }
                }
                Section(row=false, class="relative") {
                    Section(row=false, class="absolute inset-0 bg-gradient-to-tr from-blue-100/50 to-purple-100/50 dark:from-blue-900/20 dark:to-purple-900/20 rounded-3xl blur-3xl -z-10") {}
                    { demo_component() }
                }
            }
        }
    }
}
