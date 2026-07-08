use threadloom_core::{create_signal, IntoView, View};
use threadloom_macro::threadloom;
use threadloom_ui::*;

pub fn page() -> View {
    let (error_msg, set_error_msg) = create_signal(String::new());

    threadloom! {
        Section(row=false, height="screen", width="full", class="bg-background text-foreground flex flex-col items-center justify-center relative px-4") {
            // Header/Logo
            Row(items="center", gap=3, class="absolute top-6 left-6") {
                Image(src="/assets/favicon.svg", alt="Collab Whiteboard Logo", class="w-8 h-8") {}
                Text(variant="span", class="font-bold text-xl tracking-tight") { "Collab Whiteboard" }
            }

            // Login Card
            Section(row=false, class="w-full max-w-sm border border-border rounded-xl bg-card shadow-sm p-8 flex flex-col gap-6") {
                Heading(level=2, class="text-3xl font-extrabold tracking-tight text-center") {
                    "Welcome back"
                }
                
                Text(variant="p", class="text-muted-foreground text-center text-sm") {
                    "Log in to access your boards"
                }

                Column(items="center", gap=4, class="w-full mt-4") {
                    Input(
                        id="login_username",
                        placeholder="Username",
                        class="w-full"
                    ) {}
                    
                    { threadloom_core::element("input")
                        .attr("type", "password")
                        .attr("id", "login_password")
                        .attr("placeholder", "Password")
                        .attr("class", "flex w-full rounded-none border border-input bg-transparent px-3 py-2 text-sm shadow-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring text-foreground min-h-[2.5rem]")
                        .into_view() }

                    Button(
                        label="Log In",
                        primary=true,
                        class="w-full py-2 flex items-center justify-center text-sm font-semibold",
                        on_click=move || {
                            let username = threadloom_dom::get_value!("login_username");
                            let password = threadloom_dom::get_value!("login_password");
                            
                            threadloom_dom::wasm_bindgen_futures::spawn_local(async move {
                                match crate::api::login::route::login(crate::api::login::route::AuthArgs { username, password }).await {
                                    Ok(token) => {
                                        threadloom_dom::set_cookie!("auth_token", token.clone(), 60*60*24*7);
                                        crate::store::AuthState::set(token);
                                        threadloom_dom::redirect!("/");
                                    },
                                    Err(e) => set_error_msg.set(e),
                                }
                                let _ = threadloom_dom::tick();
                            });
                        }
                    )
                }

                // Error Message
                { move || {
                    let msg = error_msg.get();
                    if msg.is_empty() {
                        threadloom_core::element("div").into_view()
                    } else {
                        threadloom_core::element("p").attr("class", "text-destructive text-sm text-center font-medium mt-2").child(msg).into_view()
                    }
                }}
                
                Row(justify="center", class="mt-2 text-sm text-muted-foreground") {
                    Text(variant="span") { "Don't have an account? " }
                    { threadloom_core::element("a").attr("href", "/signup").attr("class", "text-primary hover:underline ml-1 font-medium").child("Sign up").into_view() }
                }
            }
        }
    }
}
