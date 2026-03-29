#[test]
fn implementation_plan_tracks_contract_test_coverage() {
    let plan = include_str!("../docs/implementation-plan.md");
    assert!(
        plan.contains("Contract tests cover the public pages use-cases that are already shipped."),
        "implementation plan must include contract test checklist item"
    );
}
