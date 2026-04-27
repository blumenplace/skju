use std::sync::Arc;
use futures::future::BoxFuture;
use futures::stream::BoxStream;
use tonic::{Request, Response, Status};
use self::heim::heim_server::Heim;
use self::heim::{CreateStationResponse, DeleteStationRequest, DeleteStationResponse, Empty, ListStationsRequest, ListStationsResponse, Notification, Station, StationsCountRequest, StationsCountResponse, UpdateStationResponse};
use crate::station_service::StationService;

pub mod heim {
    tonic::include_proto!("Heim");
}

#[derive(Debug, Clone)]
pub struct HeimService {
    station_service: Arc<StationService>,
}

impl HeimService {
    pub(super) fn new(station_service: StationService) -> Self {
        Self { station_service: Arc::new(station_service) }
    }
}

impl Heim for HeimService {
    fn stations_count<'s, 'a>(&'s self, _request: Request<StationsCountRequest>) -> BoxFuture<'a, Result<Response<StationsCountResponse>, Status>>
    where
        's: 'a,
    {
        let service = self.station_service.clone();
        Box::pin(async move {
            let res = service.stations_count().await?;
            Ok(Response::new(res))
        })
    }

    fn create_station<'s, 'a>(&'s self, request: Request<Station>) -> BoxFuture<'a, Result<Response<CreateStationResponse>, Status>>
    where
        's: 'a,
    {
        let service = self.station_service.clone();
        Box::pin(async move {
            let res = service.create_station(request.into_inner()).await?;
            Ok(Response::new(res))
        })
    }

    fn update_station<'s, 'a>(&self, request: Request<Station>) -> BoxFuture<'a, Result<Response<UpdateStationResponse>, Status>>
    where
        's: 'a,
    {
        let service = self.station_service.clone();
        Box::pin(async move {
            let res = service.update_station(request.into_inner()).await?;
            Ok(Response::new(res))
        })
    }

    fn delete_station<'s, 'a>(&self, request: Request<DeleteStationRequest>) -> BoxFuture<'a, Result<Response<DeleteStationResponse>, Status>>
    where
        's: 'a,
    {
        let service = self.station_service.clone();
        Box::pin(async move {
            let res = service.delete_station(request.into_inner()).await?;
            Ok(Response::new(res))
        })
    }

    fn list_stations<'s, 'a>(&self, _request: Request<ListStationsRequest>) -> BoxFuture<'a, Result<Response<ListStationsResponse>, Status>>
    where
        's: 'a,
    {
        let service = self.station_service.clone();
        Box::pin(async move {
            let res = service.list_stations().await?;
            Ok(Response::new(res))
        })
    }

    type NotificationsStream = BoxStream<'static, Result<Notification, Status>>;

    fn notifications<'s, 'a>(&self, _request: Request<Empty>) -> BoxFuture<'a, Result<Response<Self::NotificationsStream>, Status>>
    where
        's: 'a,
    {
        todo!()
    }
}
