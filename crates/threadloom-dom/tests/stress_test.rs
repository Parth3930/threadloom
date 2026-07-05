#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;
use web_sys::{window, Element, Node};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn stress_test_large_tree_render() {
    let window = window().expect("no global window");
    let document = window.document().expect("no document");
    let body = document.body().expect("no body");

    let container = document.create_element("div").unwrap();
    container.set_id("stress-container");
    body.append_child(&container).unwrap();

    // Stress test: insert 10,000 DOM nodes rapidly
    for i in 0..10_000 {
        let el = document.create_element("div").unwrap();
        el.set_inner_html(&format!("Node {}", i));
        container.append_child(&el).unwrap();
    }

    assert_eq!(container.child_nodes().length(), 10_000);
    
    // Cleanup
    container.remove();
}

#[wasm_bindgen_test]
fn stress_test_rapid_updates() {
    let window = window().expect("no global window");
    let document = window.document().expect("no document");
    let body = document.body().expect("no body");

    let el = document.create_element("div").unwrap();
    body.append_child(&el).unwrap();

    // Stress test: update the same node 10,000 times
    for i in 0..10_000 {
        el.set_inner_html(&format!("Update {}", i));
    }

    assert_eq!(el.inner_html(), "Update 9999");

    // Cleanup
    el.remove();
}
