use anyhow::{Context, Result};

use rama::graceful_shutdown::Shutdown;

use rama::tls::rustls::server::TlsAcceptorLayer;
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tracing::{info, warn};


use rama::{
    Layer,
    extensions::ExtensionsRef,
    layer::ConsumeErrLayer,
    net::{
        address::HostWithPort, forwarded::Forwarded, stream::SocketInfo,
        tls::server::SelfSignedData,
    },
    
    rt::Executor,
    service::service_fn,
    stream::Stream,
    ,
    telemetry::tracing::{
        self,
        level_filters::LevelFilter,
        subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt},
    },
    
};
use rama::tcp::{
    client::service::{Forwarder, TcpConnector},
    server::TcpListener,
};
use rama::graceful::Shutdown;
use rama::tls::rustls::server::{TlsAcceptorDataBuilder, TlsAcceptorLayer},
use rama::proxy::haproxy::{
    client::HaProxyLayer as HaProxyClientLayer, server::HaProxyLayer as HaProxyServerLayer,
},


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub listen_addr: SocketAddr,
    pub https_backend: String,
    pub mqtt_backend: String,
    pub cert_path: String,
    pub key_path: String,
    pub ca_cert_path: String,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            listen_addr: "0.0.0.0:8443".parse().unwrap(),
            https_backend: "127.0.0.1:8080".to_string(),
            mqtt_backend: "127.0.0.1:1883".to_string(),
            cert_path: "certs/server.crt".to_string(),
            key_path: "certs/server.key".to_string(),
            ca_cert_path: "certs/ca.crt".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TlsTerminationProxy {
    config: ProxyConfig,
}

impl TlsTerminationProxy {
    pub fn new(config: ProxyConfig) -> Self {
        Self { config }
    }
    
    pub async fn handle_connection(&self, stream: TcpStream, addr: SocketAddr) -> Result<()> {
        info!("Handling connection from {}", addr);
        
        // TODO: Add TLS handshake and client certificate validation here
        // TODO: Determine if this is HTTPS or MQTT traffic
        // TODO: Proxy to appropriate backend
        
        // For now, just close the connection
        drop(stream);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let acceptor_data = TlsAcceptorDataBuilder::try_new_self_signed(SelfSignedData::default())
        .expect("tls acceptor with self signed data")
        .try_with_env_key_logger()
        .expect("with env key logger")
        .build();

    shutdown.spawn_task_fn(async move |guard| {
        let tcp_service = TlsAcceptorLayer::new(acceptor_data).into_layer(
            Forwarder::new(
                Executor::graceful(guard.clone()),
                HostWithPort::local_ipv4(62800),
            )
            .with_connector(
                HaProxyClientLayer::tcp()
                    .into_layer(TcpConnector::new(Executor::graceful(guard.clone()))),
            ),
        );

        TcpListener::bind("127.0.0.1:63800", Executor::graceful(guard.clone()))
            .await
            .expect("bind TCP Listener: tls")
            .serve(tcp_service)
            .await;
    });

    shutdown
        .shutdown_with_limit(Duration::from_secs(30))
        .await
        .expect("graceful shutdown");

    /*
    let config = ProxyConfig::default();
    info!("Starting TLS/mTLS termination proxy on {}", config.listen_addr);
    
    let proxy = TlsTerminationProxy::new(config.clone());
    
    let listener = TcpListener::bind(config.listen_addr).await?;
    info!("Proxy listening on {}", config.listen_addr);
    
    // For now, use a simple server loop
    // TODO: Implement proper graceful shutdown and TLS
    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                let proxy_clone = proxy.clone();
                tokio::spawn(async move {
                    if let Err(e) = proxy_clone.handle_connection(stream, addr).await {
                        tracing::error!("Error handling connection from {}: {}", addr, e);
                    }
                });
            }
            Err(e) => {
                tracing::error!("Failed to accept connection: {}", e);
            }
        }
    }
    
    Ok(())
    */
}
