use tokio::sync::mpsc;

///
/// Granblue Fantasy Finder websocket client
/// 
/// # Arguments
/// * `boss_names`: came from client message, should include the raid boss name which client want to receive.
/// * `sender`: A sender which can send the tweet to the client.
/// 
#[derive(Clone)]
pub struct FinderClient {
  pub boss_names: Vec<String>,
  pub sender: mpsc::UnboundedSender<std::result::Result<warp::ws::Message, warp::Error>>,
}

impl FinderClient {
  pub fn new<V, S>(
    boss_names: V,
    sender: mpsc::UnboundedSender<std::result::Result<warp::ws::Message, warp::Error>>,
  ) -> Self
  where
    S: Into<String>,
    V: IntoIterator<Item = S>,
  {
    FinderClient {
      boss_names: boss_names.into_iter().map(|k| k.into()).collect::<Vec<_>>(),
      sender,
    }
  }
}
