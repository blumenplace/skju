use defmt_decoder::Table;
use std::fs;
use std::sync::{Arc, Mutex};


#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum DefmtCsError {
    #[error(".defmt data not found")]
    DefmtNotFound,

    #[error("IO error")]
    Io,

    #[error("Anyhow")]
    Anyhow,
}

type Result<T> = std::result::Result<T, DefmtCsError>;

#[derive(uniffi::Object)]
struct DefmtDecoder {
    table: Arc<Mutex<Table>>,
}

#[uniffi::export]
impl DefmtDecoder {
    #[uniffi::constructor]
    fn new(elf_path: String) -> Result<Self> {
        let bytes = fs::read(&elf_path).map_err(|_| DefmtCsError::Io)?;
        let table = Table::parse(&bytes).map_err(|_| DefmtCsError::Anyhow)?.ok_or_else(|| DefmtCsError::DefmtNotFound)?;
        Ok(Self { table: Arc::new(Mutex::new(table)) })
    }

    #[uniffi::method(name = "DecodeFrame")]
    fn decode_frame(&self, data: &[u8]) -> Result<String> {
        let table_lock = self.table.lock().expect("failed to aquire a lock for the table object");
        let (frame, _idx) = table_lock.decode(data).map_err(|_| DefmtCsError::Anyhow)?;
        Ok(frame.display(false).to_string())
    }
}

uniffi::setup_scaffolding!();
