use threadloom::threadloom;

fn main() {
    threadloom! {
        div(id="test") {
            span "Hello"
        }
    }
}
