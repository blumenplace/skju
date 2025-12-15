use crate::domain::sensor::{Sensor, SensorCreateRequest, SensorError, SensorUpdateRequest};
use crate::ports::sensors_repository::SensorRepository;
use async_trait::async_trait;
use sqlx::{PgPool, query, query_as};

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
    async fn create(&self, request: SensorCreateRequest) -> Result<Sensor, SensorError> {
        let sensor = query_as!(
            Sensor,
            r#"INSERT INTO sensors (name, description, x, y) VALUES ($1, $2, $3, $4) RETURNING *"#,
            request.name,
            request.description,
            request.x,
            request.y
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(sensor)
    }

    async fn update(&self, id: i32, request: SensorUpdateRequest) -> Result<Sensor, SensorError> {
        self.check_if_exists(id).await?;

        let sensor = query_as!(
            Sensor,
            r#"UPDATE sensors SET name = $1, description = $2, x = $3, y = $4 WHERE id = $5 RETURNING *"#,
            request.name,
            request.description,
            request.x,
            request.y,
            id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(sensor)
    }

    async fn delete(&self, id: i32) -> Result<(), SensorError> {
        self.check_if_exists(id).await?;

        query!(r#"DELETE FROM sensors WHERE id = $1"#, id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn list(&self) -> Result<Vec<Sensor>, SensorError> {
        let sensors = query_as!(Sensor, r#"SELECT * From sensors"#)
            .fetch_all(&self.pool)
            .await?;

        Ok(sensors)
    }

    async fn get_by_id(&self, id: i32) -> Result<Option<Sensor>, SensorError> {
        let sensor = query_as!(Sensor, r#"SELECT * FROM sensors WHERE id = $1"#, id)
            .fetch_optional(&self.pool)
            .await?;

        if sensor.is_none() {
            return Err(SensorError::NotFound);
        }

        Ok(sensor)
    }

    async fn delete_all(&self) -> Result<(), SensorError> {
        query!(r#"DELETE FROM sensors"#).execute(&self.pool).await?;
        Ok(())
    }

    async fn check_if_exists(&self, id: i32) -> Result<(), SensorError> {
        let sensor = query!(r#"SELECT * FROM sensors WHERE id = $1"#, id)
            .fetch_optional(&self.pool)
            .await?;

        if sensor.is_none() {
            return Err(SensorError::NotFound);
        }

        Ok(())
    }
}
