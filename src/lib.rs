//! A simple smpscheduler for smp
//!
//! Every hart has a task queue, and the smpscheduler will fetch a task from the queue.
//! If the queue is empty, the smpscheduler will fetch a task from other harts.
//!
//! # Example
//! ```
//! use std::sync::Arc;
//! use std::sync::atomic::{AtomicUsize, Ordering};
//! use crate::smpscheduler::*;
//! static TMP: AtomicUsize = AtomicUsize::new(0);
//! #[derive(Debug)]
//! struct ScheduleHartImpl;
//!
//! impl ScheduleHart for ScheduleHartImpl {
//!     fn hart_id() -> usize {
//!         TMP.load(Ordering::SeqCst)
//!     }
//! }
//! let fifo = FifoSmpScheduler::<2, usize, spin::Mutex<()>, ScheduleHartImpl>::new();
//! fifo.init();
//! fifo.add_task(Arc::new(FifoTask::new(1)));
//! TMP.store(1, Ordering::SeqCst);
//! fifo.add_task(Arc::new(FifoTask::new(2)));
//! TMP.store(0, Ordering::SeqCst);
//! let task = fifo.pick_next_task();
//! assert!(task.is_some());
//! let task = task.unwrap();
//! let v = task.inner();
//! assert_eq!(*v, 1);
//! let task = fifo.pick_next_task(); // steal task from  hart 1
//! assert!(task.is_some());
//! let task = task.unwrap();
//! let v = task.inner();
//! assert_eq!(*v, 2);
//!```
//!

#![cfg_attr(not(test), no_std)]
#![deny(missing_docs)]

#[cfg(feature = "cfs")]
pub use cfs::*;
#[cfg(feature = "fifo")]
pub use fifo::*;
#[cfg(feature = "rr")]
pub use rr::*;
mod smp;

extern crate alloc;

/// The trait for getting hart id
pub trait ScheduleHart {
    /// get the hart id
    fn hart_id() -> usize;
}
#[cfg(feature = "fifo")]
mod fifo {
    /// fifo task
    pub type FifoTask<T> = scheduler::FifoTask<T>;

    use crate::smp::SmpScheduler;
    use crate::ScheduleHart;
    use alloc::vec::Vec;
    use core::ops::{Deref, DerefMut};
    use scheduler::FifoScheduler;
    /// A simple [FIFO] preemptive smpscheduler.
    pub struct FifoSmpScheduler<const SMP: usize, T, L: lock_api::RawMutex, H: ScheduleHart> {
        inner: SmpScheduler<SMP, FifoScheduler<T>, L, H>,
    }
    impl<const SMP: usize, T, L: lock_api::RawMutex, H: ScheduleHart> FifoSmpScheduler<SMP, T, L, H> {
        /// Creates a new empty [`FifoScheduler`].
        pub fn new() -> Self {
            let mut schedulers = Vec::new();
            for _ in 0..SMP {
                schedulers.push(FifoScheduler::new());
            }
            Self {
                inner: SmpScheduler::new(schedulers),
            }
        }
    }

    impl<const SMP: usize, T, L: lock_api::RawMutex, H: ScheduleHart> Deref
        for FifoSmpScheduler<SMP, T, L, H>
    {
        type Target = SmpScheduler<SMP, FifoScheduler<T>, L, H>;

        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    }

    impl<const SMP: usize, T, L: lock_api::RawMutex, H: ScheduleHart> DerefMut
        for FifoSmpScheduler<SMP, T, L, H>
    {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.inner
        }
    }
}

#[cfg(feature = "rr")]
mod rr {
    /// rr task
    pub type RRTask<T, const MAX_TIME_SLICE: usize> = scheduler::RRTask<T, MAX_TIME_SLICE>;

    use crate::smp::SmpScheduler;
    use crate::ScheduleHart;
    use alloc::vec::Vec;
    use scheduler::RRScheduler;

    /// A simple [Round-Robin] (RR) preemptive smpscheduler.
    pub struct RRSmpScheduler<
        const SMP: usize,
        const MAX_TIME_SLICE: usize,
        T,
        L: lock_api::RawMutex,
        H: ScheduleHart,
    > {
        inner: SmpScheduler<SMP, RRScheduler<T, MAX_TIME_SLICE>, L, H>,
    }

    impl<
            const SMP: usize,
            const MAX_TIME_SLICE: usize,
            T,
            L: lock_api::RawMutex,
            H: ScheduleHart,
        > RRSmpScheduler<SMP, MAX_TIME_SLICE, T, L, H>
    {
        /// Creates a new empty [`RRScheduler`].
        pub fn new() -> Self {
            let mut schedulers = Vec::new();
            for _ in 0..SMP {
                schedulers.push(RRScheduler::<T, MAX_TIME_SLICE>::new());
            }
            Self {
                inner: SmpScheduler::new(schedulers),
            }
        }
    }

    impl<
            const SMP: usize,
            const MAX_TIME_SLICE: usize,
            T,
            L: lock_api::RawMutex,
            H: ScheduleHart,
        > core::ops::Deref for RRSmpScheduler<SMP, MAX_TIME_SLICE, T, L, H>
    {
        type Target = SmpScheduler<SMP, RRScheduler<T, MAX_TIME_SLICE>, L, H>;

        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    }

    impl<
            const SMP: usize,
            const MAX_TIME_SLICE: usize,
            T,
            L: lock_api::RawMutex,
            H: ScheduleHart,
        > core::ops::DerefMut for RRSmpScheduler<SMP, MAX_TIME_SLICE, T, L, H>
    {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.inner
        }
    }
}
#[cfg(feature = "cfs")]
mod cfs {
    /// cfs task
    pub type CFSTask<T> = scheduler::CFSTask<T>;
    use crate::smp::SmpScheduler;
    use crate::ScheduleHart;
    use alloc::vec::Vec;
    use scheduler::CFScheduler;

    /// A simple [Completely Fair Scheduler][1] (CFS).
    pub struct CFSSmpScheduler<const SMP: usize, T, L: lock_api::RawMutex, H: ScheduleHart> {
        inner: SmpScheduler<SMP, CFScheduler<T>, L, H>,
    }

    impl<const SMP: usize, T, L: lock_api::RawMutex, H: ScheduleHart> CFSSmpScheduler<SMP, T, L, H> {
        /// Creates a new empty [`CFScheduler`].
        pub fn new() -> Self {
            let mut schedulers = Vec::new();
            for _ in 0..SMP {
                schedulers.push(CFScheduler::<T>::new());
            }
            Self {
                inner: SmpScheduler::new(schedulers),
            }
        }
    }

    impl<const SMP: usize, T, L: lock_api::RawMutex, H: ScheduleHart> core::ops::Deref
        for CFSSmpScheduler<SMP, T, L, H>
    {
        type Target = SmpScheduler<SMP, CFScheduler<T>, L, H>;

        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    }

    impl<const SMP: usize, T, L: lock_api::RawMutex, H: ScheduleHart> core::ops::DerefMut
        for CFSSmpScheduler<SMP, T, L, H>
    {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.inner
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        CFSSmpScheduler, CFSTask, FifoSmpScheduler, FifoTask, RRSmpScheduler, RRTask, ScheduleHart,
    };
    use alloc::sync::Arc;
    use core::sync::atomic::{AtomicUsize, Ordering};

    static TMP: AtomicUsize = AtomicUsize::new(0);

    macro_rules! gen_test {
        ($name:ident,$task:ident) => {
            $name.init();
            $name.add_task(Arc::new($task::new(1)));
            TMP.store(1, Ordering::SeqCst);
            $name.add_task(Arc::new($task::new(2)));
            TMP.store(0, Ordering::SeqCst);
            let task = $name.pick_next_task();
            assert!(task.is_some());
            let task = task.unwrap();
            let v = task.inner();
            assert_eq!(*v, 1);
            let task = $name.pick_next_task(); // steal task from  hart 1
            assert!(task.is_some());
            let task = task.unwrap();
            let v = task.inner();
            assert_eq!(*v, 2);
        };
    }
    #[derive(Debug)]
    struct ScheduleHartImpl;

    impl ScheduleHart for ScheduleHartImpl {
        fn hart_id() -> usize {
            TMP.load(Ordering::SeqCst)
        }
    }
    #[test]
    fn rr_test() {
        let rr = RRSmpScheduler::<2, 5, usize, spin::Mutex<()>, ScheduleHartImpl>::new();
        gen_test!(rr, RRTask);
    }

    #[test]
    fn fifo_test() {
        let fifo = FifoSmpScheduler::<2, usize, spin::Mutex<()>, ScheduleHartImpl>::new();
        gen_test!(fifo, FifoTask);
    }
    #[test]
    fn cfs_test() {
        let cfs = CFSSmpScheduler::<2, usize, spin::Mutex<()>, ScheduleHartImpl>::new();
        gen_test!(cfs, CFSTask);
    }
}
