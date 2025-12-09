use crate::{
    error::{ApiError, IntoInternal},
    state::AppState,
};
use skju_core::SensorConfig;
use std::path::Path;
use tokio::fs::{OpenOptions, create_dir_all};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub async fn save_sensors(state: &AppState, sensors: Vec<SensorConfig>) -> Result<(), ApiError> {
    let data = sensors
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>()
        .join("\n");

    let file_path = format!("{}/sensors.txt", state.data_dir);
    let path = Path::new(&file_path);

    if let Some(parent) = path.parent() {
        create_dir_all(parent).await.into_internal()?;
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .await
        .into_internal()?;

    file.write_all(data.as_bytes()).await.into_internal()?;

    Ok(())
}

pub async fn get_sensors(state: &AppState) -> Result<Vec<SensorConfig>, ApiError> {
    let file_path = format!("{}/sensors.txt", state.data_dir);
    let path = Path::new(&file_path);

    if let Some(parent) = path.parent() {
        create_dir_all(parent).await.into_internal()?;
    }

    let mut file_data = String::new();
    let mut file = OpenOptions::new()
        .read(true)
        .create(true)
        .append(true)
        .open(file_path)
        .await
        .into_internal()?;

    file.read_to_string(&mut file_data).await.into_internal()?;

    if file_data.trim().is_empty() {
        return Ok(vec![]);
    }

    let sensors: Vec<SensorConfig> = file_data
        .lines()
        .map(|line| line.parse::<SensorConfig>().into_internal())
        .collect::<Result<Vec<SensorConfig>, _>>()?;

    Ok(sensors)
}
