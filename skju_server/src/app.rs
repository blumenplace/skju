use axum::Router;
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::channel;
use tokio::time;

use crate::application::messages::{AppConsumer, AppMessage, ConsumerContext};
use crate::application::readings::ReadingsBuffer;
use crate::application::{readings, sensors};
use crate::infrastructure::app_bus::AppBus;
use crate::infrastructure::pg_reading_repository::PgReadingRepository;
use crate::infrastructure::pg_sensor_repository::PgSensorRepository;
use crate::ports::bus_service::BusMessage;
use crate::routes::create_routes;
use crate::state::{AppServices, AppState};

pub fn create_app(db_pool: Pool<Postgres>) -> Router<()> {
    let (tx, rx) = channel::<BusMessage<AppMessage>>(1000);

    let sensor_repository = PgSensorRepository::new(db_pool.clone());
    let readings_repository = PgReadingRepository::new(db_pool.clone());
    let sensor_repository = Arc::new(sensor_repository);
    let readings_repository = Arc::new(readings_repository);

    let bus_service = AppBus::new(tx);
    let bus_service = Arc::new(bus_service);

    let sensor_service = sensors::Service::new(sensor_repository.clone(), bus_service.clone());
    let readings_service = readings::Service::new(readings_repository.clone(), bus_service.clone());
    let sensor_service = Arc::new(sensor_service);
    let readings_service = Arc::new(readings_service);

    let app_services = AppServices {
        sensor_service: sensor_service.clone(),
        reading_service: readings_service.clone(),
    };

    let readings_buffer = ReadingsBuffer::new(readings_repository, 1000);
    let readings_buffer = Arc::new(readings_buffer);
    let flush_readings = {
        let buffer = readings_buffer.clone();
        move || flush_readings(buffer.clone())
    };

    let context = ConsumerContext { readings_buffer: readings_buffer.clone() };
    let bus_consumer = AppConsumer::new(rx, context);
    let app_state = AppState::new(app_services);
    let app = get_app_router().with_state(app_state);

    tokio::spawn(bus_consumer.run());
    tokio::spawn(run_interval(Duration::from_millis(100), flush_readings));

    app
}

pub fn get_app_router() -> Router<AppState> {
    Router::new().nest("/api", create_routes())
}

async fn run_interval<F, Fut>(interval: Duration, mut task: F)
where
    F: FnMut() -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send,
{
    let mut interval = time::interval(interval);

    loop {
        interval.tick().await;
        task().await;
    }
}

async fn flush_readings(buffer: Arc<ReadingsBuffer>) {
    if let Err(error) = buffer.flush().await {
        eprintln!("Failed to flush readings: {}", error);
    }
}
