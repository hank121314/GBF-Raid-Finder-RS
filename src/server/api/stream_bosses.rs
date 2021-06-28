use crate::{server::client::FinderClient, server::state::AppState};
use futures::{FutureExt, StreamExt};
use log::{error, info};
use serde::Deserialize;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

#[derive(Deserialize, Clone)]
pub struct StreamRequest {
  pub boss_names: Vec<String>,
}

pub async fn stream_bosses(ws: warp::ws::WebSocket, app_state: AppState) {
  let client_id = nanoid::nanoid!();
  let (client_tx, mut client_rx) = ws.split();
  if let Some(result) = client_rx.next().await {
    let msg = if let Ok(msg) = result {
      match msg.to_str() {
        Ok(msg) => msg.to_owned(),
        Err(_) => {
          error!("Client: {}, websocket error: message should be string!", client_id);
          return;
        }
      }
    } else {
      return;
    };
    if let Ok(request) = serde_json::from_str::<StreamRequest>(msg.as_str()) {
      let mut clients = app_state.clients.write().await;
      let (tx, rx) = mpsc::unbounded_channel();
      let client = FinderClient::new(request.boss_names, tx);
      info!("Client: {} incoming...", client_id);
      clients.insert(client_id.clone(), client);
      drop(clients);
      let receiver = UnboundedReceiverStream::new(rx);
      tokio::spawn(receiver.forward(client_tx).then(|result| async move {
        if let Err(_) = result {
          let mut clients = app_state.clients.write().await;
          clients.remove(client_id.as_str());
          info!("Client: {} gone!", client_id);
        }
      }));
    }
  }
}
