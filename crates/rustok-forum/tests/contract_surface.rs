#[test]
fn implementation_plan_tracks_contract_test_coverage() {
    let plan = include_str!("../docs/implementation-plan.md");
    assert!(
        plan.contains("Contract tests cover the current public use-cases"),
        "implementation plan must include contract test checklist item"
    );
}
