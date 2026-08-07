use axum::{routing::get, Router};

#[tokio::main]
async fn main() {
    let app: Router = Router::new().route("/health", get(|| async { "OK" }));
    let _ = app;

    println!("Budget Meal Planner Server v5 stub initialized.");
}
