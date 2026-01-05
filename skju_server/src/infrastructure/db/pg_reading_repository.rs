use crate::domain::reading::{DBReading, Reading, ReadingCreate, ReadingError, ReadingsRange};
use crate::ports::reading_repository::ReadingRepository;
use async_trait::async_trait;
use sqlx::query_builder::Separated;
use sqlx::{PgPool, Postgres, QueryBuilder};
use tracing::instrument;

pub struct PgReadingRepository {
    pool: PgPool,
}

impl PgReadingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl From<sqlx::Error> for ReadingError {
    fn from(err: sqlx::Error) -> Self {
        ReadingError::Database(err.to_string())
    }
}

#[async_trait]
impl ReadingRepository for PgReadingRepository {
    #[instrument(name = "repo.reading.create", skip(self))]
    async fn create(&self, request: Vec<ReadingCreate>) -> Result<(), ReadingError> {
        let mut query_builder = QueryBuilder::new(r#"INSERT INTO readings (sensor_id, value, timestamp)"#);
        let bind_values = |mut builder: Separated<Postgres, &str>, reading: ReadingCreate| {
            builder
                .push_bind(reading.sensor_id.value())
                .push_bind(reading.value.value())
                .push_bind(reading.timestamp.value());
        };

        query_builder
            .push_values(request, bind_values)
            .build()
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    #[instrument(name = "repo.reading.get_between", skip(self))]
    async fn get_between(&self, request: ReadingsRange) -> Result<Vec<Reading>, ReadingError> {
        let mut query_builder = QueryBuilder::new(r#"SELECT * FROM readings WHERE timestamp >= "#);

        query_builder.push_bind(request.from().value());
        query_builder.push(" AND timestamp <= ");
        query_builder.push_bind(request.to().value());

        if let Some(sensor_id) = request.sensor_id() {
            query_builder.push(" AND sensor_id = ");
            query_builder.push_bind(sensor_id.value());
        }

        query_builder.push(" ORDER BY timestamp ASC");

        let readings = query_builder
            .build_query_as::<DBReading>()
            .fetch_all(&self.pool)
            .await?;

        let readings = readings.into_iter().map(Into::into).collect();

        Ok(readings)
    }
}
