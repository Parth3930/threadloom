use std::time::Instant;
use std::cell::RefCell;
use std::sync::{Arc, RwLock};

use std::hint::black_box;

thread_local! {
    static TL_GRAPH: RefCell<usize> = RefCell::new(0);
}

fn bench_thread_local(iterations: usize) {
    let start = Instant::now();
    let mut sum = 0;
    for _i in 0..iterations {
        TL_GRAPH.with(|g| {
            let mut val = g.borrow_mut();
            *val += black_box(1);
        });
        sum += black_box(TL_GRAPH.with(|g| *g.borrow()));
    }
    black_box(sum);
    println!("Thread Local ({} iters): {:?}", iterations, start.elapsed());
}

fn bench_rwlock(iterations: usize) {
    let graph = Arc::new(RwLock::new(0_usize));
    let start = Instant::now();
    let mut sum = 0;
    for _ in 0..iterations {
        {
            let mut val = graph.write().unwrap();
            *val += 1;
        }
        sum += {
            let val = graph.read().unwrap();
            *val
        };
    }
    black_box(sum);
    println!("RwLock ({} iters): {:?}", iterations, start.elapsed());
}

fn bench_mutex(iterations: usize) {
    let graph = Arc::new(std::sync::Mutex::new(0_usize));
    let start = Instant::now();
    let mut sum = 0;
    for _ in 0..iterations {
        {
            let mut val = graph.lock().unwrap();
            *val += 1;
        }
        sum += {
            let val = graph.lock().unwrap();
            *val
        };
    }
    black_box(sum);
    println!("Mutex ({} iters): {:?}", iterations, start.elapsed());
}

fn main() {
    let iterations = 10_000_000;
    bench_thread_local(iterations);
    bench_rwlock(iterations);
    bench_mutex(iterations);
}
