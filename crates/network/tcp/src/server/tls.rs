use crate::TcpError;
use std::sync::Arc;
use tokio_rustls::TlsAcceptor;
use tokio_rustls::rustls::pki_types::pem::PemObject;
use tokio_rustls::rustls::pki_types::{CertificateDer, PrivateKeyDer};
use tokio_rustls::rustls::{ServerConfig, crypto::ring};

/// 证书由房主提供，或由 frpc 在应用外终止 TLS。
#[derive(Clone)]
pub struct ServerTls {
    pub(super) acceptor: TlsAcceptor,
}

impl ServerTls {
    pub fn from_pem(certificate_chain: &[u8], private_key: &[u8]) -> Result<Self, TcpError> {
        let certificates = CertificateDer::pem_slice_iter(certificate_chain)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| TcpError::Tls(error.to_string()))?;
        let key = PrivateKeyDer::from_pem_slice(private_key)
            .map_err(|error| TcpError::Tls(error.to_string()))?;
        let config = ServerConfig::builder_with_provider(Arc::new(ring::default_provider()))
            .with_safe_default_protocol_versions()
            .map_err(|error| TcpError::Tls(error.to_string()))?
            .with_no_client_auth()
            .with_single_cert(certificates, key)
            .map_err(|error| TcpError::Tls(error.to_string()))?;
        Ok(Self {
            acceptor: TlsAcceptor::from(Arc::new(config)),
        })
    }
}
