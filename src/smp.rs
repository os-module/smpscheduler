use crate::ScheduleHart;
use alloc::vec::Vec;
use core::marker::PhantomData;
use scheduler::BaseScheduler;

pub struct SmpScheduler<const SMP: usize, S: BaseScheduler, L: lock_api::RawMutex, H: ScheduleHart>
{
    local_queues: Vec<lock_api::Mutex<L, S>>,
    hart: PhantomData<H>,
}

impl<const SMP: usize, S: BaseScheduler, L: lock_api::RawMutex, H: ScheduleHart>
    SmpScheduler<SMP, S, L, H>
{
    /// Creates a new empty [`SmpScheduler`].
    pub fn new(mut schedulers: Vec<S>) -> Self {
        assert_eq!(schedulers.len(), SMP);
        let mut local_queues = Vec::new();
        for _ in 0..SMP {
            local_queues.push(lock_api::Mutex::new(schedulers.pop().unwrap()));
        }
        Self {
            local_queues,
            hart: PhantomData,
        }
    }
}

impl<const SMP: usize, S: BaseScheduler, L: lock_api::RawMutex, H: ScheduleHart>
    SmpScheduler<SMP, S, L, H>
{
    pub fn init(&self) {
        for i in 0..SMP {
            self.local_queues[i].lock().init();
        }
    }

    pub fn add_task(&self, task: S::SchedItem) {
        let hart_id = H::hart_id();
        self.local_queues[hart_id].lock().add_task(task);
    }

    pub fn remove_task(&self, task: &S::SchedItem) -> Option<S::SchedItem> {
        let hart_id = H::hart_id();
        self.local_queues[hart_id].lock().remove_task(task)
    }

    pub fn pick_next_task(&self) -> Option<S::SchedItem> {
        let hart_id = H::hart_id();
        let local = self.local_queues[hart_id].lock().pick_next_task();
        if local.is_some() {
            return local;
        }
        // steal task from other harts
        for i in 0..SMP {
            if i != hart_id {
                let lock = self.local_queues[i].try_lock();
                if lock.is_some() {
                    let mut other = lock.unwrap();
                    let task = other.pick_next_task();
                    if task.is_some() {
                        return task;
                    }
                }
            }
        }
        None
    }

    pub fn put_prev_task(&self, prev: S::SchedItem, preempt: bool) {
        let hart_id = H::hart_id();
        self.local_queues[hart_id]
            .lock()
            .put_prev_task(prev, preempt);
    }

    pub fn task_tick(&self, current: &S::SchedItem) -> bool {
        let hart_id = H::hart_id();
        self.local_queues[hart_id].lock().task_tick(current)
    }

    pub fn set_priority(&self, task: &S::SchedItem, prio: isize) -> bool {
        let hart_id = H::hart_id();
        self.local_queues[hart_id].lock().set_priority(task, prio)
    }
}
