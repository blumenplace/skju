use crate::station_repository::StationRepository;
use crate::heim::{Station, StationsCountResponse, CreateStationResponse, UpdateStationResponse, DeleteStationResponse, ListStationsResponse, DeleteStationRequest};
use tonic::Status;

#[derive(Debug)]
pub struct Station {
    repo: StationRepository,
}

impl Station {
    pub fn new(repo: StationRepository) -> Self {
        Self { repo }
    }

    pub async fn stations_count(&self) -> Result<StationsCountResponse, Status> {
        let count = self.repo.count().await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(StationsCountResponse { count: count as u64 })
    }

    pub async fn create_station(&self, station: Station) -> Result<CreateStationResponse, Status> {
        self.repo.create(&station).await
            .map_err(|e| Status::internal(e.to_string()))?;
        
        Ok(CreateStationResponse {
            result: Some(crate::heim::create_station_response::Result::Ok(crate::heim::Empty {}))
        })
    }

    pub async fn update_station(&self, station: Station) -> Result<UpdateStationResponse, Status> {
        self.repo.update(&station).await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(UpdateStationResponse {
            result: Some(crate::heim::update_station_response::Result::Ok(crate::heim::Empty {}))
        })
    }

    pub async fn delete_station(&self, request: DeleteStationRequest) -> Result<DeleteStationResponse, Status> {
        let id = request.id.ok_or_else(|| Status::invalid_argument("Missing ID"))?.value;
        self.repo.delete(id).await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(DeleteStationResponse {
            result: Some(crate::heim::delete_station_response::Result::Ok(crate::heim::Empty {}))
        })
    }

    pub async fn list_stations(&self) -> Result<ListStationsResponse, Status> {
        let stations = self.repo.list().await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(ListStationsResponse { stations })
    }
}
