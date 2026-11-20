use super::session::run_server;
use super::tls::ServerTls;
use crate::TcpError;
use leocard_host::HostSession;
use std::net::SocketAddr;
use tokio::net::{TcpListener, ToSocketAddrs};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

pub struct TcpServerHandle {
    local_addr: SocketAddr,
    shutdown: Option<oneshot::Sender<()>>,
    task: JoinHandle<Result<(), TcpError>>,
}

impl TcpServerHandle {
    pub async fn bind(address: impl ToSocketAddrs, session: HostSession) -> Result<Self, TcpError> {
        Self::bind_inner(address, session, None).await
    }

    /// 同一端口同时接受原有 TCP、WebSocket 和使用指定证书的 WSS。
    pub async fn bind_with_tls(
        address: impl ToSocketAddrs,
        session: HostSession,
        tls: ServerTls,
    ) -> Result<Self, TcpError> {
        Self::bind_inner(address, session, Some(tls)).await
    }

    async fn bind_inner(
        address: impl ToSocketAddrs,
        session: HostSession,
        tls: Option<ServerTls>,
    ) -> Result<Self, TcpError> {
        let listener = TcpListener::bind(address).await?;
        let local_addr = listener.local_addr()?;
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let task = tokio::spawn(run_server(listener, session, shutdown_rx, tls));
        Ok(Self {
            local_addr,
            shutdown: Some(shutdown_tx),
            task,
        })
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    pub async fn shutdown(mut self) -> Result<(), TcpError> {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        self.task
            .await
            .map_err(|error| TcpError::ServerTask(error.to_string()))?
    }
}
