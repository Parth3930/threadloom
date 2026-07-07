#![allow(warnings)]
#[cfg(not(loom))]
pub(crate) mod sync {
    pub use std::sync::{Arc, Mutex, RwLock, mpsc};
    pub use std::thread;
}

#[cfg(loom)]
pub(crate) mod sync {
    pub use loom::sync::{Arc, Mutex, RwLock};
    // For mpsc, loom has crossbeam-like channels or we can use loom's thread.
    // loom::sync::mpsc doesn't exist, loom uses loom::sync::atomic etc.
    // Actually, loom supports std::sync::mpsc or crossbeam. We will just use loom's primitives where possible.
    pub use loom::thread;
    pub use std::sync::mpsc;
}

use std::collections::{HashMap, HashSet};
use std::cell::RefCell;
use std::rc::Rc;
use threadloom_core::{Boundary, NodeId, View, take_pending_boundaries};
use crate::sync::{Arc, Mutex, thread, mpsc};

/// Compile-time proof that Boundary cannot be sent across threads.
/// 
/// ```compile_fail
/// use threadloom_core::Boundary;
/// fn assert_send<T: Send>() {}
/// assert_send::<Boundary>();
/// ```
pub struct CompileTimeProof;

pub struct Shard {
    pub runtime_id: usize,
    boundaries: RefCell<HashMap<NodeId, Boundary>>,
    pub sender: mpsc::Sender<SchedulerMsg>,
}

pub enum SchedulerMsg {
    /// Notify the scheduler that some boundaries are dirty
    Dirty(Vec<NodeId>),
    /// Global state changed, invalidate specific nodes
    GlobalUpdate(NodeId),
}

pub struct Scheduler {
    shards: Arc<Mutex<HashMap<usize, mpsc::Sender<Vec<NodeId>>>>>,
    global_receiver: mpsc::Receiver<SchedulerMsg>,
    global_sender: mpsc::Sender<SchedulerMsg>,
    thread_pool: Vec<thread::JoinHandle<()>>,
}

impl Scheduler {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            shards: Arc::new(Mutex::new(HashMap::new())),
            global_receiver: rx,
            global_sender: tx,
            thread_pool: Vec::new(),
        }
    }

    pub fn register_shard(&self, runtime_id: usize, tx: mpsc::Sender<Vec<NodeId>>) {
        self.shards.lock().unwrap().insert(runtime_id, tx);
    }

    pub fn tick(&self) {
        // Collect all dirty nodes and batch them by runtime_id
        let mut batched = HashMap::new();
        let mut processed = 0;
        
        // Non-blocking drain of messages (batching)
        while let Ok(msg) = self.global_receiver.try_recv() {
            match msg {
                SchedulerMsg::Dirty(nodes) => {
                    for node in nodes {
                        batched.entry(node.runtime_id()).or_insert_with(Vec::new).push(node);
                    }
                },
                SchedulerMsg::GlobalUpdate(_node) => {
                    // Handled below in cross-shard
                }
            }
            processed += 1;
        }

        if processed > 0 {
            let shards = self.shards.lock().unwrap();
            for (runtime_id, nodes) in batched {
                // Size threshold logic:
                // If a boundary's tree size is very small (e.g., < 100 nodes), we could theoretically process it
                // synchronously on a single main thread to avoid thread-pool overhead.
                // However, since boundaries belong to shards, we dispatch them to their owning shard's thread.
                let size_threshold_passed = nodes.len() > 10; // Simple threshold stub
                
                if let Some(tx) = shards.get(&runtime_id) {
                    // Send unique nodes only (batching/dedup)
                    let unique_nodes: HashSet<_> = nodes.into_iter().collect();
                    if size_threshold_passed {
                        // Parallelize: send to the shard thread for parallel evaluation
                        let _ = tx.send(unique_nodes.into_iter().collect());
                    } else {
                        // Small tree logic: process immediately (in a real app, this would be on a local main thread)
                        // For the skeleton, we still route it but mark it as high priority or local.
                        let _ = tx.send(unique_nodes.into_iter().collect());
                    }
                }
            }
        }
    }

    pub fn global_sender(&self) -> mpsc::Sender<SchedulerMsg> {
        self.global_sender.clone()
    }
}

// Global state / cross-shard problem solution:
// Instead of Rc<RefCell<T>>, we use Arc<RwLock<T>> for shared state.
// We maintain a global dependency graph (or just simple subscription list) 
// using NodeIds (which are Send and Copy).
pub struct GlobalSignal<T> {
    value: Arc<crate::sync::RwLock<T>>,
    subscribers: Arc<Mutex<HashSet<NodeId>>>,
    scheduler_tx: mpsc::Sender<SchedulerMsg>,
}

impl<T> Clone for GlobalSignal<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            subscribers: self.subscribers.clone(),
            scheduler_tx: self.scheduler_tx.clone(),
        }
    }
}

impl<T: Clone> GlobalSignal<T> {
    pub fn new(initial: T, scheduler_tx: mpsc::Sender<SchedulerMsg>) -> Self {
        Self {
            value: Arc::new(crate::sync::RwLock::new(initial)),
            subscribers: Arc::new(Mutex::new(HashSet::new())),
            scheduler_tx,
        }
    }

    pub fn get(&self, current_node: NodeId) -> T {
        self.subscribers.lock().unwrap().insert(current_node);
        self.value.read().unwrap().clone()
    }

    pub fn set(&self, new_value: T) {
        *self.value.write().unwrap() = new_value;
        let subs = self.subscribers.lock().unwrap().clone();
        
        // Notify scheduler that these nodes are dirty due to global state change
        let _ = self.scheduler_tx.send(SchedulerMsg::Dirty(subs.into_iter().collect()));
    }
}
