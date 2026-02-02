use defmt_decoder::{DecodeError, StreamDecoder, Table};
use std::fs;
use std::marker::PhantomPinned;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use uniffi::deps::anyhow;

#[derive(Debug, thiserror::Error, uniffi::Object)]
#[error(transparent)]
#[uniffi::export(Debug, Display)]
struct DefmtError {
    #[from]
    error: DefmtInnerError,
}

#[derive(Debug, thiserror::Error)]
enum DefmtInnerError {
    #[error(".defmt data not found")]
    DefmtNotFound,

    #[error(transparent)]
    DecodeError(#[from] DecodeError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

type Result<T> = std::result::Result<T, DefmtError>;

struct DefmtDecoderInner {
    decoder: Option<Box<dyn StreamDecoder + Send + 'static>>,
    table: Pin<Box<Table>>,
    _pin: PhantomPinned,
}

impl DefmtDecoderInner {
    fn new(elf_path: &str) -> Result<Self> {
        let bytes = fs::read(&elf_path).map_err(DefmtInnerError::from)?;
        let table = Table::parse(&bytes)
            .map_err(DefmtInnerError::from)?
            .ok_or_else(|| DefmtInnerError::DefmtNotFound)?;

        let mut this = Self {
            decoder: None,
            table: Box::pin(table),
            _pin: PhantomPinned,
        };
        let table_ref: &Table = this.table.as_ref().get_ref();
        let decoder = table_ref.new_stream_decoder();
        let decoder_static = unsafe { std::mem::transmute::<_, Box<dyn StreamDecoder + Send + 'static>>(decoder) };
        this.decoder = Some(decoder_static);

        Ok(this)
    }

    pub fn decode_frame(&mut self, data: &[u8]) -> Result<Option<String>> {
        match &mut self.decoder {
            Some(decoder) => {
                decoder.received(data);
                match decoder.decode() {
                    Ok(frame) => Ok(Some(frame.display(false).to_string())),
                    Err(DecodeError::UnexpectedEof) => Ok(None),
                    Err(e) => Err(DefmtInnerError::DecodeError(e).into()),
                }
            }
            None => Ok(None),
        }
    }
}

impl Drop for DefmtDecoderInner {
    fn drop(&mut self) {
        drop(self.decoder.take());
    }
}

#[derive(uniffi::Object)]
struct DefmtDecoder {
    table: Arc<Mutex<DefmtDecoderInner>>,
}

#[uniffi::export]
impl DefmtDecoder {
    #[uniffi::constructor]
    pub fn new(elf_path: String) -> Result<Self> {
        DefmtDecoderInner::new(&elf_path).map(|inner| Self { table: Arc::new(Mutex::new(inner)) })
    }

    #[uniffi::method(name = "DecodeFrame")]
    pub fn decode_frame(&self, data: &[u8]) -> Result<Option<String>> {
        let mut guard = self.table.lock().expect("lock poisoned");
        guard.decode_frame(data)
    }
}

uniffi::setup_scaffolding!();
