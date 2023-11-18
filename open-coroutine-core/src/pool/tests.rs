use super::*;
use crate::common::Named;

#[test]
fn test_simple() {
    #[derive(Debug)]
    struct SleepBlocker {}

    impl Named for SleepBlocker {
        fn get_name(&self) -> &str {
            "SleepBlocker"
        }
    }
    impl Blocker for SleepBlocker {
        fn block(&self, time: Duration) {
            std::thread::sleep(time)
        }
    }

    let pool = Box::leak(Box::new(CoroutinePoolImpl::new(
        0,
        0,
        0,
        2,
        0,
        SleepBlocker {},
    )));
    _ = pool.submit(
        |_, _| {
            println!("1");
            None
        },
        None,
    );
    _ = pool.submit(
        |_, _| {
            println!("2");
            None
        },
        None,
    );
    _ = pool.try_timed_schedule(Duration::from_secs(1));
}