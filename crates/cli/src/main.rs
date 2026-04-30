fn main() {
    let health = core::health();
    println!(
        "{}",
        serde_json::to_string_pretty(&health).expect("serialize health response")
    );
}
