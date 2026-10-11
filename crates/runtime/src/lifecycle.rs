//! Product adapters own actual commands, process groups, stdout limits and policy.
use crate::refresh::CollectError;
use std::time::Duration;

/// Implementors retain an owned child handle, never a global PID discovery rule.
/// Error means ownership remains with the adapter; it cannot claim the child reaped.
/// This interface intentionally cannot spawn commands or touch product settings.
pub trait OwnedChildLifecycle {
    fn terminate_and_reap(&mut self, budget: Duration) -> Result<(), CollectError>;
    fn is_reaped(&self) -> bool;
}
