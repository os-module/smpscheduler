# A SMP Scheduler

## Introduction
This Crate is a SMP Scheduler for Rust. It is based on the [Scheduler](https://github.com/rcore-os/arceos/tree/main/crates/scheduler) crate.

## Usage
```rust
static TMP: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
struct ScheduleHartImpl;

impl ScheduleHart for ScheduleHartImpl {
    fn hart_id() -> usize {
        TMP.load(Ordering::SeqCst)
    }
}

fn main() {
    let fifo = FifoSmpScheduler::<2, usize, spin::Mutex<()>, ScheduleHartImpl>::new();
    fifo.init();
    fifo.add_task(Arc::new(FifoTask::new(1)));
    TMP.store(1, Ordering::SeqCst);
    fifo.add_task(Arc::new(FifoTask::new(2)));
    TMP.store(0, Ordering::SeqCst);
    let task = fifo.pick_next_task();
    assert!(task.is_some());
    let task = task.unwrap();
    let v = task.inner();
    assert_eq!(*v, 1);
    let task = fifo.pick_next_task(); // steal task from  hart 1
    assert!(task.is_some());
    let task = task.unwrap();
    let v = task.inner();
    assert_eq!(*v, 2);
}
```