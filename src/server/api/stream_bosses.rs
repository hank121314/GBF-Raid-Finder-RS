use crate::{error, resources::ws, server::client::FinderClient, server::state::AppState};
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

enum WebsocketMsgType {
  Request(String),
  Json(StreamRequest),
  Pong,
  NoneString,
}

pub async fn stream_bosses(ws: warp::ws::WebSocket, app_state: AppState) {
  // Generate client uuid
  let client_id = nanoid::nanoid!();
  // Get client transportation
  let (client_tx, client_rx) = ws.split();
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
  let stream = client_rx
    .then(|result| async {
      match result {
        Ok(msg) => match msg.to_str() {
          Ok(msg) => match msg {
            ws::PING => Ok(WebsocketMsgType::Pong),
            msg => Ok(WebsocketMsgType::Request(msg.to_owned())),
          },
          Err(_) => match msg.is_close() {
            true => Err(error::Error::WebsocketClientClose),
            false => Ok(WebsocketMsgType::NoneString),
          },
        },
        Err(error) => Err(error::Error::WebsocketClient { error }),
      }
    })
    .then(|result| async {
      match result {
        Ok(msg) => match msg {
          WebsocketMsgType::Request(ref s) => match serde_json::from_str::<StreamRequest>(s) {
            Ok(json) => Ok(WebsocketMsgType::Json(json)),
            Err(_) => Ok(WebsocketMsgType::Request(s.to_owned())),
          },
          all => Ok(all),
        },
        Err(e) => Err(e),
      }
    });
  tokio::pin!(stream);
  while let Some(result) = stream.next().await {
    if let Ok(msg) = result {
      match msg {
        WebsocketMsgType::Json(json) => {
          let mut clients = app_state.clients.write().await;
          if let Some(client) = clients.get_mut(&client_id) {
            (*client).boss_names = json.boss_names;
          }
        }
        WebsocketMsgType::Pong => {
          let pong_result = client_tx.lock().await.send(warp::ws::Message::text(ws::PONG)).await;
          // When server is unable to sent a pong pack to client, it might be disconnected.
          if pong_result.is_err() {
            app_state.clients.write().await.remove(client_id.as_str());
            info!("Client: {} gone!", client_id);
            break;
          }
        }
        WebsocketMsgType::Request(s) => {
          info!("Client: {}, cannot convert message to string, {}", client_id, s);
          continue;
        }
        WebsocketMsgType::NoneString => {
          error!("Client: {}, websocket error: message should be string!", client_id);
          continue;
        }
      }
    } else {
      app_state.clients.write().await.remove(client_id.as_str());
      info!("Client: {} gone!", client_id);
      break;
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

      match result {
        Ok(_) => Ok(()),
        Err(error) => {
          app_state.clients.write().await.remove(client_id.as_str());
          info!("Client: {} gone!", client_id);
          Err(error::Error::WebsocketClient { error })
        }
      }
    });
    tokio::pin!(stream);
    while let Some(result) = stream.next().await {
      if result.is_err() {
        break;
      }
    }

    info!("Client {} sending message stream end.", client_id);
    Ok::<(), error::Error>(())
  });
}
