use std::os::raw::c_void;

pub use base_coroutine::*;

#[allow(dead_code)]
extern "C" {
    fn init_hook();

    fn coroutine_crate(
        f: UserFunc<Option<&'static mut c_void>, (), Option<&'static mut c_void>>,
        param: Option<&'static mut c_void>,
        stack_size: usize,
    ) -> libc::c_int;

    fn try_timed_schedule(ns_time: u64) -> libc::c_int;

    fn timed_schedule(ns_time: u64) -> libc::c_int;
}

pub fn init() {
    unsafe { init_hook() }
}

pub fn co(
    f: UserFunc<Option<&'static mut c_void>, (), Option<&'static mut c_void>>,
    param: Option<&'static mut c_void>,
    stack_size: usize,
) -> bool {
    unsafe { coroutine_crate(f, param, stack_size) == 0 }
}

pub fn schedule() -> bool {
    unsafe { try_timed_schedule(u64::MAX) == 0 }
}

#[cfg(test)]
mod tests {
    use crate::{co, init, schedule, Yielder};
    use std::os::raw::c_void;
    use std::time::Duration;

    #[test]
    fn test_link() {
        init();
    }

    extern "C" fn f1(
        _yielder: &Yielder<Option<&'static mut c_void>, (), Option<&'static mut c_void>>,
        _input: Option<&'static mut c_void>,
    ) -> Option<&'static mut c_void> {
        println!("[coroutine1] launched");
        None
    }

    extern "C" fn f2(
        _yielder: &Yielder<Option<&'static mut c_void>, (), Option<&'static mut c_void>>,
        _input: Option<&'static mut c_void>,
    ) -> Option<&'static mut c_void> {
        println!("[coroutine2] launched");
        None
    }

    #[test]
    fn simplest() {
        assert!(co(f1, None, 4096));
        assert!(co(f2, None, 4096));
        assert!(schedule());
    }

    fn hook_test(secs: u64) {
        assert!(co(f1, None, 4096));
        assert!(co(f2, None, 4096));
        std::thread::sleep(Duration::from_millis(secs))
    }

    #[test]
    fn hook_test_schedule_timeout() {
        hook_test(1)
    }

    #[test]
    fn hook_test_schedule_normal() {
        hook_test(1_000)
    }
}
