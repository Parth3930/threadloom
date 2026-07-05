use threadloom::threadloom;

fn main() {
    threadloom! {
        div(id "my-div") {
            "Hello"
        }
    }
}
