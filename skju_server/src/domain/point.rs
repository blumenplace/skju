use byteorder::{BigEndian, ByteOrder, LittleEndian, ReadBytesExt};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::io::Cursor;
use std::io::Error as IoError;
use std::mem::size_of;

pub(crate) type Coordinate = f64;

pub(crate) struct Point {
    x: Coordinate,
    y: Coordinate,
}

impl Point {
    pub(crate) fn new(x: Coordinate, y: Coordinate) -> Self {
        Self { x, y }
    }
}

impl sqlx::Type<sqlx::Postgres> for Point {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("geometry")
    }

    fn compatible(ty: &sqlx::postgres::PgTypeInfo) -> bool {
        println!("\n\n\n\n\n!!!!!!!TYPE INFO NAME: {:?}\n\n\n\n", ty);
        sqlx::TypeInfo::name(ty) == "geometry"
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for Point {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let bytes: &'r [u8] = <&'r [u8] as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        decode(bytes).map_err(|e| e.into())
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for Point {
    fn encode_by_ref(
        &self,
        buf: &mut <sqlx::Postgres as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        encode(self).encode_by_ref(buf)
    }

    #[inline]
    fn size_hint(&self) -> usize {
        FULL_POINT_EWKB_SIZE
    }
}

#[derive(Debug)]
pub(crate) enum PointDecodeError {
    UnexpectedEod,
    WrongPointType(u32),
    Io(IoError),
}

impl Display for PointDecodeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PointDecodeError::UnexpectedEod => write!(f, "Unexpected end of data"),
            PointDecodeError::WrongPointType(typ) => write!(f, "Wrong point type: got {}", typ),
            PointDecodeError::Io(err) => Display::fmt(err, f),
        }
    }
}

impl Error for PointDecodeError {}

impl From<IoError> for PointDecodeError {
    fn from(err: IoError) -> Self {
        PointDecodeError::Io(err)
    }
}

fn decode(data: &[u8]) -> Result<Point, PointDecodeError> {
    if data.len() < 1 {
        return Err(PointDecodeError::UnexpectedEod);
    }

    if data[0] == 0 {
        decode_with_byte_order::<BigEndian>(&data[1..])
    } else {
        decode_with_byte_order::<LittleEndian>(&data[1..])
    }
}

#[repr(u32)]
enum EwkbFlags {
    HasZ = 0x8000_0000,
    HasM = 0x4000_0000,
    HasSRID = 0x2000_0000,
}

const POINT_TYPE: u32 = 1;

const POINT_EWKB_SIZE: usize = size_of::<u32>() /* geometry type */ + size_of::<Coordinate>() /* x coordinate */ + size_of::<Coordinate>() /* y coordinate */;

const FULL_POINT_EWKB_SIZE: usize = size_of::<u8>() /* byte order */ + POINT_EWKB_SIZE;

fn decode_with_byte_order<E: ByteOrder>(data: &[u8]) -> Result<Point, PointDecodeError> {
    if data.len() < POINT_EWKB_SIZE {
        return Err(PointDecodeError::UnexpectedEod);
    }

    let mut cursor = Cursor::new(data);
    let geometry_type = cursor.read_u32::<E>()?;

    let typ = geometry_type & 0x0000_FFFF;
    if typ != POINT_TYPE {
        return Err(PointDecodeError::WrongPointType(typ));
    }

    if geometry_type & (EwkbFlags::HasSRID as u32) != 0 {
        // ignore SRID for now
        let _ = cursor.read_u32::<E>()?;
    }

    let x = cursor.read_f64::<E>()?;
    let y = cursor.read_f64::<E>()?;

    // For not ignoring Option<f64> (HasZ) and Option<f64> (HasM) bytes

    Ok(Point::new(x, y))
}

fn encode(point: &Point) -> [u8; FULL_POINT_EWKB_SIZE] {
    let mut output = [0u8; FULL_POINT_EWKB_SIZE];
    // output[0] <- 0 BigEndian
    encode_with_byte_order::<BigEndian>(point, &mut output[1..]);
    output
}

fn encode_with_byte_order<E: ByteOrder>(point: &Point, output: &mut [u8]) {
    E::write_u32(&mut output[0..], POINT_TYPE);
    E::write_f64(&mut output[4..], point.x);
    E::write_f64(&mut output[12..], point.y);
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;
    use std::time::Duration;
    use testcontainers::core::{ContainerPort, WaitFor};
    use testcontainers::runners::AsyncRunner;
    use testcontainers::*;

    async fn connect_with_retry(url: &str) -> sqlx::PgPool {
        let mut last_err = None;

        for _ in 0..50 {
            match PgPoolOptions::new()
                .max_connections(2)
                .acquire_timeout(Duration::from_secs(2))
                .connect(url)
                .await
            {
                Ok(pool) => return pool,
                Err(e) => {
                    last_err = Some(e);
                    tokio::time::sleep(Duration::from_millis(250)).await;
                }
            }
        }

        panic!("DB never became ready: {:?}", last_err);
    }

    #[tokio::test]
    async fn test_decode() {
        let image = GenericImage::new("postgis/postgis", "18-3.6-alpine")
            .with_wait_for(WaitFor::message_on_stdout(
                "database system is ready to accept connections",
            ))
            .with_exposed_port(ContainerPort::Tcp(5432));

        let container = image
            .with_env_var("POSTGRES_USER", "postgres")
            .with_env_var("POSTGRES_PASSWORD", "postgres")
            .with_env_var("POSTGRES_DB", "testdb")
            .start()
            .await
            .expect("failed to start PostGIS testcontainer");

        let host_port = container
            .get_host_port_ipv4(ContainerPort::Tcp(5432))
            .await
            .expect("failed to get host port");
        let database_url = format!("postgres://postgres:postgres@127.0.0.1:{host_port}/testdb");

        let pool = connect_with_retry(&database_url).await;

        sqlx::query(r#"CREATE TABLE IF NOT EXISTS places (id BIGSERIAL PRIMARY KEY, point geometry(Point, 4326))"#)
            .execute(&pool)
            .await
            .expect("failed to create places table");

        sqlx::query(r#"INSERT INTO places (point) VALUES ($1)"#)
            .bind(Point::new(32.1, 45.6))
            .execute(&pool)
            .await
            .expect("failed to insert point");

        #[derive(sqlx::FromRow)]
        #[allow(unused)]
        struct Place {
            id: i64,
            point: Point,
        }

        let place: Place = sqlx::query_as(r#"SELECT * FROM places LIMIT 1"#)
            .fetch_one(&pool)
            .await
            .expect("failed to fetch point");

        assert_eq!(place.point.x, 32.1);
        assert_eq!(place.point.y, 45.6);
    }
}
