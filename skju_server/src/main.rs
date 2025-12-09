mod app;
mod error;
mod features;
mod routes;
mod state;

use crate::app::create_app;
use crate::state::AppState;

const PORT: &str = "3001";

#[tokio::main]
async fn main() {
    let app = create_app().with_state(AppState::new());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", PORT))
        .await
        .expect("Failed to bind TcpListener to port");

    println!("Server is running on port {PORT}...");

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
