use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
        mpsc::{SyncSender, sync_channel},
    },
    thread,
    time::{Duration, Instant, SystemTime},
};
use terminal_ui_contract::keymap::MonotonicClock;
use terminal_ui_runtime::{refresh::*, snapshot::*};
struct Clock(AtomicU64);
impl MonotonicClock for Clock {
    fn now(&self) -> Duration {
        Duration::from_millis(self.0.load(Ordering::Acquire))
    }
}
struct Timed {
    delay: Duration,
    started: SyncSender<()>,
}
impl ReadOnlyCollector<(), u64> for Timed {
    fn collect(
        &mut self,
        _: &RefreshRequest<()>,
        control: &CollectionControl,
    ) -> Result<Collected<u64>, CollectError> {
        let end = control.now() + self.delay;
        self.started.send(()).unwrap();
        while control.now() < end {
            control.check()?;
            thread::sleep(Duration::from_millis(1));
        }
        control.check()?;
        Ok(Collected {
            value: 1,
            empty: false,
            metadata: SnapshotMetadata {
                source: "fake clock".into(),
                scope: "synthetic only".into(),
                sampled_at: SystemTime::UNIX_EPOCH,
                freshness: Freshness::Unknown,
            },
        })
    }
}
#[test]
fn fake_two_ten_second_and_hung_deadline_are_deterministic() {
    for seconds in [2, 10, 60] {
        let clock = Arc::new(Clock(AtomicU64::new(0)));
        let (tx, rx) = sync_channel(1);
        let mut runtime = RefreshController::spawn_with_clock(
            Timed {
                delay: Duration::from_secs(seconds),
                started: tx,
            },
            RefreshPolicy {
                task_timeout: Duration::from_secs(20),
                poll_interval: Duration::from_secs(1),
                stale_after: Duration::from_secs(10),
            },
            clock.clone(),
        )
        .unwrap();
        runtime.request((), RequestKind::Manual).unwrap();
        rx.recv_timeout(Duration::from_secs(2)).unwrap();
        let finish = seconds.min(20) * 1000;
        clock.0.store(finish - 1, Ordering::Release);
        assert_eq!(runtime.poll(), RefreshPoll::Pending);
        clock.0.store(finish, Ordering::Release);
        let wall = Instant::now();
        let outcome = loop {
            let outcome = runtime.poll();
            if outcome != RefreshPoll::Pending {
                break outcome;
            }
            assert!(wall.elapsed() < Duration::from_secs(2));
            thread::yield_now();
        };
        if seconds == 60 {
            assert_eq!(outcome, RefreshPoll::Failed(CollectError::Timeout));
        } else {
            assert_eq!(
                outcome,
                RefreshPoll::Accepted {
                    generation: 1,
                    revision: 1,
                    changed: true
                }
            );
        }
        runtime.shutdown(Duration::from_secs(1)).unwrap();
    }
}
