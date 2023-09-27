use smpscheduler::{FifoSmpScheduler, FifoTask, ScheduleHart};
use std::hint::spin_loop;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

#[derive(Debug)]
struct ScheduleHartImpl;

impl ScheduleHart for ScheduleHartImpl {
    fn hart_id() -> usize {
        TMP.load(Ordering::SeqCst)
    }
}
static TMP: AtomicUsize = AtomicUsize::new(0);

const TASKS: usize = 4;

struct Task {
    task: fn(usize) -> usize,
    id: usize,
}
fn print_task(_thread: usize) -> usize {
    let mut count = 0;
    for i in 0..1000_000_000 {
        count += i;
        // count %= 1000;
    }
    count
}

impl Task {
    fn new(task: fn(usize) -> usize, id: usize) -> Self {
        Self { task, id }
    }
    fn run(&self, thread: usize) {
        let _res = (self.task)(thread);
        println!("Thread [{}] run over task {}", thread, self.id);
    }
}

fn main() {
    let fifo = FifoSmpScheduler::<TASKS, Task, spin::Mutex<()>, ScheduleHartImpl>::new();
    let fifo = Arc::new(fifo);
    fifo.init();
    // Make a vector to hold the children which are spawned.
    let mut children = vec![];
    static FLAG: AtomicUsize = AtomicUsize::new(0);
    static TASK_ID: AtomicUsize = AtomicUsize::new(0);
    for i in 0..TASKS {
        let fifo = fifo.clone();
        // Spin up another thread
        children.push(thread::spawn(move || {
            TMP.store(i, Ordering::SeqCst);
            for _ in 0..=i {
                let id = TASK_ID.fetch_add(1, Ordering::SeqCst);
                fifo.add_task(Arc::new(FifoTask::new(Task::new(print_task, id))));
            }
            FLAG.fetch_add(1, Ordering::SeqCst);
            while FLAG.load(Ordering::SeqCst) != TASKS {
                spin_loop()
            }
            loop {
                let task = fifo.pick_next_task();
                if task.is_some() {
                    let task = task.unwrap();
                    task.run(i);
                } else {
                    break;
                }
            }
        }));
    }
    for child in children {
        // Wait for the thread to finish. Returns a result.
        let _ = child.join();
    }
}
