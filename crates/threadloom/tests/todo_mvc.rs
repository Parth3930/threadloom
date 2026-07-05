use threadloom::{threadloom, create_signal, View, WriteSignal};
use threadloom_core::Boundary;

#[derive(Clone, PartialEq, Debug)]
struct Todo {
    id: usize,
    text: String,
    completed: bool,
}

fn todo_app() -> (View, WriteSignal<Vec<Todo>>) {
    let (todos, set_todos) = create_signal(vec![
        Todo { id: 1, text: "Buy milk".into(), completed: false },
        Todo { id: 2, text: "Learn Rust".into(), completed: true },
    ]);
    
    let view = threadloom! {
        div(class="todoapp") {
            ul(class="todo-list") {
                {
                    move || {
                        let current_todos = todos.get();
                        let items = current_todos.into_iter().map(|todo| {
                            let class_name = if todo.completed { "completed" } else { "" };
                            threadloom! {
                                li(class=class_name) {
                                    label { { todo.text.clone() } }
                                }
                            }
                        }).collect::<Vec<_>>();
                        threadloom! { { items } }
                    }
                }
            }
        }
    };
    
    (view, set_todos)
}

fn find_boundary(view: &View) -> Option<Boundary> {
    match view {
        View::DynamicNode(boundary) => Some(boundary.clone()),
        View::Element { children, .. } => children.iter().find_map(find_boundary),
        View::Fragment(children) => children.iter().find_map(find_boundary),
        _ => None,
    }
}

fn count_items(view: &View) -> usize {
    match view {
        View::Element { tag, .. } if tag == "li" => 1,
        View::Element { children, .. } => children.iter().map(count_items).sum(),
        View::Fragment(children) => children.iter().map(count_items).sum(),
        _ => 0,
    }
}

#[test]
fn test_reactivity_end_to_end() {
    let (app_view, set_todos) = todo_app();
    
    let boundary = find_boundary(&app_view).expect("Expected dynamic boundary in view");
    println!("Boundary id is {:?}", boundary.id);
    
    // Initial mount: explicitly evaluate the boundary closure to register dependencies!
    let initial_inner_view = boundary.id.track(|| (boundary.compute.borrow_mut())());
    
    // Assert initial state: 2 todos
    assert_eq!(count_items(&initial_inner_view), 2);
    
    // Verify it is NOT dirty initially
    println!("Boundary before set: {:?}", boundary.id.is_dirty());
    assert!(!boundary.id.is_dirty());
    
    // Update state (e.g. user adds a todo)
    println!("Setting todos...");
    set_todos.set(vec![
        Todo { id: 1, text: "Buy milk".into(), completed: false },
        Todo { id: 2, text: "Learn Rust".into(), completed: true },
        Todo { id: 3, text: "Ship Threadloom".into(), completed: false },
    ]);
    
    println!("Boundary after set: {:?}", boundary.id.is_dirty());
    // Verify the boundary is now marked dirty due to the signal change!
    assert!(boundary.id.is_dirty());
    
    // Verify it is in the scheduler's pending queue!
    let pending = threadloom_core::take_pending_boundaries();
    assert!(pending.contains(&boundary.id));
    
    // Scheduler handles it by re-evaluating the dirty boundary
    let new_inner_view = boundary.id.track(|| (boundary.compute.borrow_mut())());
    
    // Assert new state reflects the reactive update: 3 todos!
    assert_eq!(count_items(&new_inner_view), 3);
}
