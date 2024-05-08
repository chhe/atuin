use eyre::{eyre, Result};
#[cfg(not(windows))]
use tokio::net::UnixStream;
#[cfg(not(windows))]
use tonic::transport::Uri;
use tonic::transport::{Channel, Endpoint};
#[cfg(not(windows))]
use tower::service_fn;

use atuin_client::history::History;

use crate::history::{
    history_client::HistoryClient as HistoryServiceClient, EndHistoryRequest, StartHistoryRequest,
};

pub struct HistoryClient {
    client: HistoryServiceClient<Channel>,
}

// Wrap the grpc client
impl HistoryClient {
    pub async fn new(socket_path_or_port: String) -> Result<Self> {
        #[cfg(not(windows))]
        let channel = Endpoint::try_from("http://atuin_local_daemon:0")?
            .connect_with_connector(service_fn(move |_: Uri| {
                let path = path.to_string();

                UnixStream::connect(path)
            }))
            .await
            .map_err(|_| eyre!("failed to connect to local atuin daemon. Is it running?"))?;

        #[cfg(windows)]
        let channel = Endpoint::try_from(format!("http://localhost:{}", socket_path_or_port))?
            .connect()
            .await
            .map_err(|_| eyre!("failed to connect to local atuin daemon. Is it running?"))?;

        let client = HistoryServiceClient::new(channel);

        Ok(HistoryClient { client })
    }

    pub async fn start_history(&mut self, h: History) -> Result<String> {
        let req = StartHistoryRequest {
            command: h.command,
            cwd: h.cwd,
            hostname: h.hostname,
            session: h.session,
            timestamp: h.timestamp.unix_timestamp_nanos() as u64,
        };

        let resp = self.client.start_history(req).await?;

        Ok(resp.into_inner().id)
    }

    pub async fn end_history(
        &mut self,
        id: String,
        duration: u64,
        exit: i64,
    ) -> Result<(String, u64)> {
        let req = EndHistoryRequest { id, duration, exit };

        let resp = self.client.end_history(req).await?;
        let resp = resp.into_inner();

        Ok((resp.id, resp.idx))
    }
}
