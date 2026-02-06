use crate::domain::sensor::{DBSensor, Sensor, SensorCreate, SensorError, SensorID, SensorUpdate};
use crate::ports::sensors_repository::SensorRepository;
use async_trait::async_trait;
use futures::{StreamExt, TryStreamExt};
use sqlx::{Executor, PgPool, Postgres};
use tracing::instrument;

#[derive(Debug)]
pub struct PgSensorRepository {
    pool: PgPool,
}

impl PgSensorRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn check_if_exists<'trans, 'conn: 'trans, E>(exec: E, id: SensorID) -> Result<(), SensorError>
    where
        E: 'trans + Executor<'conn, Database = Postgres>,
    {
        let exists: bool = sqlx::query_scalar(r#"SELECT EXISTS (SELECT 1 FROM sensors WHERE id = $1 LIMIT 1)"#)
            .bind(id)
            .fetch_one(exec)
            .await?;

        if exists { Ok(()) } else { Err(SensorError::NotFound(id)) }
    }
}

impl From<sqlx::Error> for SensorError {
    fn from(err: sqlx::Error) -> Self {
        SensorError::Database(err.to_string())
    }
}

#[async_trait]
impl SensorRepository for PgSensorRepository {
    #[instrument(name = "repo.sensor.create", skip(self))]
    async fn create(&self, request: SensorCreate) -> Result<Sensor, SensorError> {
        let sensor: DBSensor =
            sqlx::query_as(r#"INSERT INTO sensors (name, description, x, y) VALUES ($1, $2, $3, $4) RETURNING *"#)
                .bind(request.name)
                .bind(request.description)
                .bind(request.coordinates.x())
                .bind(request.coordinates.y())
                .fetch_one(&self.pool)
                .await?;

        Ok(sensor.into())
    }

    #[instrument(name = "repo.sensor.update", skip(self))]
    async fn update(&self, id: SensorID, request: SensorUpdate) -> Result<Sensor, SensorError> {
        let sensor: DBSensor = sqlx::query_as(
            r#"UPDATE sensors SET name = $1, description = $2, x = $3, y = $4 WHERE id = $5 RETURNING *"#,
        )
        .bind(request.name)
        .bind(request.description)
        .bind(request.coordinates.x())
        .bind(request.coordinates.y())
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(sensor.into())
    }

    #[instrument(name = "repo.sensor.delete", skip(self))]
    async fn delete(&self, id: SensorID) -> Result<(), SensorError> {
        let result = sqlx::query(r#"DELETE FROM sensors WHERE id = $1"#)
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() != 0 {
            Ok(())
        } else {
            Err(SensorError::NotFound(id))
        }
    }

    #[instrument(name = "repo.sensor.list", skip(self))]
    async fn list(&self) -> Result<Vec<Sensor>, SensorError> {
        sqlx::query_as(r#"SELECT * FROM sensors"#)
            .fetch(&self.pool)
            .map(|r: Result<DBSensor, _>| r.map(Sensor::from))
            .try_collect()
            .await
            .map_err(SensorError::from)
    }

    #[instrument(name = "repo.sensor.get_by_id", skip(self))]
    async fn get_by_id(&self, id: SensorID) -> Result<Sensor, SensorError> {
        let maybe_sensor: Option<DBSensor> = sqlx::query_as(r#"SELECT * FROM sensors WHERE id = $1"#)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        maybe_sensor
            .map(Into::into)
            .ok_or(SensorError::NotFound(id))
    }

    #[instrument(name = "repo.sensor.delete_all", skip(self))]
    async fn delete_all(&self) -> Result<(), SensorError> {
        sqlx::query(r#"DELETE FROM sensors"#)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
