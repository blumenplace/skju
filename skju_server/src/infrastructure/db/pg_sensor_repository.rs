use crate::domain::sensor::{DBSensor, Sensor, SensorCreate, SensorError, SensorID, SensorUpdate};
use crate::ports::sensors_repository::SensorRepository;
use async_trait::async_trait;
use sqlx::{query, query_as, PgPool};
use tracing::instrument;

#[derive(Debug)]
pub struct PgSensorRepository {
    pool: PgPool,
}

impl PgSensorRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
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
        let sensor = query_as!(
            DBSensor,
            r#"INSERT INTO sensors (name, description, x, y) VALUES ($1, $2, $3, $4) RETURNING *"#,
            request.name.value(),
            request.description.value(),
            request.coordinates.x(),
            request.coordinates.y()
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(sensor.into())
    }

    #[instrument(name = "repo.sensor.update", skip(self))]
    async fn update(&self, id: SensorID, request: SensorUpdate) -> Result<Sensor, SensorError> {
        self.check_if_exists(id).await?;

        let sensor = query_as!(
            DBSensor,
            r#"UPDATE sensors SET name = $1, description = $2, x = $3, y = $4 WHERE id = $5 RETURNING *"#,
            request.name.value(),
            request.description.value(),
            request.coordinates.x(),
            request.coordinates.y(),
            id.value()
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(sensor.into())
    }

    #[instrument(name = "repo.sensor.delete", skip(self))]
    async fn delete(&self, id: SensorID) -> Result<(), SensorError> {
        self.check_if_exists(id).await?;

        query!(r#"DELETE FROM sensors WHERE id = $1"#, id.value())
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    #[instrument(name = "repo.sensor.list", skip(self))]
    async fn list(&self) -> Result<Vec<Sensor>, SensorError> {
        let sensors = query_as!(DBSensor, r#"SELECT * From sensors"#)
            .fetch_all(&self.pool)
            .await?;

        let sensors = sensors
            .into_iter()
            .map(|s| s.into())
            .collect::<Vec<Sensor>>();

        Ok(sensors)
    }

    #[instrument(name = "repo.sensor.get_by_id", skip(self))]
    async fn get_by_id(&self, id: SensorID) -> Result<Option<Sensor>, SensorError> {
        let sensor = query_as!(DBSensor, r#"SELECT * FROM sensors WHERE id = $1"#, id.value())
            .fetch_optional(&self.pool)
            .await?;

        let sensor = sensor.map(Into::into);

        if sensor.is_none() {
            return Err(SensorError::NotFound);
        }

        Ok(sensor)
    }

    #[instrument(name = "repo.sensor.delete_all", skip(self))]
    async fn delete_all(&self) -> Result<(), SensorError> {
        query!(r#"DELETE FROM sensors"#).execute(&self.pool).await?;
        Ok(())
    }

    async fn check_if_exists(&self, id: SensorID) -> Result<(), SensorError> {
        let sensor = query!(r#"SELECT * FROM sensors WHERE id = $1"#, id.value())
            .fetch_optional(&self.pool)
            .await?;

        if sensor.is_none() {
            return Err(SensorError::NotFound);
        }

        Ok(())
    }
}
