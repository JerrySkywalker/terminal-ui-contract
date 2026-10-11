use std::{
    io::{BufRead, BufReader},
    process::{Child, Command, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
use terminal_ui_runtime::{lifecycle::OwnedChildLifecycle, refresh::CollectError};

struct FixtureChild {
    child: Child,
    reaped: bool,
}
impl OwnedChildLifecycle for FixtureChild {
    fn terminate_and_reap(&mut self, budget: Duration) -> Result<(), CollectError> {
        if self.reaped {
            return Ok(());
        }
        if self
            .child
            .try_wait()
            .map_err(|e| CollectError::Failed(e.to_string()))?
            .is_some()
        {
            self.reaped = true;
            return Ok(());
        }
        self.child
            .kill()
            .map_err(|e| CollectError::Failed(e.to_string()))?;
        let started = Instant::now();
        loop {
            if self
                .child
                .try_wait()
                .map_err(|e| CollectError::Failed(e.to_string()))?
                .is_some()
            {
                self.reaped = true;
                return Ok(());
            }
            if started.elapsed() >= budget {
                return Err(CollectError::Timeout);
            }
            thread::sleep(Duration::from_millis(1));
        }
    }
    fn is_reaped(&self) -> bool {
        self.reaped
    }
}
impl Drop for FixtureChild {
    fn drop(&mut self) {
        if !self.reaped {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

#[test]
#[ignore = "owned synthetic child only; launched explicitly by lifecycle test"]
fn child_fixture() {
    println!("FIXTURE_READY");
    thread::sleep(Duration::from_secs(60));
}

#[test]
fn only_owned_synthetic_child_is_terminated_and_reaped() {
    let child = Command::new(std::env::current_exe().unwrap())
        .args(["--ignored", "--exact", "child_fixture", "--nocapture"])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut owned = FixtureChild {
        child,
        reaped: false,
    };
    let stdout = owned.child.stdout.take().unwrap();
    let (tx, rx) = mpsc::sync_channel(1);
    let reader = thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            if line.unwrap_or_default().contains("FIXTURE_READY") {
                let _ = tx.send(());
                break;
            }
        }
    });
    let ready = rx.recv_timeout(Duration::from_secs(5));
    owned.terminate_and_reap(Duration::from_secs(2)).unwrap();
    reader.join().unwrap();
    ready.unwrap();
    assert!(owned.is_reaped());
    assert!(owned.child.try_wait().unwrap().is_some());
}
