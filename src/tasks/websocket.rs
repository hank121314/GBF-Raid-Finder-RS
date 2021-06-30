use crate::{proto::raid_tweet::RaidTweet, FinderClients};

pub fn sending_message_to_websocket_client(raid_tweet: RaidTweet, clients: FinderClients) {
  tokio::spawn(async move {
    let clients = clients.read().await;
    clients.iter().for_each(move |(_, client)| {
      if client.boss_names.contains(&raid_tweet.boss_name) {
        if let Ok(bytes) = raid_tweet.to_bytes() {
          let msg = warp::ws::Message::binary(bytes);
          let _ = client.sender.send(Ok(msg));
        }
      }
    });
  });
}
