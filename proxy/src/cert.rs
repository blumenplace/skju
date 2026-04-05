use tokio::net::TcpStream;
use tokio_rustls::server::TlsStream;
use x509_parser::{parse_x509_certificate, error::X509Error, asn1_rs::Err as AsnErr};
use x509_parser;
use itertools::Itertools;

#[derive(thiserror::Error, Debug)]
pub(crate) enum CertError {
    #[error("no peer certificate found")]
    NoCert,
    #[error(transparent)]
    X509Error(AsnErr<X509Error>),
}

type Result<T> = std::result::Result<T, CertError>;

struct ClientIdentity {
    common_name: String,
    sans: Vec<String>,
}

fn extract_identity(tls: &TlsStream<TcpStream>) -> Result<ClientIdentity> {
    let (_, server_conn) = tls.get_ref();

    let cert_der = server_conn
        .peer_certificates()
        .and_then(|c| c.first().cloned())
        .ok_or_else(|| CertError::NoCert)?;

    // Parse the DER cert to extract fields
    let (_, cert) = parse_x509_certificate(&cert_der).map_err(CertError::X509Error)?;

    let common_name = cert
        .subject()
        .iter_common_name()
        .next()
        .and_then(|cn| cn.as_str().ok())
        .unwrap_or("")
        .to_string();

    let sans = cert
        .subject_alternative_name()
        .into_iter()
        .flatten()
        .flat_map(|san| &san.value.general_names)
        .filter_map(|gn| match gn {
            x509_parser::extensions::GeneralName::DNSName(s) => Some(s.to_string()),
            _ => None,
        })
        .collect();

    Ok(ClientIdentity { common_name, sans })
}