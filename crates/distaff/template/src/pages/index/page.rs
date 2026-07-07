use threadloom_core::{create_signal, element, IntoView, View};
use threadloom_macro::threadloom;
use threadloom_ui::*;

pub fn page() -> View {
    let (count, set_count) = create_signal(0);
    let (rpc_msg, set_rpc_msg) = create_signal(String::new());
    let (is_loading, set_is_loading) = create_signal(false);

    threadloom! {
        Section(row=false, height="screen", width="full", class="bg-background text-foreground font-sans flex flex-col relative") {

            // Minimal Header
            Row(items="center", justify="between", class="w-full px-8 py-6 border-b border-border bg-background absolute top-0 z-50") {
                Row(items="center", gap=3) {
                    // Modern Layered SVG Logo
                    Image(src="/assets/favicon.svg", alt="Threadloom Logo", class="w-7 h-7") {}
                    Text(variant="span", class="font-bold text-lg tracking-tight ml-2") { "Threadloom" }
                }
                Row(items="center", gap=6, class="hidden sm:flex") {
                    Text(variant="a", href="https://distaff.vercel.app/docs", target="_blank", class="text-sm font-medium text-muted-foreground hover:text-foreground transition-colors cursor-pointer") { "Documentation" }
                    Text(variant="a", href="https://distaff.vercel.app/components", target="_blank", class="text-sm font-medium text-muted-foreground hover:text-foreground transition-colors cursor-pointer") { "Components" }
                    Text(variant="a", href="https://github.com/Parth3930/threadloom", target="_blank", class="text-sm font-medium text-muted-foreground hover:text-foreground transition-colors cursor-pointer") { "GitHub" }
                }
            }

            // Main Hero
            Section(row=false, class="flex-1 flex flex-col items-center justify-center w-full max-w-5xl mx-auto px-6 mt-[10rem] mb-12") {

                Heading(level=1, class="text-5xl sm:text-7xl md:text-8xl font-extrabold tracking-tighter text-center leading-none mb-8") {
                    "Build your next"
                    br() {}
                    "idea even faster."
                }

                Text(variant="p", class="text-lg md:text-xl text-muted-foreground text-center max-w-2xl font-normal mb-10 leading-relaxed") {
                    "A meticulously crafted Rust full-stack framework. Zero configuration. Maximum performance. Beautiful by default."
                }

                Row(items="center", justify="center", gap=4, class="w-full sm:w-auto flex-col sm:flex-row") {
                    Button(
                        label="Get Started",
                        primary=true,
                        class="px-8 py-6 rounded-md text-sm font-semibold shadow-sm hover:opacity-90 transition-opacity bg-foreground text-background w-full sm:w-auto border-0",
                        on_click=move || { let _ = threadloom_dom::window().location().set_href("https://distaff.vercel.app/docs/installation"); }
                    )
                    Button(
                        label="Read the Docs",
                        primary=false,
                        class="px-8 py-6 rounded-md text-sm font-medium bg-transparent border border-border hover:bg-muted text-foreground transition-colors w-full sm:w-auto shadow-sm",
                        on_click=move || { let _ = threadloom_dom::window().location().set_href("https://distaff.vercel.app/docs"); }
                    )
                }

                // Side-by-side Demos
                Grid(cols=1, md_cols=2, gap=6, class="mt-24 w-full max-w-4xl px-4") {
                    // Box 1: Interactive State
                    Section(row=false, class="w-full h-full border border-border rounded-xl bg-card shadow-sm p-8 flex flex-col items-center justify-between gap-6") {
                        Text(variant="span", class="text-xs font-bold text-muted-foreground uppercase tracking-[0.2em]") { "Interactive State" }

                        Text(variant="div", class="text-7xl font-light tabular-nums tracking-tighter text-foreground my-4") {
                            { move || count.get() }
                        }

                        Row(items="center", gap=4) {
                            Button(
                                label="-",
                                primary=false,
                                class="w-12 h-12 p-0 rounded-full border border-border bg-transparent hover:bg-muted text-foreground flex items-center justify-center text-xl transition-colors",
                                on_click=move || set_count.set(count.get() - 1)
                            )
                            Button(
                                label="+",
                                primary=false,
                                class="w-12 h-12 p-0 rounded-full border border-border bg-transparent hover:bg-muted text-foreground flex items-center justify-center text-xl transition-colors",
                                on_click=move || set_count.set(count.get() + 1)
                            )
                        }
                    }

                    // Box 2: Type-Safe RPC
                    Section(row=false, class="w-full h-full border border-border rounded-xl bg-card shadow-sm p-8 flex flex-col items-center justify-between gap-6") {
                        Text(variant="span", class="text-xs font-bold text-muted-foreground uppercase tracking-[0.2em]") { "Type-Safe RPC" }

                        Text(variant="p", class="text-center text-muted-foreground text-sm font-medium leading-relaxed max-w-[220px] my-auto") {
                            "Fetch a personalized greeting from the server using the current count."
                        }

                        Column(items="center", gap=3, class="w-full mt-4") {
                            Button(
                                label="",
                                primary=false,
                                class="px-6 py-2 rounded-full border border-border bg-transparent hover:bg-muted text-foreground text-sm font-medium transition-colors w-full max-w-[200px] flex items-center justify-center",
                                on_click=move || {
                                    if is_loading.get() { return; }
                                    let c = count.get();

                                    set_is_loading.set(true);

                                    wasm_bindgen_futures::spawn_local(async move {
                                        let msg = match crate::api::hello::route::hello(crate::api::hello::route::HelloArgs { name: format!("Count {}", c) }).await {
                                            Ok(res) => res,
                                            Err(e) => format!("Error: {}", e),
                                        };

                                        set_is_loading.set(false);
                                        set_rpc_msg.set(msg);

                                        // Flush reactivity — pending_boundaries don't auto-process after async boundary
                                        let _ = threadloom_dom::tick();
                                    });
                                }
                            ) {
                                { move || if is_loading.get() {
                                    element("div")
                                        .attr("class", "flex flex-row items-center justify-center gap-2")
                                        .child(
                                            element("div")
                                                .attr("class", "w-4 h-4 rounded-full border-2 border-current border-t-transparent animate-spin")
                                        )
                                        .child(element("span").child("Loading...".to_string()))
                                        .into_view()
                                } else {
                                    element("span").child("Greet from Server".to_string()).into_view()
                                } }
                            }

                            Text(variant="p", class="text-sm text-foreground font-mono font-semibold min-h-[1.5rem] text-center w-full truncate px-2") {
                                { move || rpc_msg.get() }
                            }
                        }
                    }
                }

                Text(variant="p", class="text-xs text-muted-foreground mt-16 font-mono") {
                    "src/pages/index/page.rs"
                }
            }
        }
    }
}
