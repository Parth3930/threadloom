use crate::{OptClass, Callback};
use threadloom_core::{IntoView, View, element, text, create_effect};

/// Properties for GlitchText
#[derive(Default)]
pub struct GlitchTextProps {
    pub text: String,
    pub class: OptClass,
    pub as_tag: OptClass,
    pub children: Vec<View>,
}

#[allow(non_snake_case)]
pub fn GlitchText(props: GlitchTextProps) -> View {
    let tag = props.as_tag.0.unwrap_or_else(|| "h2".to_string());
    let mut class_str = "glitch-text relative inline-block m-0".to_string();
    if let Some(c) = props.class.0 {
        class_str.push(' ');
        class_str.push_str(&c);
    }
    
    let id = crate::next_id();
    let text_content = props.text.clone();
    
    create_effect({
        let id = id.clone();
        let text = text_content.clone();
        move || {
            let script = format!(r#"
                setTimeout(() => {{
                    const el = document.getElementById('{id}');
                    if (el) {{
                        el.dataset.text = `{text}`;
                        if (!document.getElementById('tl-glitch-css')) {{
                            const style = document.createElement('style');
                            style.id = 'tl-glitch-css';
                            style.innerHTML = `
                                .glitch-text::before, .glitch-text::after {{
                                    content: attr(data-text);
                                    position: absolute;
                                    top: 0; left: 0; width: 100%; height: 100%;
                                    opacity: 0.8;
                                }}
                                .glitch-text::before {{
                                    left: 2px;
                                    text-shadow: -2px 0 red;
                                    clip: rect(44px, 450px, 56px, 0);
                                    animation: tl-glitch-anim 5s infinite linear alternate-reverse;
                                }}
                                .glitch-text::after {{
                                    left: -2px;
                                    text-shadow: -2px 0 blue;
                                    clip: rect(44px, 450px, 56px, 0);
                                    animation: tl-glitch-anim2 5s infinite linear alternate-reverse;
                                }}
                                @keyframes tl-glitch-anim {{
                                    0% {{ clip: rect(10px, 9999px, 83px, 0); }}
                                    20% {{ clip: rect(32px, 9999px, 14px, 0); }}
                                    40% {{ clip: rect(64px, 9999px, 98px, 0); }}
                                    60% {{ clip: rect(8px, 9999px, 44px, 0); }}
                                    80% {{ clip: rect(92px, 9999px, 5px, 0); }}
                                    100% {{ clip: rect(51px, 9999px, 73px, 0); }}
                                }}
                                @keyframes tl-glitch-anim2 {{
                                    0% {{ clip: rect(82px, 9999px, 12px, 0); }}
                                    20% {{ clip: rect(4px, 9999px, 66px, 0); }}
                                    40% {{ clip: rect(38px, 9999px, 91px, 0); }}
                                    60% {{ clip: rect(18px, 9999px, 34px, 0); }}
                                    80% {{ clip: rect(74px, 9999px, 88px, 0); }}
                                    100% {{ clip: rect(21px, 9999px, 57px, 0); }}
                                }}
                            `;
                            document.head.appendChild(style);
                        }}
                    }}
                }}, 50);
            "#);
            crate::run_animation(script);
        }
    });

    element(tag)
        .attr("id", id)
        .attr("class", class_str)
        .child(text(text_content))
        .into_view()
}

/// Properties for GradientText
#[derive(Default)]
pub struct GradientTextProps {
    pub text: String,
    pub class: OptClass,
    pub as_tag: OptClass,
    pub from_color: OptClass,
    pub to_color: OptClass,
    pub children: Vec<View>,
}

#[allow(non_snake_case)]
pub fn GradientText(props: GradientTextProps) -> View {
    let tag = props.as_tag.0.unwrap_or_else(|| "span".to_string());
    let mut class_str = "bg-clip-text text-transparent bg-gradient-to-r".to_string();
    
    let from_c = props.from_color.0.unwrap_or_else(|| "from-primary".to_string());
    let to_c = props.to_color.0.unwrap_or_else(|| "to-secondary".to_string());
    
    class_str.push_str(" ");
    class_str.push_str(&from_c);
    class_str.push_str(" ");
    class_str.push_str(&to_c);

    if let Some(c) = props.class.0 {
        class_str.push(' ');
        class_str.push_str(&c);
    }
    
    element(tag)
        .attr("class", class_str)
        .child(text(props.text))
        .into_view()
}

/// Properties for TypingText
#[derive(Default)]
pub struct TypingTextProps {
    pub text: String,
    pub class: OptClass,
    pub as_tag: OptClass,
    pub speed: i32,
    pub children: Vec<View>,
}

#[allow(non_snake_case)]
pub fn TypingText(props: TypingTextProps) -> View {
    let tag = props.as_tag.0.unwrap_or_else(|| "span".to_string());
    let mut class_str = "tl-typing-text".to_string();
    if let Some(c) = props.class.0 {
        class_str.push(' ');
        class_str.push_str(&c);
    }
    
    let id = crate::next_id();
    let text_content = props.text.clone();
    let speed = if props.speed > 0 { props.speed } else { 50 };
    
    create_effect({
        let id = id.clone();
        let text = text_content.clone();
        move || {
            let script = format!(r#"
                setTimeout(() => {{
                    const el = document.getElementById('{id}');
                    if (el) {{
                        const fullText = `{text}`;
                        el.innerText = '';
                        let i = 0;
                        const typeWriter = () => {{
                            if (i < fullText.length) {{
                                el.innerHTML += fullText.charAt(i) === '\\n' ? '<br/>' : fullText.charAt(i);
                                i++;
                                setTimeout(typeWriter, {speed});
                            }}
                        }};
                        typeWriter();
                        
                        if (!document.getElementById('tl-typing-css')) {{
                            const style = document.createElement('style');
                            style.id = 'tl-typing-css';
                            style.innerHTML = `
                                .tl-typing-text::after {{
                                    content: '|';
                                    animation: tl-blink 1s step-end infinite;
                                }}
                                @keyframes tl-blink {{ 50% {{ opacity: 0; }} }}
                            `;
                            document.head.appendChild(style);
                        }}
                    }}
                }}, 50);
            "#);
            crate::run_animation(script);
        }
    });

    element(tag)
        .attr("id", id)
        .attr("class", class_str)
        .into_view()
}

/// Properties for SplitText
#[derive(Default)]
pub struct SplitTextProps {
    pub text: String,
    pub class: OptClass,
    pub as_tag: OptClass,
    pub children: Vec<View>,
}

#[allow(non_snake_case)]
pub fn SplitText(props: SplitTextProps) -> View {
    let tag = props.as_tag.0.unwrap_or_else(|| "span".to_string());
    let mut class_str = "tl-split-text".to_string();
    if let Some(c) = props.class.0 {
        class_str.push(' ');
        class_str.push_str(&c);
    }
    
    let id = crate::next_id();
    let text_content = props.text.clone();
    
    create_effect({
        let id = id.clone();
        let text = text_content.clone();
        move || {
            let script = format!(r#"
                setTimeout(() => {{
                    const el = document.getElementById('{id}');
                    if (el && window.gsap) {{
                        const fullText = `{text}`;
                        el.innerHTML = '';
                        fullText.split('').forEach(char => {{
                            const span = document.createElement('span');
                            span.style.display = 'inline-block';
                            span.style.opacity = '0';
                            span.innerHTML = char === ' ' ? '&nbsp;' : char;
                            el.appendChild(span);
                        }});
                        
                        gsap.to(el.querySelectorAll('span'), {{
                            opacity: 1,
                            y: 0,
                            yFrom: 20,
                            duration: 0.5,
                            stagger: 0.05,
                            ease: "back.out(1.7)"
                        }});
                    }} else if (el) {{
                        el.innerText = `{text}`;
                    }}
                }}, 50);
            "#);
            crate::run_animation(script);
        }
    });

    element(tag)
        .attr("id", id)
        .attr("class", class_str)
        .into_view()
}
