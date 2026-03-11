#[test]
fn server_scenarios_do_not_advertise_alternative_domain_implementations() {
    let scenarios = [
        include_str!("integration/content_flow_test.rs"),
        include_str!("integration/order_flow_test.rs"),
        include_str!("integration/event_flow_test.rs"),
    ];

    for scenario in scenarios {
        assert!(
            !scenario.contains("simplified version"),
            "scenario should use crate APIs instead of simplified replacements"
        );
        assert!(
            !scenario.contains("would use"),
            "scenario should not describe alternative non-library implementation"
        );
        assert!(
            scenario.contains("rustok_"),
            "scenario should import and use rustok library APIs"
        );
    }
}
