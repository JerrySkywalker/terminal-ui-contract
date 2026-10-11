//! Accepted immutable values and provenance; this layer never evaluates domain truth.
use std::{sync::Arc, time::SystemTime};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Freshness {
    CurrentObservation,
    Historical,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnapshotMetadata {
    pub source: String,
    pub scope: String,
    pub sampled_at: SystemTime,
    pub freshness: Freshness,
}

#[derive(Clone, Debug)]
pub struct Collected<T> {
    pub value: T,
    pub empty: bool,
    pub metadata: SnapshotMetadata,
}

/// Private storage exposes only shared references. Collectors must supply immutable
/// normalized values, not interior-mutable handles to external/product state.
#[derive(Debug)]
pub struct Snapshot<T> {
    value: T,
    metadata: SnapshotMetadata,
    generation: u64,
    epoch: u64,
    revision: u64,
    empty: bool,
}
impl<T> Snapshot<T> {
    pub(crate) fn new(collected: Collected<T>, generation: u64, epoch: u64, revision: u64) -> Self {
        Self {
            value: collected.value,
            metadata: collected.metadata,
            generation,
            epoch,
            revision,
            empty: collected.empty,
        }
    }
    pub fn value(&self) -> &T {
        &self.value
    }
    pub fn metadata(&self) -> &SnapshotMetadata {
        &self.metadata
    }
    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn epoch(&self) -> u64 {
        self.epoch
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn is_empty(&self) -> bool {
        self.empty
    }
}

#[derive(Debug)]
pub enum ResourceState<T> {
    Loading {
        last_accepted: Option<Arc<Snapshot<T>>>,
        stale: bool,
        last_error: Option<crate::refresh::CollectError>,
    },
    Empty {
        snapshot: Option<Arc<Snapshot<T>>>,
    },
    Ready(Arc<Snapshot<T>>),
    Error {
        last_accepted: Option<Arc<Snapshot<T>>>,
        error: crate::refresh::CollectError,
    },
    Stale {
        last_accepted: Arc<Snapshot<T>>,
    },
}
impl<T> Default for ResourceState<T> {
    fn default() -> Self {
        Self::Empty { snapshot: None }
    }
}
impl<T> ResourceState<T> {
    pub fn accepted(&self) -> Option<&Arc<Snapshot<T>>> {
        match self {
            Self::Ready(snapshot)
            | Self::Stale {
                last_accepted: snapshot,
            } => Some(snapshot),
            Self::Loading { last_accepted, .. } | Self::Error { last_accepted, .. } => {
                last_accepted.as_ref()
            }
            Self::Empty { snapshot } => snapshot.as_ref(),
        }
    }
    pub fn is_stale(&self) -> bool {
        matches!(
            self,
            Self::Loading { stale: true, .. }
                | Self::Stale { .. }
                | Self::Error {
                    last_accepted: Some(_),
                    ..
                }
        )
    }
    pub fn error(&self) -> Option<&crate::refresh::CollectError> {
        match self {
            Self::Error { error, .. } => Some(error),
            Self::Loading { last_error, .. } => last_error.as_ref(),
            _ => None,
        }
    }
}
