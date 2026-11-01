//! Offline-capable cross-device synchronization contracts.
//!
//! Version vectors provide causal ordering without pretending that unrelated
//! per-device counters are comparable. Concurrent versions are always retained
//! for explicit resolution; reconciliation has no silent-loss branch.

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

use crate::{DeviceId, OpaqueRef, SensitivityClass, SpaceId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionVectorViolation {
    Empty,
    ZeroCounter,
    DuplicateDevice,
    CounterOverflow,
}

impl Display for VersionVectorViolation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Empty => "version vectors must contain a causal device entry",
            Self::ZeroCounter => "version-vector counters must be positive",
            Self::DuplicateDevice => "version-vector device entries must be unique",
            Self::CounterOverflow => "version-vector counter overflow",
        })
    }
}
impl Error for VersionVectorViolation {}

/// Causal version of one durable object across registered devices.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionVector(BTreeMap<DeviceId, u64>);

impl VersionVector {
    /// Creates the first write made on a device.
    pub fn initial(device_ref: DeviceId) -> Self {
        Self(BTreeMap::from([(device_ref, 1)]))
    }

    /// Rehydrates a persisted vector while rejecting ambiguous/invalid data.
    pub fn from_entries(
        entries: impl IntoIterator<Item = (DeviceId, u64)>,
    ) -> Result<Self, VersionVectorViolation> {
        let mut values = BTreeMap::new();
        for (device, counter) in entries {
            if counter == 0 {
                return Err(VersionVectorViolation::ZeroCounter);
            }
            if values.insert(device, counter).is_some() {
                return Err(VersionVectorViolation::DuplicateDevice);
            }
        }
        if values.is_empty() {
            return Err(VersionVectorViolation::Empty);
        }
        Ok(Self(values))
    }

    pub fn entries(&self) -> impl Iterator<Item = (&DeviceId, u64)> {
        self.0.iter().map(|(device, counter)| (device, *counter))
    }

    /// Creates the next local write after all state represented by this vector.
    pub fn advanced_on(&self, device_ref: DeviceId) -> Result<Self, VersionVectorViolation> {
        let mut values = self.0.clone();
        let next = values
            .get(&device_ref)
            .copied()
            .unwrap_or(0)
            .checked_add(1)
            .ok_or(VersionVectorViolation::CounterOverflow)?;
        values.insert(device_ref, next);
        Ok(Self(values))
    }

    /// Element-wise causal merge used only after both payload versions have
    /// been retained and an explicit resolution/merge has been selected.
    pub fn merged_with(&self, other: &Self) -> Self {
        let mut values = self.0.clone();
        for (device, counter) in &other.0 {
            values
                .entry(device.clone())
                .and_modify(|current| *current = (*current).max(*counter))
                .or_insert(*counter);
        }
        Self(values)
    }

    fn causal_order(&self, other: &Self) -> Option<Ordering> {
        let self_greater = self
            .0
            .iter()
            .any(|(device, value)| *value > other.0.get(device).copied().unwrap_or(0));
        let other_greater = other
            .0
            .iter()
            .any(|(device, value)| *value > self.0.get(device).copied().unwrap_or(0));
        match (self_greater, other_greater) {
            (false, false) => Some(Ordering::Equal),
            (true, false) => Some(Ordering::Greater),
            (false, true) => Some(Ordering::Less),
            (true, true) => None,
        }
    }
}

/// Where durable state is authoritative.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DurabilityClass {
    /// Account service is canonical; clients keep durable local replicas.
    AccountBacked { space_ref: SpaceId },
    /// Explicit local-only/fallback truth owned by one device and Space.
    DeviceLocal {
        space_ref: SpaceId,
        device_ref: DeviceId,
    },
    /// Cache/projection that can be rebuilt without loss of user truth.
    Rebuildable,
}

/// Durable local synchronization status of one object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncState {
    PendingUpload,
    Synced,
    Conflicted,
}

/// Sync metadata carried beside every account-backed durable object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncMetadata {
    object_ref: OpaqueRef,
    space_ref: SpaceId,
    sensitivity: SensitivityClass,
    version: VersionVector,
    state: SyncState,
}

impl SyncMetadata {
    pub const fn new(
        object_ref: OpaqueRef,
        space_ref: SpaceId,
        sensitivity: SensitivityClass,
        version: VersionVector,
        state: SyncState,
    ) -> Self {
        Self {
            object_ref,
            space_ref,
            sensitivity,
            version,
            state,
        }
    }
    pub fn object_ref(&self) -> &OpaqueRef {
        &self.object_ref
    }
    pub fn space_ref(&self) -> &SpaceId {
        &self.space_ref
    }
    pub const fn sensitivity(&self) -> SensitivityClass {
        self.sensitivity
    }
    pub fn version(&self) -> &VersionVector {
        &self.version
    }
    pub const fn state(&self) -> SyncState {
        self.state
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncOrdering {
    LocalWins,
    RemoteWins,
}

/// Reconciliation never means deletion: superseded versions remain history.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictOutcome {
    Equivalent,
    Resolved(SyncOrdering),
    /// Both payload versions must remain durable until explicit user resolution.
    RequiresManualResolution,
}

/// Compares causal history without arbitrarily ordering concurrent devices.
pub fn reconcile(local: &VersionVector, remote: &VersionVector) -> ConflictOutcome {
    match local.causal_order(remote) {
        Some(Ordering::Equal) => ConflictOutcome::Equivalent,
        Some(Ordering::Greater) => ConflictOutcome::Resolved(SyncOrdering::LocalWins),
        Some(Ordering::Less) => ConflictOutcome::Resolved(SyncOrdering::RemoteWins),
        None => ConflictOutcome::RequiresManualResolution,
    }
}
