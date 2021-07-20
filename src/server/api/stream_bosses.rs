use crate::{resources::ws, server::client::FinderClient, server::state::AppState};
use futures::stream::SplitSink;
use futures::{SinkExt, StreamExt};
use log::{error, info};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
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
  let client_tx = Arc::new(Mutex::new(client_tx));
  // Create a new thread to sending message to client
  sending_message(client_id.clone(), client_tx.clone(), receiver, app_state.clone());
  // Consuming incoming message
  while let Some(result) = client_rx.next().await {
    // Once client did not send clarify message, it will disconnect.
    let msg = if let Ok(msg) = result {
      match msg.to_str() {
        Ok(msg) => msg.to_owned(),
        Err(_) => {
          error!("Client: {}, websocket error: message should be string!", client_id);
          continue;
        }
      }
    } else {
      continue;
    };
    match msg.as_str() {
      ws::PING => {
        let pong_result = client_tx.lock().await.send(warp::ws::Message::text(ws::PONG)).await;
        // When server is unable to sent a pong pack to client, it might be disconnected.
        if pong_result.is_err() {
          app_state.clients.write().await.remove(client_id.as_str());
          info!("Client: {} gone!", client_id);
          break;
        }
      }
      msg => {
        if let Ok(request) = serde_json::from_str::<StreamRequest>(msg) {
          let mut clients = app_state.clients.write().await;
          if let Some(client) = clients.get_mut(&client_id) {
            (*client).boss_names = request.boss_names;
          }
        }
      }
    }
  }
}

///
/// A thread to forward raid tweet message to websocket client
///
/// # Arguments
/// * `client_id`: the client where we want to send, use to remove the global state when retrieving error.
/// * `client_tx`: client transportation.
/// * `receiver`: twitter stream receiver.
///
fn sending_message(
  client_id: String,
  client_tx: Arc<Mutex<SplitSink<warp::ws::WebSocket, warp::ws::Message>>>,
  receiver: UnboundedReceiverStream<Result<warp::ws::Message, warp::Error>>,
  app_state: AppState,
) {
  tokio::spawn(async move {
    let stream = receiver.then(|result| async {
      let result = match result {
        Ok(message) => client_tx.lock().await.send(message).await,
        Err(err) => Err(err),
      };
      if result.is_err() {
        app_state.clients.write().await.remove(client_id.as_str());
        info!("Client: {} gone!", client_id);
        return Err(());
      }

      Ok(())
    });
    tokio::pin!(stream);
    while stream.next().await.is_some() {}
  });
}
