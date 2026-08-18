//! Update policy — Strategy pattern.
//!
//! The desktop shell's version/update strategy is deliberately pluggable so the
//! deepseek-desk-rsi engine can drive it. Strategies:
//!
//! - `Pin`: stay on the exact pinned dsh version (release binaries default here).
//! - `AutoBump`: trust the latest version and update in place.
//! - `AutoBumpWithGate`: update only after the RSI engine's verify pass reports ok.

/// The three selectable update strategies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateStrategy {
    Pin,
    AutoBump,
    AutoBumpWithGate,
}

/// Common interface for an update policy.
pub trait UpdatePolicy {
    fn strategy(&self) -> UpdateStrategy;
}

/// Stays on the pinned version.
pub struct PinnedPolicy;
impl UpdatePolicy for PinnedPolicy {
    fn strategy(&self) -> UpdateStrategy {
        UpdateStrategy::Pin
    }
}

/// Updates to the latest version unconditionally.
pub struct AutoBumpPolicy;
impl UpdatePolicy for AutoBumpPolicy {
    fn strategy(&self) -> UpdateStrategy {
        UpdateStrategy::AutoBump
    }
}

/// Updates only after the RSI engine verifies the new version.
pub struct GatedPolicy;
impl UpdatePolicy for GatedPolicy {
    fn strategy(&self) -> UpdateStrategy {
        UpdateStrategy::AutoBumpWithGate
    }
}

/// Decide whether an update may be applied under a policy and the RSI result.
pub fn may_update(policy: &dyn UpdatePolicy, rsi_verified: bool) -> bool {
    match policy.strategy() {
        UpdateStrategy::Pin => false,
        UpdateStrategy::AutoBump => true,
        UpdateStrategy::AutoBumpWithGate => rsi_verified,
    }
}
