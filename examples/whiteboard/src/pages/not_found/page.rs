use threadloom_core::View;
use threadloom_macro::threadloom;
use threadloom_ui::*;

pub fn page() -> View {
    threadloom! {
        Section(row=false, class="min-h-screen w-full flex flex-col items-center justify-center bg-background text-foreground font-sans selection:bg-foreground selection:text-background") {
            Column(items="center", gap=6, class="max-w-lg text-center p-8") {
                Heading(level=1, class="text-8xl sm:text-9xl font-bold tracking-tighter text-foreground leading-none") {
                    "404"
                }

                Text(variant="p", class="text-xl sm:text-2xl text-muted-foreground font-medium mt-4") {
                    "Page not found."
                }

                Text(variant="p", class="text-base sm:text-lg text-muted-foreground mb-8 leading-relaxed max-w-md") {
                    "The page you are looking for might have been removed, had its name changed, or is temporarily unavailable."
                }

                Row(items="center", justify="center", gap=4, class="w-full sm:w-auto flex-col sm:flex-row") {
                    Button(
                        label="Go Back",
                        primary=false,
                        class="px-8 py-5 rounded-none text-sm font-medium border border-border hover:border-foreground text-foreground bg-transparent transition-colors shadow-none w-full sm:w-auto",
                        on_click=move || {
                            if let Some(w) = web_sys::window() {
                                if let Ok(h) = w.history() {
                                    let _ = h.back();
                                }
                            }
                        }
                    )
                    Button(
                        label="Go Home",
                        primary=true,
                        class="px-8 py-5 rounded-none text-sm font-medium border border-foreground bg-foreground text-background hover:bg-muted-foreground transition-colors shadow-none w-full sm:w-auto",
                        on_click=move || threadloom::navigate!("/")
                    )
                }
            }
        }
    }
}
