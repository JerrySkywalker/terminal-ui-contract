//! Single-flight scheduler with bounded cooperative task control and honest shutdown.
//! Rust traits are capability contracts, not process/permission isolation. Adapters
//! must not capture writers and must honor control deadlines. An uncooperative
//! collector returns a shutdown error; ownership is retained until it actually exits.
use crate::snapshot::{Collected, ResourceState, Snapshot};
use std::{
    fmt,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, SyncSender, TryRecvError, sync_channel},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use terminal_ui_contract::keymap::{MonotonicClock, SystemClock};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CollectError {
    Timeout,
    Cancelled,
    Failed(String),
    WorkerPanicked,
    WorkerUnavailable,
    ShutdownTimedOut,
}
impl fmt::Display for CollectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for CollectError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RequestKind {
    Manual,
    Poll,
}
#[derive(Clone, Debug)]
pub struct RefreshRequest<R> {
    pub generation: u64,
    pub epoch: u64,
    pub reason: R,
    pub kind: RequestKind,
}

pub type SharedClock = Arc<dyn MonotonicClock + Send + Sync>;

#[derive(Clone)]
pub struct CollectionControl {
    clock: SharedClock,
    deadline: Duration,
    cancelled: Arc<AtomicBool>,
}
impl CollectionControl {
    pub fn check(&self) -> Result<(), CollectError> {
        if self.cancelled.load(Ordering::Acquire) {
            Err(CollectError::Cancelled)
        } else if self.clock.now() >= self.deadline {
            Err(CollectError::Timeout)
        } else {
            Ok(())
        }
    }
    pub fn deadline(&self) -> Duration {
        self.deadline
    }
    pub fn now(&self) -> Duration {
        self.clock.now()
    }
    /// Cooperative bounded sleep, also usable for a controlled-hang synthetic fixture.
    pub fn wait_for(&self, duration: Duration) -> Result<(), CollectError> {
        let end = self.clock.now().saturating_add(duration);
        loop {
            self.check()?;
            let now = self.clock.now();
            if now >= end {
                return Ok(());
            }
            thread::sleep(end.saturating_sub(now).min(Duration::from_millis(5)));
        }
    }
}

/// No mutation callback, terminal handle, command or product configuration exists
/// in the request. Implementors are audited read-only and cooperatively bounded.
pub trait ReadOnlyCollector<R, T>: Send + 'static {
    fn collect(
        &mut self,
        request: &RefreshRequest<R>,
        control: &CollectionControl,
    ) -> Result<Collected<T>, CollectError>;
}

struct Response<T> {
    generation: u64,
    epoch: u64,
    result: Result<Collected<T>, CollectError>,
}

#[derive(Clone, Copy, Debug)]
pub struct RefreshPolicy {
    pub task_timeout: Duration,
    pub poll_interval: Duration,
    pub stale_after: Duration,
}
impl RefreshPolicy {
    pub fn validate(self) -> Result<Self, CollectError> {
        if self.task_timeout.is_zero() || self.poll_interval.is_zero() || self.stale_after.is_zero()
        {
            return Err(CollectError::Failed(
                "refresh policy durations must be positive".into(),
            ));
        }
        Ok(self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RefreshPoll {
    Pending,
    Accepted {
        generation: u64,
        revision: u64,
        changed: bool,
    },
    Stale,
    Failed(CollectError),
    Stopped,
}

pub struct RefreshController<R, T> {
    sender: Option<SyncSender<RefreshRequest<R>>>,
    receiver: Receiver<Response<T>>,
    worker: Option<JoinHandle<()>>,
    cancelled: Arc<AtomicBool>,
    clock: SharedClock,
    policy: RefreshPolicy,
    generation: u64,
    required_generation: u64,
    epoch: u64,
    revision: u64,
    active: Option<u64>,
    pending: Option<RefreshRequest<R>>,
    state: ResourceState<T>,
    last: Option<Arc<Snapshot<T>>>,
    accepted_at: Option<Duration>,
    next_poll: Duration,
    closing: bool,
}

impl<R: Send + 'static, T: Eq + Send + Sync + 'static> RefreshController<R, T> {
    pub fn spawn(
        collector: impl ReadOnlyCollector<R, T>,
        policy: RefreshPolicy,
    ) -> Result<Self, CollectError> {
        Self::spawn_with_clock(collector, policy, Arc::new(SystemClock::default()))
    }
    pub fn spawn_with_clock(
        mut collector: impl ReadOnlyCollector<R, T>,
        policy: RefreshPolicy,
        clock: SharedClock,
    ) -> Result<Self, CollectError> {
        let policy = policy.validate()?;
        let (sender, requests) = sync_channel::<RefreshRequest<R>>(1);
        let (responses, receiver) = sync_channel(1);
        let cancelled = Arc::new(AtomicBool::new(false));
        let worker_cancelled = cancelled.clone();
        let worker_clock = clock.clone();
        let worker = thread::Builder::new()
            .name("tui-read-only-collector".into())
            .spawn(move || {
                while let Ok(request) = requests.recv() {
                    if worker_cancelled.load(Ordering::Acquire) {
                        break;
                    }
                    let control = CollectionControl {
                        clock: worker_clock.clone(),
                        deadline: worker_clock.now().saturating_add(policy.task_timeout),
                        cancelled: worker_cancelled.clone(),
                    };
                    let result =
                        catch_unwind(AssertUnwindSafe(|| collector.collect(&request, &control)))
                            .unwrap_or(Err(CollectError::WorkerPanicked));
                    let result = control.check().and(result);
                    // One result per dispatched request; response never blocks shutdown.
                    if responses
                        .try_send(Response {
                            generation: request.generation,
                            epoch: request.epoch,
                            result,
                        })
                        .is_err()
                    {
                        break;
                    }
                }
            })
            .map_err(|e| CollectError::Failed(e.to_string()))?;
        Ok(Self {
            sender: Some(sender),
            receiver,
            worker: Some(worker),
            cancelled,
            clock: clock.clone(),
            policy,
            generation: 0,
            required_generation: 0,
            epoch: 0,
            revision: 0,
            active: None,
            pending: None,
            state: ResourceState::default(),
            last: None,
            accepted_at: None,
            next_poll: clock.now(),
            closing: false,
        })
    }
    pub fn state(&self) -> &ResourceState<T> {
        &self.state
    }
    pub fn snapshot(&self) -> Option<&Arc<Snapshot<T>>> {
        self.last.as_ref()
    }
    pub fn epoch(&self) -> u64 {
        self.epoch
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn is_in_flight(&self) -> bool {
        self.active.is_some()
    }
    pub fn has_pending(&self) -> bool {
        self.pending.is_some()
    }
    pub fn worker_owned(&self) -> bool {
        self.worker.is_some()
    }

    pub fn request(&mut self, reason: R, kind: RequestKind) -> Result<u64, CollectError> {
        if self.closing {
            return Err(CollectError::Cancelled);
        }
        self.generation = self
            .generation
            .checked_add(1)
            .ok_or_else(|| CollectError::Failed("generation exhausted".into()))?;
        if kind == RequestKind::Manual {
            self.required_generation = self.generation;
        }
        let request = RefreshRequest {
            generation: self.generation,
            epoch: self.epoch,
            reason,
            kind,
        };
        if self.active.is_some() {
            // Timer traffic cannot replace a queued explicit refresh or starve the active result.
            if kind == RequestKind::Manual
                || self
                    .pending
                    .as_ref()
                    .is_none_or(|p| p.kind == RequestKind::Poll)
            {
                self.pending = Some(request);
            }
        } else {
            self.dispatch(request)?;
        }
        Ok(self.generation)
    }
    fn dispatch(&mut self, request: RefreshRequest<R>) -> Result<(), CollectError> {
        let generation = request.generation;
        self.sender
            .as_ref()
            .ok_or(CollectError::Cancelled)?
            .try_send(request)
            .map_err(|_| CollectError::WorkerUnavailable)?;
        self.active = Some(generation);
        self.state = ResourceState::Loading {
            last_accepted: self.last.clone(),
            stale: self.state.is_stale()
                || self
                    .last
                    .as_ref()
                    .is_some_and(|last| last.epoch() != self.epoch),
            last_error: self.state.error().cloned(),
        };
        Ok(())
    }
    /// Product calls after its own successful baseline-changing transaction.
    /// No product draft, CAS, writer or permission is present in this controller.
    pub fn advance_epoch(&mut self) -> Result<u64, CollectError> {
        self.epoch = self
            .epoch
            .checked_add(1)
            .ok_or_else(|| CollectError::Failed("epoch exhausted".into()))?;
        self.pending = None;
        if let Some(last) = &self.last {
            self.state = ResourceState::Stale {
                last_accepted: last.clone(),
            };
        }
        Ok(self.epoch)
    }
    pub fn tick(&mut self, reason: R) -> Result<Option<u64>, CollectError> {
        let now = self.clock.now();
        if self
            .accepted_at
            .is_some_and(|time| now.saturating_sub(time) >= self.policy.stale_after)
            && let Some(last) = &self.last
        {
            match &mut self.state {
                ResourceState::Loading { stale, .. } => *stale = true,
                ResourceState::Error { .. } => {}
                _ => {
                    self.state = ResourceState::Stale {
                        last_accepted: last.clone(),
                    }
                }
            }
        }
        if now < self.next_poll {
            return Ok(None);
        }
        self.next_poll = now.saturating_add(self.policy.poll_interval);
        self.request(reason, RequestKind::Poll).map(Some)
    }
    pub fn poll(&mut self) -> RefreshPoll {
        if self.closing {
            return RefreshPoll::Stopped;
        }
        let response = match self.receiver.try_recv() {
            Ok(response) => response,
            Err(TryRecvError::Empty) => return RefreshPoll::Pending,
            Err(TryRecvError::Disconnected) => {
                self.active = None;
                self.pending = None;
                self.state = ResourceState::Error {
                    last_accepted: self.last.clone(),
                    error: CollectError::WorkerUnavailable,
                };
                return RefreshPoll::Failed(CollectError::WorkerUnavailable);
            }
        };
        let matches_active = self.active == Some(response.generation);
        if matches_active {
            self.active = None;
        }
        let outcome = if !matches_active
            || response.epoch != self.epoch
            || response.generation < self.required_generation
        {
            RefreshPoll::Stale
        } else {
            match response.result {
                Ok(collected) => {
                    // Sampling time is metadata, not content change. Provenance/currentness are content.
                    let changed = self.last.as_ref().is_none_or(|last| {
                        last.value() != &collected.value
                            || last.epoch() != response.epoch
                            || last.is_empty() != collected.empty
                            || last.metadata().source != collected.metadata.source
                            || last.metadata().scope != collected.metadata.scope
                            || last.metadata().freshness != collected.metadata.freshness
                    });
                    if changed {
                        self.revision = self.revision.saturating_add(1);
                    }
                    let snapshot = Arc::new(Snapshot::new(
                        collected,
                        response.generation,
                        response.epoch,
                        self.revision,
                    ));
                    self.state = if snapshot.is_empty() {
                        ResourceState::Empty {
                            snapshot: Some(snapshot.clone()),
                        }
                    } else {
                        ResourceState::Ready(snapshot.clone())
                    };
                    self.last = Some(snapshot);
                    self.accepted_at = Some(self.clock.now());
                    RefreshPoll::Accepted {
                        generation: response.generation,
                        revision: self.revision,
                        changed,
                    }
                }
                Err(error) => {
                    self.state = ResourceState::Error {
                        last_accepted: self.last.clone(),
                        error: error.clone(),
                    };
                    RefreshPoll::Failed(error)
                }
            }
        };
        if let Some(pending) = self.pending.take()
            && let Err(error) = self.dispatch(pending)
        {
            self.state = ResourceState::Error {
                last_accepted: self.last.clone(),
                error: error.clone(),
            };
            return RefreshPoll::Failed(error);
        }
        outcome
    }
    /// Bounded caller wait. Timeout retains JoinHandle, never claims cancellation.
    /// Caller must retain/retry this controller until the admitted worker exits.
    pub fn shutdown(&mut self, budget: Duration) -> Result<(), CollectError> {
        self.closing = true;
        self.pending = None;
        self.cancelled.store(true, Ordering::Release);
        self.sender.take();
        let started = Instant::now();
        while self
            .worker
            .as_ref()
            .is_some_and(|worker| !worker.is_finished())
        {
            if started.elapsed() >= budget {
                return Err(CollectError::ShutdownTimedOut);
            }
            thread::sleep(Duration::from_millis(1).min(budget.saturating_sub(started.elapsed())));
        }
        if let Some(worker) = self.worker.take() {
            worker.join().map_err(|_| CollectError::WorkerPanicked)?;
        }
        self.active = None;
        Ok(())
    }
}

impl<R, T> Drop for RefreshController<R, T> {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Release);
        self.sender.take();
        // Last-resort ownership preservation, NOT a bounded-cancellation claim.
        // Explicit shutdown is required; an inadmissible uncooperative collector
        // may block here. We never detach its JoinHandle and claim it stopped.
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
