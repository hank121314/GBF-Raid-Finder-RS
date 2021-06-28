use crate::{server::client::FinderClient, server::state::AppState};
use futures::stream::SplitSink;
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
  // Generate client uuid
  let client_id = nanoid::nanoid!();
  // Get client transportation
  let (client_tx, mut client_rx) = ws.split();
  // Create a channel between ws and tweet stream
  let (tx, rx) = mpsc::unbounded_channel();
  // Create finder client and inset it into global state
  let client = FinderClient::new([""], tx);
  app_state.clients.write().await.insert(client_id.clone(), client);
  info!("Client: {} incoming...", client_id);
  // Receiver for tweet stream
  let receiver = UnboundedReceiverStream::new(rx);
  // Create a new thread to sending message to client
  sending_message(client_id.clone(), client_tx, receiver, app_state.clone());
  // Consuming incoming message
  while let Some(result) = client_rx.next().await {
    // Once client did not send clarify message, it will disconnect.
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
      if let Some(client) = clients.get_mut(&client_id) {
        (*client).boss_names = request.boss_names;
      }
    }
  }
}

fn sending_message(
  client_id: String,
  client_tx: SplitSink<warp::ws::WebSocket, warp::ws::Message>,
  receiver: UnboundedReceiverStream<Result<warp::ws::Message, warp::Error>>,
  app_state: AppState,
) {
  tokio::spawn(receiver.forward(client_tx).then(|result| async move {
    if let Err(_) = result {
      app_state.clients.write().await.remove(client_id.as_str());
      info!("Client: {} gone!", client_id);
    }
  }));
}
