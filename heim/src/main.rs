use tonic::{transport::Server, Request, Response, Status};
use dotenvy::var;

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use crate::heim::heim::heim_server::HeimServer;
use crate::heim::heim_server::{HeimServer};
use crate::heim::HeimService;


// use std::sync::Arc;
// use crate::station_repository::StationRepository;
// use crate::station_service::StationService;
// use crate::heim::{Notification, CreateStationResponse, DeleteStationRequest, DeleteStationResponse, Empty, ListStationsRequest, ListStationsResponse, Station, StationsCountRequest, StationsCountResponse, UpdateStationResponse};
// use futures::future::BoxFuture;
// use futures::stream::BoxStream;


mod models;
mod heim;
mod services;
mod service;
mod repository;
mod cache;


const HEIM_DB_FILE: &str = "HEIM_DB_FILE";

const HEIM_DB_FILE_DEFAULT: &str = "/run/secrets/heim_db";

const HEIM_ADDRESS: &str = "HEIM_ADDRESS";

const HEIM_ADDRESS_DEFAULT: &str = "[::1]:50051";


async fn make_db_pool() -> Result<PgPool, Box<dyn std::error::Error>> {
    let db_connstr = var(HEIM_DB_FILE).unwrap_or(HEIM_DB_FILE_DEFAULT.to_string());
    let db = PgPoolOptions::new()
        .connect(&db_connstr).await?;
    Ok(db)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = make_db_pool().await?;

    let address = var(HEIM_ADDRESS).unwrap_or(HEIM_ADDRESS_DEFAULT.to_string()).parse()?;

    let station_repo = repository::Station::new(db);
    let station_service = service::Station::new(station_repo);
    let heim_service = HeimService::new(station_service);

    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter.set_serving::<HeimServer<HeimService>>().await;

    Server::builder()
        .add_service(health_service)
        .add_service(HeimServer::new(heim_service))
        .serve(address)
        .await?;

    Ok(())
}
