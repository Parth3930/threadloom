use threadloom_core::{create_signal, IntoView, View};
use threadloom_macro::threadloom;
use threadloom_ui::*;

pub fn page() -> View {
    let (active_tool, set_active_tool) = create_signal("brush".to_string());
    let (active_color, set_active_color) = create_signal("#000000".to_string());

    #[cfg(target_arch = "wasm32")]
    super::client::init_board(active_tool.clone(), active_color.clone());

    threadloom! {
            Section(row=false, height="screen", width="full", class="bg-background text-foreground flex flex-col relative overflow-hidden") {
                // Header
                Row(items="center", justify="between", class="w-full px-6 py-4 border-b border-border bg-background shadow-sm z-20") {
                    Row(items="center", gap=3) {
                        Text(variant="span", class="font-bold tracking-tight") { "Collab Whiteboard" }
                        Text(variant="span", class="text-xs text-muted-foreground ml-4 hidden sm:block") { "Share the URL to collaborate" }
                    }
                    Button(
                        label="Leave Room",
                        primary=false,
                        class="px-4 py-1.5 text-sm rounded-md",
                        on_click=move || { threadloom_dom::redirect!("/"); }
                    )
                }

                // Toolbar (Floating)
                { move || {
                    let tool = active_tool.get();
                    let color = active_color.get();

                    let icon = |paths: &[(&'static str, &[(&'static str, &'static str)])]| -> View {
                        let mut svg = threadloom_core::element("svg")
                            .attr("xmlns", "http://www.w3.org/2000/svg")
                            .attr("width", "20")
                            .attr("height", "20")
                            .attr("viewBox", "0 0 24 24")
                            .attr("fill", "none")
                            .attr("stroke", "currentColor")
                            .attr("stroke-width", "2")
                            .attr("stroke-linecap", "round")
                            .attr("stroke-linejoin", "round");

                        for (tag, attrs) in paths {
                            let mut el = threadloom_core::element(*tag);
                            for (k, v) in *attrs {
                                el = el.attr(*k, *v);
                            }
                            svg = svg.child(el);
                        }
                        svg.into_view()
                    };

                    let active_btn = "rounded-full px-2 sm:px-4 py-2 !bg-primary !text-primary-foreground font-medium text-sm !border-none hover:!bg-primary hover:!text-primary-foreground !shadow-none flex items-center justify-center";
                    let inactive_btn = "rounded-full px-2 sm:px-4 py-2 !bg-transparent !text-foreground font-medium text-sm !border-none hover:!bg-transparent hover:!text-foreground !shadow-none hover:!shadow-none flex items-center justify-center";

                    let is_shape = tool == "shape" || tool == "rect" || tool == "circle" || tool == "triangle";
                    let brush_cls = if tool == "brush" { active_btn } else { inactive_btn };
                    let shape_cls = if is_shape { active_btn } else { inactive_btn };
                    let text_cls = if tool == "text" { active_btn } else { inactive_btn };
                    let eraser_cls = if tool == "eraser" { active_btn } else { inactive_btn };

                    let rect_cls = if tool == "rect" || tool == "shape" { active_btn } else { inactive_btn };
                    let circle_cls = if tool == "circle" { active_btn } else { inactive_btn };
                    let triangle_cls = if tool == "triangle" { active_btn } else { inactive_btn };

                    let color_btn_base = "w-6 h-6 rounded-full border-2 cursor-pointer !shadow-none hover:!shadow-none !p-0 min-w-0 flex-shrink-0";
                    let c_black = format!("{} !bg-black {}", color_btn_base, if color == "#000000" { "!border-primary scale-110" } else { "!border-transparent hover:!border-transparent" });
                    let c_red = format!("{} !bg-red-500 {}", color_btn_base, if color == "#ef4444" { "!border-primary scale-110" } else { "!border-transparent hover:!border-transparent" });
                    let c_blue = format!("{} !bg-blue-500 {}", color_btn_base, if color == "#3b82f6" { "!border-primary scale-110" } else { "!border-transparent hover:!border-transparent" });
                    let c_green = format!("{} !bg-green-500 {}", color_btn_base, if color == "#22c55e" { "!border-primary scale-110" } else { "!border-transparent hover:!border-transparent" });

                    threadloom! {
                        Section(row=false, class="absolute left-1/2 bottom-8 -translate-x-1/2 flex flex-col items-center gap-2 z-20") {
                            { move || {
                                if is_shape {
                                    threadloom! {
                                        Row(items="center", class="gap-1 sm:gap-2 bg-card border border-border shadow-lg rounded-full px-2 sm:px-4 py-2") {
                                            Button(label="", primary=false, class=rect_cls, on_click=move || set_active_tool.set("rect".to_string())) {
                                                { icon(&[("rect", &[("x", "3"), ("y", "3"), ("width", "18"), ("height", "18"), ("rx", "2"), ("ry", "2")])]) }
                                            }
                                            Button(label="", primary=false, class=circle_cls, on_click=move || set_active_tool.set("circle".to_string())) {
                                                { icon(&[("circle", &[("cx", "12"), ("cy", "12"), ("r", "10")])]) }
                                            }
                                            Button(label="", primary=false, class=triangle_cls, on_click=move || set_active_tool.set("triangle".to_string())) {
                                                { icon(&[("path", &[("d", "m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z")])]) }
                                            }
                                        }
                                    }.into_view()
                                } else {
                                    threadloom! { Section(row=false, class="hidden") {} }.into_view()
                                }
                            }}
                            Row(items="center", class="gap-1 sm:gap-2 bg-card border border-border shadow-lg rounded-full px-2 sm:px-4 py-2 sm:py-3") {
                                Button(label="", primary=false, class=brush_cls, on_click=move || set_active_tool.set("brush".to_string())) {
                                    { icon(&[("path", &[("d", "m9.06 11.9 8.07-8.06a2.85 2.85 0 1 1 4.03 4.03l-8.06 8.08")]), ("path", &[("d", "M7.07 14.94c-1.66 0-3 1.35-3 3.02 0 1.33-2.5 1.52-2 2.02 1.08 1.35 2.22 2.02 3 2.02 2.2 0 4-1.8 4-4.04a3.01 3.01 0 0 0-3-3.02z")])]) }
                                }
                                Button(label="", primary=false, class=shape_cls, on_click=move || set_active_tool.set("rect".to_string())) {
                                    { icon(&[("rect", &[("x", "3"), ("y", "3"), ("width", "18"), ("height", "18"), ("rx", "2"), ("ry", "2")])]) }
                                }
                                Button(label="", primary=false, class=text_cls, on_click=move || set_active_tool.set("text".to_string())) {
                                    { icon(&[("polyline", &[("points", "4 7 4 4 20 4 20 7")]), ("line", &[("x1", "9"), ("x2", "15"), ("y1", "20"), ("y2", "20")]), ("line", &[("x1", "12"), ("x2", "12"), ("y1", "4"), ("y2", "20")])]) }
                                }
                                Button(label="", primary=false, class=eraser_cls, on_click=move || set_active_tool.set("eraser".to_string())) {
                                    { icon(&[("path", &[("d", "m7 21-4.3-4.3c-1-1-1-2.5 0-3.4l9.6-9.6c1-1 2.5-1 3.4 0l5.6 5.6c1 1 1 2.5 0 3.4L13 21")]), ("path", &[("d", "M22 21H7")]), ("path", &[("d", "m5 11 9 9")])]) }
                                }

                                Section(row=false, class="w-px h-6 bg-border mx-2 sm:mx-2") {
    }

                                Row(items="center", class="gap-2 sm:gap-5") {
                                    Button(label="", primary=false, class=c_black, on_click=move || set_active_color.set("#00000".to_string())) {}
                                    Button(label="", primary=false, class=c_red, on_click=move || set_active_color.set("#ef4444".to_string())) {}
                                    Button(label="", primary=false, class=c_blue, on_click=move || set_active_color.set("#3b82f6".to_string())) {}
                                    Button(label="", primary=false, class=c_green, on_click=move || set_active_color.set("#22c55e".to_string())) {}
                                }
                            }
                        }
                    }
                }}

                // Canvas
                Section(row=false, class="flex-1 w-full relative bg-muted/10 z-10") {
                    { threadloom_core::element("canvas")
                        .attr("id", "board-canvas")
                        .attr("class", "absolute inset-0 w-full h-full cursor-none touch-none")
                        .into_view() }
                }
            }
        }
}
