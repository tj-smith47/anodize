//! Sibling-publisher isolation test (closes [B10]).
//!
//! Asserts that a failing publisher in the middle of a group does not
//! abort siblings before or after it in the same group. Ships
//! regardless of whether B10 needed a code fix — guards against
//! future regression.

use anodizer_core::context::Context;
use anodizer_core::{PublishEvidence, Publisher, PublisherGroup, PublisherOutcome};
use anodizer_stage_publish::{DispatchOptions, dispatch};

// Re-implement a minimal FakePublisher here rather than importing the
// crate-internal `testing` mod (it's pub(crate)). Same shape.
struct FakePublisher {
    name: &'static str,
    group: PublisherGroup,
    required: bool,
    succeed: bool,
}

impl Publisher for FakePublisher {
    fn name(&self) -> &str {
        self.name
    }
    fn group(&self) -> PublisherGroup {
        self.group
    }
    fn required(&self) -> bool {
        self.required
    }
    fn run(&self, _ctx: &mut Context) -> anyhow::Result<PublishEvidence> {
        if self.succeed {
            Ok(PublishEvidence::new(self.name))
        } else {
            anyhow::bail!("fake failure from '{}'", self.name)
        }
    }
}

#[test]
fn three_managers_middle_fails_siblings_still_run() {
    // A succeeds, B fails, C succeeds. All Manager. None required.
    let publishers: Vec<Box<dyn Publisher>> = vec![
        Box::new(FakePublisher {
            name: "a",
            group: PublisherGroup::Manager,
            required: false,
            succeed: true,
        }),
        Box::new(FakePublisher {
            name: "b",
            group: PublisherGroup::Manager,
            required: false,
            succeed: false,
        }),
        Box::new(FakePublisher {
            name: "c",
            group: PublisherGroup::Manager,
            required: false,
            succeed: true,
        }),
    ];

    let mut ctx = Context::test_fixture();
    let opts = DispatchOptions::default();
    let report = dispatch(&publishers, &mut ctx, &opts).expect("dispatch returns Ok");

    assert_eq!(report.results.len(), 3, "all three publishers ran");
    assert_eq!(report.results[0].name, "a");
    assert_eq!(report.results[1].name, "b");
    assert_eq!(report.results[2].name, "c");

    assert!(
        matches!(report.results[0].outcome, PublisherOutcome::Succeeded),
        "A: succeeded"
    );
    assert!(
        matches!(report.results[1].outcome, PublisherOutcome::Failed(_)),
        "B: failed"
    );
    assert!(
        matches!(report.results[2].outcome, PublisherOutcome::Succeeded),
        "C: succeeded"
    );

    assert!(
        !report.submitter_gated,
        "no Submitter configured, gate did not fire"
    );
    assert_eq!(
        report.required_failures(),
        0,
        "B is required=false; no required failures"
    );
}

#[test]
fn three_managers_middle_required_failure_counts_in_required_failures() {
    // Same shape but B is required=true.
    let publishers: Vec<Box<dyn Publisher>> = vec![
        Box::new(FakePublisher {
            name: "a",
            group: PublisherGroup::Manager,
            required: false,
            succeed: true,
        }),
        Box::new(FakePublisher {
            name: "b",
            group: PublisherGroup::Manager,
            required: true,
            succeed: false,
        }),
        Box::new(FakePublisher {
            name: "c",
            group: PublisherGroup::Manager,
            required: false,
            succeed: true,
        }),
    ];

    let mut ctx = Context::test_fixture();
    let opts = DispatchOptions::default();
    let report = dispatch(&publishers, &mut ctx, &opts).expect("dispatch returns Ok");

    assert_eq!(report.results.len(), 3);
    assert!(matches!(
        report.results[0].outcome,
        PublisherOutcome::Succeeded
    ));
    assert!(matches!(
        report.results[1].outcome,
        PublisherOutcome::Failed(_)
    ));
    assert!(matches!(
        report.results[2].outcome,
        PublisherOutcome::Succeeded
    ));
    assert_eq!(
        report.required_failures(),
        1,
        "B is required=true and failed"
    );
}
