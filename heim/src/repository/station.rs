use sqlx::PgPool;
use crate::heim::Station;

#[derive(Debug, Clone)]
pub struct StationRepository {
    pool: PgPool,
}

impl StationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn count(&self) -> Result<i64, sqlx::Error> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM stations")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0)
    }

    pub async fn create(&self, station: &Station) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO stations (id, name, longitude, latitude, version) VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(station.id.as_ref().map(|id| id.value as i64))
        .bind(&station.name)
        .bind(station.location.as_ref().map(|l| l.longitude))
        .bind(station.location.as_ref().map(|l| l.latitude))
        .bind(station.version as i64)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn update(&self, station: &Station) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE stations SET name = $1, longitude = $2, latitude = $3, version = $4 WHERE id = $5"
        )
        .bind(&station.name)
        .bind(station.location.as_ref().map(|l| l.longitude))
        .bind(station.location.as_ref().map(|l| l.latitude))
        .bind(station.version as i64)
        .bind(station.id.as_ref().map(|id| id.value as i64))
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(&self, id: u64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM stations WHERE id = $1")
            .bind(id as i64)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn list(&self) -> Result<Vec<Station>, sqlx::Error> {
        let rows: Vec<(i64, String, f64, f64, i64)> = sqlx::query_as("SELECT id, name, longitude, latitude, version FROM stations")
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(|(id, name, longitude, latitude, version)| Station {
            id: Some(crate::heim::Id { value: id as u64 }),
            name,
            location: Some(crate::heim::Location {
                longitude,
                latitude,
            }),
            version: version as u64,
        }).collect())
    }
}
