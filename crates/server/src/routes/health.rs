pub fn health_json() -> String {
    let health = core::health();
    serde_json::to_string_pretty(&health).expect("serialize health response")
}
