//! Closed kernel-service identity and ownership references.

use crate::PackId;

/// Canonical kernel service identifiers.
///
/// This enum is intentionally closed and has no default or catch-all variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServiceId {
    /// Resolves the active system manifest.
    ManifestResolver,
    /// Assembles typed runtime context.
    ContextAssembler,
    /// Evaluates Protocols.
    ProtocolRuntime,
    /// Composes recommendations.
    RecommendationComposer,
    /// Resolves availability.
    AvailabilityService,
    /// Schedules due work.
    DueWorkScheduler,
    /// Arbitrates bounded attention surfaces.
    AttentionArbiter,
    /// Owns registered content.
    ContentRegistry,
    /// Probes external capabilities.
    CapabilityProbe,
    /// Owns versioned event topics.
    EventBus,
    /// Applies schema migrations.
    MigrationEngine,
    /// Exports and erases user data.
    ExportEraseService,
    /// Backs up and restores product state.
    BackupRestoreService,
    /// Enforces permissions.
    PermissionService,
}

/// Owner or publisher of a kernel contract.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OwningRef {
    /// A fixed kernel service owns the contract.
    KernelService(ServiceId),
    /// A registered Capability Pack owns the contract.
    CapabilityPack(PackId),
}
