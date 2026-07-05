use threadloom_core::View;
use threadloom_macro::threadloom;

pub fn page() -> View {
    let name = crate::store::GlobalState::get();
    let display_text = if name.is_empty() {
        "Empty".to_string()
    } else {
        format!("Hello, {}!", name)
    };

    threadloom! {
        div(class="min-h-screen bg-gray-50 dark:bg-gray-900 text-gray-900 dark:text-gray-100 p-8 flex flex-col items-center justify-center gap-4") {
            h1(class="text-4xl font-bold text-blue-600 dark:text-blue-400") {
                { display_text }
            }
            button(class="px-4 py-2 bg-gray-200 dark:bg-gray-800 rounded shadow hover:bg-gray-300 dark:hover:bg-gray-700 transition", on_click={|| {
                crate::store::navigate("/");
            }}) {
                "Go Back"
            }
        }
    }
}
