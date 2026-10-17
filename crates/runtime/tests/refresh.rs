use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
        mpsc::{Receiver, SyncSender, sync_channel},
    },
    thread,
    time::{Duration, Instant, SystemTime},
};
use terminal_ui_contract::keymap::MonotonicClock;
use terminal_ui_runtime::{refresh::*, snapshot::*};

fn policy() -> RefreshPolicy {
    RefreshPolicy {
        task_timeout: Duration::from_secs(20),
        poll_interval: Duration::from_secs(1),
        stale_after: Duration::from_secs(5),
    }
}
fn collected(value: u64) -> Collected<u64> {
    Collected {
        value,
        empty: value == 0,
        metadata: SnapshotMetadata {
            source: "synthetic".into(),
            scope: "fixture".into(),
            sampled_at: SystemTime::UNIX_EPOCH,
            freshness: Freshness::Unknown,
        },
    }
}
fn await_poll(controller: &mut RefreshController<u64, u64>) -> RefreshPoll {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let outcome = controller.poll();
        if outcome != RefreshPoll::Pending {
            return outcome;
        }
        assert!(Instant::now() < deadline, "worker did not complete");
        thread::yield_now();
    }
}
struct Gate {
    started: SyncSender<RefreshRequest<u64>>,
    release: Receiver<()>,
    cooperative: bool,
}
impl ReadOnlyCollector<u64, u64> for Gate {
    fn collect(
        &mut self,
        request: &RefreshRequest<u64>,
        control: &CollectionControl,
    ) -> Result<Collected<u64>, CollectError> {
        self.started.send(request.clone()).unwrap();
        loop {
            if self.cooperative {
                control.check()?;
            }
            match self.release.recv_timeout(Duration::from_millis(1)) {
                Ok(()) => break,
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                Err(_) => return Err(CollectError::Cancelled),
            }
        }
        if request.reason == u64::MAX {
            Err(CollectError::Failed("synthetic failure".into()))
        } else {
            Ok(collected(request.reason))
        }
    }
}
fn gated(
    cooperative: bool,
) -> (
    RefreshController<u64, u64>,
    Receiver<RefreshRequest<u64>>,
    SyncSender<()>,
) {
    let (started_tx, started) = sync_channel(1);
    let (release, release_rx) = sync_channel(1);
    let controller = RefreshController::spawn(
        Gate {
            started: started_tx,
            release: release_rx,
            cooperative,
        },
        policy(),
    )
    .unwrap();
    (controller, started, release)
}
#[test]
fn single_flight_one_pending_timer_has_no_starvation() {
    let (mut controller, started, release) = gated(true);
    controller.request(1, RequestKind::Manual).unwrap();
    assert_eq!(
        started
            .recv_timeout(Duration::from_secs(2))
            .unwrap()
            .generation,
        1
    );
    for reason in 2..=101 {
        controller.request(reason, RequestKind::Poll).unwrap();
    }
    assert!(controller.is_in_flight() && controller.has_pending());
    assert!(started.try_recv().is_err());
    release.send(()).unwrap();
    assert_eq!(
        await_poll(&mut controller),
        RefreshPoll::Accepted {
            generation: 1,
            revision: 1,
            changed: true
        }
    );
    assert_eq!(
        started.recv_timeout(Duration::from_secs(2)).unwrap().reason,
        101
    );
    release.send(()).unwrap();
    assert_eq!(
        await_poll(&mut controller),
        RefreshPoll::Accepted {
            generation: 101,
            revision: 2,
            changed: true
        }
    );
    assert_eq!(*controller.snapshot().unwrap().value(), 101);
    controller.shutdown(Duration::from_secs(1)).unwrap();
    assert!(!controller.worker_owned());
}
#[test]
fn manual_supersede_and_epoch_reject_late_response() {
    let (mut controller, started, release) = gated(true);
    controller.request(1, RequestKind::Manual).unwrap();
    started.recv_timeout(Duration::from_secs(2)).unwrap();
    controller.request(2, RequestKind::Manual).unwrap();
    controller.request(3, RequestKind::Poll).unwrap();
    release.send(()).unwrap();
    assert_eq!(await_poll(&mut controller), RefreshPoll::Stale);
    assert_eq!(
        started.recv_timeout(Duration::from_secs(2)).unwrap().reason,
        2
    ); // timer did not replace manual
    release.send(()).unwrap();
    await_poll(&mut controller);
    let retained = controller.snapshot().unwrap().clone();
    assert_eq!(*retained.value(), 2);
    controller.request(4, RequestKind::Manual).unwrap();
    started.recv_timeout(Duration::from_secs(2)).unwrap();
    assert_eq!(controller.advance_epoch().unwrap(), 1);
    controller.request(5, RequestKind::Manual).unwrap();
    release.send(()).unwrap();
    assert_eq!(await_poll(&mut controller), RefreshPoll::Stale);
    assert_eq!(controller.snapshot().unwrap().value(), &2);
    assert_eq!(
        started.recv_timeout(Duration::from_secs(2)).unwrap().epoch,
        1
    );
    release.send(()).unwrap();
    await_poll(&mut controller);
    assert_eq!(controller.snapshot().unwrap().epoch(), 1);
    assert_eq!(controller.snapshot().unwrap().value(), &5);
    assert_eq!(retained.value(), &2);
    controller.shutdown(Duration::from_secs(1)).unwrap();
}
#[test]
fn errors_empty_last_snapshot_and_no_change_100_ticks() {
    let (mut controller, started, release) = gated(true);
    for iteration in 0..100 {
        controller.request(7, RequestKind::Manual).unwrap();
        started.recv_timeout(Duration::from_secs(2)).unwrap();
        release.send(()).unwrap();
        assert_eq!(
            await_poll(&mut controller),
            RefreshPoll::Accepted {
                generation: iteration + 1,
                revision: 1,
                changed: iteration == 0
            }
        );
    }
    controller.request(u64::MAX, RequestKind::Manual).unwrap();
    started.recv_timeout(Duration::from_secs(2)).unwrap();
    release.send(()).unwrap();
    assert_eq!(
        await_poll(&mut controller),
        RefreshPoll::Failed(CollectError::Failed("synthetic failure".into()))
    );
    assert!(controller.state().is_stale());
    assert_eq!(controller.state().accepted().unwrap().value(), &7);
    assert_eq!(
        controller.state().error(),
        Some(&CollectError::Failed("synthetic failure".into()))
    );
    controller.request(0, RequestKind::Manual).unwrap();
    started.recv_timeout(Duration::from_secs(2)).unwrap();
    release.send(()).unwrap();
    await_poll(&mut controller);
    assert!(matches!(
        controller.state(),
        ResourceState::Empty { snapshot: Some(_) }
    ));
    assert_eq!(controller.snapshot().unwrap().value(), &0);
    controller.shutdown(Duration::from_secs(1)).unwrap();
}

struct Slow {
    delay: Duration,
    started: SyncSender<()>,
}
impl ReadOnlyCollector<u64, u64> for Slow {
    fn collect(
        &mut self,
        request: &RefreshRequest<u64>,
        control: &CollectionControl,
    ) -> Result<Collected<u64>, CollectError> {
        self.started.send(()).unwrap();
        control.wait_for(self.delay)?;
        Ok(collected(request.reason))
    }
}
#[test]
fn actual_two_and_ten_second_collectors_do_not_block_foreground() {
    for seconds in [2, 10] {
        let (tx, rx) = sync_channel(1);
        let mut controller = RefreshController::spawn(
            Slow {
                delay: Duration::from_secs(seconds),
                started: tx,
            },
            policy(),
        )
        .unwrap();
        let started = Instant::now();
        controller.request(1, RequestKind::Manual).unwrap();
        rx.recv_timeout(Duration::from_secs(1)).unwrap();
        let foreground = Instant::now();
        let mut interactions = 0;
        for _ in 0..100 {
            assert_eq!(controller.poll(), RefreshPoll::Pending);
            interactions += 1;
        }
        assert_eq!(interactions, 100);
        assert!(foreground.elapsed() < Duration::from_secs(1));
        loop {
            if matches!(controller.poll(), RefreshPoll::Accepted { .. }) {
                break;
            }
            assert!(started.elapsed() < Duration::from_secs(seconds + 3));
            thread::sleep(Duration::from_millis(5));
        }
        assert!(started.elapsed() >= Duration::from_secs(seconds));
        controller.shutdown(Duration::from_secs(1)).unwrap();
    }
}
#[test]
fn controlled_hang_times_out_and_busy_shutdown_joins() {
    let (tx, rx) = sync_channel(1);
    let mut short = policy();
    short.task_timeout = Duration::from_millis(40);
    let mut controller = RefreshController::spawn(
        Slow {
            delay: Duration::from_secs(60),
            started: tx,
        },
        short,
    )
    .unwrap();
    controller.request(1, RequestKind::Manual).unwrap();
    rx.recv_timeout(Duration::from_secs(1)).unwrap();
    assert_eq!(
        await_poll(&mut controller),
        RefreshPoll::Failed(CollectError::Timeout)
    );
    assert!(controller.snapshot().is_none());
    controller.shutdown(Duration::from_secs(1)).unwrap();
    let (mut controller, started, _release) = gated(true);
    controller.request(1, RequestKind::Manual).unwrap();
    started.recv_timeout(Duration::from_secs(1)).unwrap();
    controller.shutdown(Duration::from_secs(1)).unwrap();
    assert!(!controller.worker_owned());
    assert_eq!(
        controller.request(2, RequestKind::Manual),
        Err(CollectError::Cancelled)
    );
}
#[test]
fn inadmissible_uncooperative_worker_is_reported_and_handle_retained() {
    let (mut controller, started, release) = gated(false);
    controller.request(1, RequestKind::Manual).unwrap();
    started.recv_timeout(Duration::from_secs(1)).unwrap();
    let start = Instant::now();
    assert_eq!(
        controller.shutdown(Duration::from_millis(10)),
        Err(CollectError::ShutdownTimedOut)
    );
    assert!(start.elapsed() < Duration::from_secs(1));
    assert!(controller.worker_owned());
    release.send(()).unwrap();
    controller.shutdown(Duration::from_secs(1)).unwrap();
    assert!(!controller.worker_owned());
}

struct FakeClock(AtomicU64);
impl MonotonicClock for FakeClock {
    fn now(&self) -> Duration {
        Duration::from_millis(self.0.load(Ordering::Acquire))
    }
}
struct Immediate;
impl ReadOnlyCollector<u64, u64> for Immediate {
    fn collect(
        &mut self,
        request: &RefreshRequest<u64>,
        control: &CollectionControl,
    ) -> Result<Collected<u64>, CollectError> {
        control.check()?;
        Ok(collected(request.reason))
    }
}
#[test]
fn injected_clock_cadence_and_freshness_do_not_invent_provider_currentness() {
    let clock = Arc::new(FakeClock(AtomicU64::new(0)));
    let mut controller =
        RefreshController::spawn_with_clock(Immediate, policy(), clock.clone()).unwrap();
    assert!(controller.tick(1).unwrap().is_some());
    await_poll(&mut controller);
    assert_eq!(
        controller.snapshot().unwrap().metadata().freshness,
        Freshness::Unknown
    );
    assert_eq!(controller.tick(1).unwrap(), None);
    clock.0.store(5000, Ordering::Release);
    assert!(controller.tick(1).unwrap().is_some());
    await_poll(&mut controller);
    assert_eq!(controller.revision(), 1);
    controller.shutdown(Duration::from_secs(1)).unwrap();
}
struct Panics;
impl ReadOnlyCollector<u64, u64> for Panics {
    fn collect(
        &mut self,
        _: &RefreshRequest<u64>,
        _: &CollectionControl,
    ) -> Result<Collected<u64>, CollectError> {
        panic!("synthetic worker panic")
    }
}
#[test]
fn panicking_collector_returns_typed_error() {
    let mut controller = RefreshController::spawn(Panics, policy()).unwrap();
    controller.request(1, RequestKind::Manual).unwrap();
    assert_eq!(
        await_poll(&mut controller),
        RefreshPoll::Failed(CollectError::WorkerPanicked)
    );
    controller.shutdown(Duration::from_secs(1)).unwrap();
}
