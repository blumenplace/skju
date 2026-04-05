use std::{
    error::Error,
    fmt::{self, Display},
    sync::atomic::{AtomicBool, Ordering},
};

use tokio::signal;
use tokio_util::sync::CancellationToken;

#[derive(Debug, PartialEq, Eq)]
pub struct AlreadyCreatedError;

impl Error for AlreadyCreatedError {}

impl Display for AlreadyCreatedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("shutdown handler already created")
    }
}

static CREATED: AtomicBool = AtomicBool::new(false);

pub struct Shutdown {
    token: CancellationToken,
}

impl Shutdown {
    pub fn new() -> Result<Shutdown, AlreadyCreatedError> {
        if (CREATED).swap(true, Ordering::SeqCst) {
            return Err(AlreadyCreatedError);
        }

        let token = CancellationToken::new();
        let child = token.clone();

        tokio::spawn(async move {
            register_handlers().await;
            child.cancel();
        });

        Ok(Self { token })
    }

    pub fn token(&self) -> CancellationToken {
        self.token.clone()
    }

    pub fn handle(&self) -> impl Future<Output = ()> + use<> {
        let token = self.token.clone();
        async move {
            token.cancelled().await;
        }
    }
}

fn register_handlers() -> impl Future<Output = ()> {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    async {
        tokio::select! {
            () = ctrl_c => {},
            () = terminate => {},
        }
        tracing::info!("shutdown signal received");
    }
}
