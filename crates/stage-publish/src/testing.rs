//! Shared in-crate test doubles for publisher dispatch tests.
//!
//! Gated as `#[cfg(test)] pub(crate) mod testing;` in `lib.rs` so it's
//! visible to every test module in the crate but never compiled into the
//! library. The dispatch tests originally defined `FakePublisher` /
//! `FakeOutcome` privately; promoting them here lets the per-publisher
//! migrations reuse the same double without re-rolling one each time.

use anodizer_core::context::Context;
use anodizer_core::{PublishEvidence, Publisher, PublisherGroup};

/// Drives [`FakePublisher::run`].
pub enum FakeOutcome {
    Succeed,
    Fail(String),
}

/// Drives [`FakePublisher::rollback`]. Independent from [`FakeOutcome`]
/// because publishing and rollback are exercised by separate tests:
/// rollback dispatch only walks publishers whose `run()` succeeded, so
/// the rollback-failure path needs `Succeed` for the publish side AND a
/// failing rollback to verify the per-step `RollbackFailed` outcome.
pub enum FakeRollback {
    Succeed,
    Fail(String),
}

/// Minimal [`Publisher`] implementation that records its identity and
/// returns a predetermined [`FakeOutcome`] from `run`.
pub struct FakePublisher {
    pub name: String,
    pub group: PublisherGroup,
    pub required: bool,
    pub outcome: FakeOutcome,
    pub rollback_outcome: FakeRollback,
    /// Mirrors [`Publisher::rollback_scope_needed`]. When `Some`, the
    /// rollback dispatcher checks for the corresponding env var before
    /// invoking `rollback()`.
    pub rollback_scope: Option<&'static str>,
}

impl Publisher for FakePublisher {
    fn name(&self) -> &str {
        &self.name
    }
    fn group(&self) -> PublisherGroup {
        self.group
    }
    fn required(&self) -> bool {
        self.required
    }
    fn run(&self, _ctx: &mut Context) -> anyhow::Result<PublishEvidence> {
        match &self.outcome {
            FakeOutcome::Succeed => Ok(PublishEvidence::new(self.name.clone())),
            FakeOutcome::Fail(msg) => anyhow::bail!("{}", msg),
        }
    }
    fn rollback(&self, _ctx: &mut Context, _evidence: &PublishEvidence) -> anyhow::Result<()> {
        match &self.rollback_outcome {
            FakeRollback::Succeed => Ok(()),
            FakeRollback::Fail(msg) => anyhow::bail!("{}", msg),
        }
    }
    fn rollback_scope_needed(&self) -> Option<&'static str> {
        self.rollback_scope
    }
}

/// Convenience constructor returning the boxed-trait-object shape the
/// dispatcher consumes. Defaults rollback to a no-op success and
/// declares no scope, matching the dispatcher's "rollback runs cleanly"
/// case used by every dispatch-side test.
pub fn fake(
    name: &str,
    group: PublisherGroup,
    required: bool,
    outcome: FakeOutcome,
) -> Box<dyn Publisher> {
    Box::new(FakePublisher {
        name: name.to_string(),
        group,
        required,
        outcome,
        rollback_outcome: FakeRollback::Succeed,
        rollback_scope: None,
    })
}

/// Like [`fake`] but lets the test drive both the publish outcome AND
/// the rollback outcome. Use for rollback-failure tests where the
/// publisher must `Succeed` (so rollback dispatch picks it up) but the
/// `rollback()` call itself returns `Err`.
pub fn fake_with_rollback(
    name: &str,
    group: PublisherGroup,
    required: bool,
    outcome: FakeOutcome,
    rollback_outcome: FakeRollback,
) -> Box<dyn Publisher> {
    Box::new(FakePublisher {
        name: name.to_string(),
        group,
        required,
        outcome,
        rollback_outcome,
        rollback_scope: None,
    })
}

/// Like [`fake`] but declares a non-`None` `rollback_scope_needed`. Use
/// for the `RollbackSkippedNoScope` path where the dispatcher should
/// skip the rollback because the env var is unset.
pub fn fake_with_scope(
    name: &str,
    group: PublisherGroup,
    required: bool,
    outcome: FakeOutcome,
    rollback_scope: &'static str,
) -> Box<dyn Publisher> {
    Box::new(FakePublisher {
        name: name.to_string(),
        group,
        required,
        outcome,
        rollback_outcome: FakeRollback::Succeed,
        rollback_scope: Some(rollback_scope),
    })
}
