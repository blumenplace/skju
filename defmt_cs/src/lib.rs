use defmt_decoder::Table;
use std::fs;
use std::sync::{Arc, Mutex};
use uniffi::deps::anyhow;

#[derive(Debug, thiserror::Error, uniffi::Object)]
#[error(transparent)]
#[uniffi::export(Debug, Display)]
pub struct DefmtError {
    #[from]
    error: DefmtInnerError,
}

#[derive(Debug, thiserror::Error)]
enum DefmtInnerError {
    #[error(".defmt data not found")]
    DefmtNotFound,

    #[error(transparent)]
    DecodeError(#[from] defmt_decoder::DecodeError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

type Result<T> = std::result::Result<T, DefmtError>;

#[derive(uniffi::Object)]
struct DefmtDecoder {
    table: Arc<Mutex<Table>>,
}

#[uniffi::export]
impl DefmtDecoder {
    #[uniffi::constructor]
    fn new(elf_path: String) -> Result<Self> {
        let bytes = fs::read(&elf_path).map_err(DefmtInnerError::from)?;
        let table = Table::parse(&bytes)
            .map_err(DefmtInnerError::from)?
            .ok_or_else(|| DefmtInnerError::DefmtNotFound)?;
        Ok(Self { table: Arc::new(Mutex::new(table)) })
    }

    #[uniffi::method(name = "DecodeFrame")]
    fn decode_frame(&self, data: &[u8]) -> Result<String> {
        let table_lock = self
            .table
            .lock()
            .expect("failed to acquire a lock for the table object");
        let (frame, _idx) = table_lock
            .decode(data).map_err(DefmtInnerError::from)?;
        Ok(frame.display(false).to_string())
    }
}

uniffi::setup_scaffolding!();
