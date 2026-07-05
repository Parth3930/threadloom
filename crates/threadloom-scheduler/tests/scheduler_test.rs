use std::sync::Arc;
use threadloom_scheduler::{Scheduler, SchedulerMsg, GlobalSignal};
#[cfg(not(loom))]
use std::thread;
#[cfg(loom)]
use loom::thread;

#[test]
fn test_global_signal_loom() {
    #[cfg(loom)]
    loom::model(|| {
        run_global_test();
    });

    #[cfg(not(loom))]
    run_global_test();
}

fn run_global_test() {
    let scheduler = Scheduler::new();
    let tx = scheduler.global_sender();
    
    let global_sig = Arc::new(GlobalSignal::new(0, tx.clone()));
    
    let gs1 = global_sig.clone();
    let gs2 = global_sig.clone();

    let t1 = thread::spawn(move || {
        let val = gs1.get(threadloom_core::NodeId::test_new(1, 1)); // We need a way to mock NodeId
        gs1.set(val + 1);
    });

    let t2 = thread::spawn(move || {
        let val = gs2.get(threadloom_core::NodeId::test_new(2, 1));
        gs2.set(val + 1);
    });

    t1.join().unwrap();
    t2.join().unwrap();
}
