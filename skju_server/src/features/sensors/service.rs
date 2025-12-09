use crate::{
    error::{ApiError, IntoInternal},
    state::AppState,
};
use skju_core::SensorConfig;
use std::path::Path;
use tokio::fs::{OpenOptions, create_dir_all};
use tokio::io::AsyncWriteExt;

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
