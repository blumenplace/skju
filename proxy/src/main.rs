use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::{TlsAcceptor, rustls};
use rustls::server::{ServerConfig, ClientCertVerifier, WebPkiClientVerifier};
use std::sync::Arc;
use dotenvy::dotenv;


mod cert;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenv()?;

    let config = make_tls_config()?;
    let acceptor = TlsAcceptor::from(Arc::new(config));
    let listener = TcpListener::bind("0.0.0.0:8883").await?;

    loop {
        let (stream, peer_addr) = listener.accept().await?;
        let acceptor = acceptor.clone();

        tokio::spawn(async move {
            match handle(stream, acceptor).await {
                Ok(_) => {}
                Err(e) => tracing::warn!(%peer_addr, "connection error: {e}"),
            }
        });
    }
}

async fn handle(stream: TcpStream, acceptor: TlsAcceptor) -> anyhow::Result<()> {
    let tls = acceptor.accept(stream).await?;

    let identity = extract_identity(&tls)?;
    tracing::info!(cn = %identity.common_name, "client connected");

    let host_port = dotenvy::var("SKJU_PROXY_HOST")?;

    let upstream = TcpStream::connect(host_port).await?;

    let (mut client_rd, mut client_wr) = tokio::io::split(tls);
    let (mut upstream_rd, mut upstream_wr) = tokio::io::split(upstream);

    let client_to_upstream = tokio::io::copy(&mut client_rd, &mut upstream_wr);
    let upstream_to_client = tokio::io::copy(&mut upstream_rd, &mut client_wr);

    tokio::try_join!(client_to_upstream, upstream_to_client)?;

    Ok(())
}

fn make_tls_config() -> anyhow::Result<ServerConfig> {

    let ca_pem = dotenvy::var("SKJU_PROXY_CA")?;
    let server_cert_chain = dotenvy::var("SKJU_PROXY_CERT_CHAIN")?;
    let server_cert_key = dotenvy::var("SKJU_PROXY_CERT_KEY")?;

    let ca_cert = load_ca_cert(ca_pem)?;
    let server_cert_chain = load_cert_chain(server_cert_chain)?;
    let server_key = load_private_key(server_cert_key)?;

    let mut roots = rustls::RootCertStore::empty();
    roots.add(ca_cert)?;

    let verifier = WebPkiClientVerifier::builder(Arc::new(roots))
        .build()?;

    let config = ServerConfig::builder()
        .with_client_cert_verifier(verifier)
        .with_single_cert(server_cert, server_key)?;

    Ok(config)
}
