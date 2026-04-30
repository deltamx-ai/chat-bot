use api_contracts::HealthResponse;

pub fn health() -> HealthResponse {
    HealthResponse {
        status: "ok".into(),
        service: "application".into(),
    }
}
