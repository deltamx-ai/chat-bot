mod bootstrap;
mod routes;

fn main() {
    println!("{}", bootstrap::bootstrap_banner());
    println!("{}", routes::health::health_json());
}
