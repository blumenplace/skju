use crate::domain::reading::{Reading, ReadingCreateRequest, ReadingError, ReadingGetBetweenRequest};
use crate::ports::reading_repository::ReadingRepository;
use async_trait::async_trait;
use sqlx::query_builder::Separated;
use sqlx::{PgPool, Postgres, QueryBuilder};

pub struct PgReadingRepository {
    pool: PgPool,
}

impl PgReadingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ReadingRepository for PgReadingRepository {
    async fn create(&self, request: Vec<ReadingCreateRequest>) -> Result<(), ReadingError> {
        let mut query_builder = QueryBuilder::new(r#"INSERT INTO readings (sensor_id, value, timestamp)"#);
        let bind_values = |mut builder: Separated<Postgres, &str>, reading: ReadingCreateRequest| {
            builder
                .push_bind(reading.sensor_id)
                .push_bind(reading.value)
                .push_bind(reading.timestamp);
        };

        query_builder
            .push_values(request, bind_values)
            .build()
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn get_between(&self, request: ReadingGetBetweenRequest) -> Result<Vec<Reading>, ReadingError> {
        let mut query_builder = QueryBuilder::new(r#"SELECT * FROM readings WHERE timestamp >= "#);

        query_builder.push_bind(request.from);

        if let Some(sensor_id) = request.sensor_id {
            query_builder.push(" AND sensor_id = ");
            query_builder.push_bind(sensor_id);
        }

        if let Some(to) = request.to {
            query_builder.push(" AND timestamp <= ");
            query_builder.push_bind(to);
        }

        query_builder.push(" ORDER BY timestamp ASC");

        let readings = query_builder
            .build_query_as::<Reading>()
            .fetch_all(&self.pool)
            .await?;

        Ok(readings)
    }
}
