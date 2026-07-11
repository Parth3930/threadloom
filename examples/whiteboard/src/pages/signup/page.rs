use threadloom_core::{create_signal, IntoView, View};
use threadloom_macro::threadloom;
use threadloom_ui::*;

pub fn page() -> View {
    let (error_msg, set_error_msg) = create_signal(String::new());
    let form_ctx = FormContext::new();
    let form_ctx_submit = form_ctx.clone();

    threadloom! {
        Section(row=false, height="screen", width="full", class="bg-background text-foreground flex flex-col items-center justify-center relative px-4") {
            // Header/Logo
            Row(items="center", gap=3, class="absolute top-6 left-6") {
                Image(src="/assets/favicon.svg", alt="Collab Whiteboard Logo", class="w-8 h-8") {}
                Text(variant="span", class="font-bold text-xl tracking-tight") { "Collab Whiteboard" }
            }

            // Signup Card
            Section(row=false, class="w-full max-w-sm border border-border rounded-xl bg-card shadow-sm p-8 flex flex-col gap-6") {
                Heading(level=2, class="text-3xl font-extrabold tracking-tight text-center") {
                    "Create Account"
                }
                
                Text(variant="p", class="text-muted-foreground text-center text-sm") {
                    "Sign up to save your whiteboard sessions"
                }

                Form(class="w-full mt-4 flex flex-col items-center gap-4", on_submit=move || {
                    form_ctx_submit.clear_all();
                    set_error_msg.set(String::new());
                    
                    let username = threadloom_dom::get_value!("signup_username");
                    let password = threadloom_dom::get_value!("signup_password");
                    
                    let mut has_err = false;
                    if username.trim().is_empty() {
                        form_ctx_submit.set_error("signup_username", "Username is required");
                        has_err = true;
                    }
                    if password.len() < 6 {
                        form_ctx_submit.set_error("signup_password", "Password must be at least 6 characters");
                        has_err = true;
                    }
                    
                    if has_err {
                        let _ = threadloom_dom::tick();
                        return;
                    }
                    
                    threadloom_dom::wasm_bindgen_futures::spawn_local(async move {
                        match crate::api::signup::route::signup(crate::api::signup::route::AuthArgs { username, password }).await {
                            Ok(token) => {
                                threadloom_dom::set_cookie!("auth_token", token.clone(), 60*60*24*7);
                                crate::store::AuthState::set(token);
                                threadloom_dom::redirect!("/");
                            },
                            Err(e) => set_error_msg.set(e),
                        }
                        let _ = threadloom_dom::tick();
                    });
                }) {
                    FormField(
                        id="signup_username",
                        placeholder="Username",
                        context=form_ctx.clone()
                    ) {}
                    
                    FormField(
                        id="signup_password",
                        type_="password",
                        placeholder="Password",
                        context=form_ctx.clone()
                    ) {}

                    Button(
                        label="Sign Up",
                        primary=true,
                        class="w-full py-2 flex items-center justify-center text-sm font-semibold mt-2"
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
                    Text(variant="span") { "Already have an account? " }
                    { threadloom_core::element("a").attr("href", "/login").attr("class", "text-primary hover:underline ml-1 font-medium").child("Log in").into_view() }
                }
            }
        }
    }
}
