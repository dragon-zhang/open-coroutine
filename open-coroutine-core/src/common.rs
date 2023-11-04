use crate::coroutine::suspender::SimpleDelaySuspender;
use std::fmt::Debug;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

#[allow(clippy::pedantic, missing_docs)]
pub fn page_size() -> usize {
    static PAGE_SIZE: AtomicUsize = AtomicUsize::new(0);
    let mut ret = PAGE_SIZE.load(Ordering::Relaxed);
    if ret == 0 {
        unsafe {
            cfg_if::cfg_if! {
                if #[cfg(windows)] {
                    let mut info = std::mem::zeroed();
                    windows_sys::Win32::System::SystemInformation::GetSystemInfo(&mut info);
                    ret = info.dwPageSize as usize
                } else {
                    ret = libc::sysconf(libc::_SC_PAGESIZE) as usize;
                }
            }
        }
        PAGE_SIZE.store(ret, Ordering::Relaxed);
    }
    ret
}

/// Give the object a name.
pub trait Named {
    /// Get the name of this object.
    fn get_name(&self) -> &str;
}

/// A trait implemented for which needs `current()`.
pub trait Current<'c> {
    /// Init the current.
    fn init_current(current: &Self)
    where
        Self: Sized;

    /// Get the current if has.
    fn current() -> Option<&'c Self>
    where
        Self: Sized;

    /// clean the current.
    fn clean_current()
    where
        Self: Sized;
}

/// A trait for blocking current thread.
pub trait Blocker: Debug + Named {
    /// Block current thread for a while.
    fn block(&self, dur: Duration);
}

#[allow(missing_docs)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct DelayBlocker {}

/// const `DELAY_BLOCKER_NAME`.
pub const DELAY_BLOCKER_NAME: &str = "DelayBlocker";

impl Named for DelayBlocker {
    fn get_name(&self) -> &str {
        DELAY_BLOCKER_NAME
    }
}

impl Blocker for DelayBlocker {
    fn block(&self, dur: Duration) {
        if let Some(suspender) = crate::scheduler::SchedulableSuspender::current() {
            suspender.delay(dur);
        }
    }
}

#[allow(missing_docs)]
#[derive(Debug, Default)]
pub struct CondvarBlocker(std::sync::Mutex<()>, std::sync::Condvar);

/// const `CONDVAR_BLOCKER_NAME`.
pub const CONDVAR_BLOCKER_NAME: &str = "CondvarBlocker";

impl Named for CondvarBlocker {
    fn get_name(&self) -> &str {
        CONDVAR_BLOCKER_NAME
    }
}

impl Blocker for CondvarBlocker {
    fn block(&self, dur: Duration) {
        _ = self.1.wait_timeout(self.0.lock().unwrap(), dur);
    }
}

/// Join abstraction.
pub trait JoinHandle<T> {
    /// create `JoinHandle` instance.
    fn new(t: *const T, name: &str) -> Self;

    /// get the task name.
    ///
    /// # Errors
    /// if the task name is invalid.
    fn get_name(&self) -> std::io::Result<&str>;

    /// join with `Duration`.
    ///
    /// # Errors
    /// see `timeout_at_join`.
    fn timeout_join(&self, dur: Duration) -> std::io::Result<Result<Option<usize>, &str>> {
        self.timeout_at_join(open_coroutine_timer::get_timeout_time(dur))
    }

    /// join.
    ///
    /// # Errors
    /// see `timeout_at_join`.
    fn join(&self) -> std::io::Result<Result<Option<usize>, &str>> {
        self.timeout_at_join(u64::MAX)
    }

    /// join with timeout.
    ///
    /// # Errors
    /// if join failed.
    fn timeout_at_join(&self, timeout_time: u64) -> std::io::Result<Result<Option<usize>, &str>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    #[test]
    fn condvar_blocker() {
        let blocker = CondvarBlocker::default();
        let time = open_coroutine_timer::now();
        blocker.block(Duration::from_secs(1));
        let cost = Duration::from_nanos(open_coroutine_timer::now().saturating_sub(time));
        if Ordering::Less == cost.cmp(&Duration::from_secs(1)) {
            crate::error!("condvar_blocker cost {cost:?}");
        }
    }
}
