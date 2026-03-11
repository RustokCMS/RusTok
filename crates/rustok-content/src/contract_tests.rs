#[test]
fn crate_api_defines_minimal_contract_sections() {
    let api = include_str!("../CRATE_API.md");
    for marker in [
        "## Минимальный набор контрактов",
        "### Входные DTO/команды",
        "### Доменные инварианты",
        "### События / outbox-побочные эффекты",
        "### Ошибки / коды отказов",
    ] {
        assert!(api.contains(marker), "CRATE_API.md must contain section: {marker}");
    }
}
