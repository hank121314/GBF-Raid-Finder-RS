use crate::{
  client::redis::Redis,
  server::{api, body_parser::post_json, state::AppState},
  FinderClients,
};
use log::info;
use std::{net::SocketAddr, sync::Arc};
use warp::Filter;

///
/// Create an http server listening on port 50051
/// 
/// # Arguments
/// * `redis` - Granblue fantasy finder rs backend database client
/// * `finder_clients` - a map of clients.
/// 
pub fn create_http_server(redis: Arc<Redis>, finder_clients: FinderClients) {
  let app_state = AppState::new(redis, finder_clients);

  let server = warp::any().map(move || app_state.clone());

  let healthz_route = warp::get()
    .and(warp::path("healthz"))
    .and(warp::path::end())
    .and(server.clone())
    .map(api::healthz::healthz);

  let get_bosses_route = warp::post()
    .and(warp::path("get_bosses"))
    .and(warp::path::end())
    .and(post_json())
    .and(server.clone())
    .and_then(api::get_bosses::get_bosses);

  let get_persistence_boss = warp::post()
    .and(warp::path("get_persistence_boss"))
    .and(warp::path::end())
    .and(post_json())
    .and(server.clone())
    .and_then(api::get_persistence_boss::get_persistence_boss);

  let stream_bosses_route = warp::path("stream_bosses")
    // The `ws()` filter will prepare the Websocket handshake.
    .and(warp::ws())
    .and(server)
    .map(|ws: warp::ws::Ws, state: AppState| {
      // And then our closure will be called when it completes...
      ws.on_upgrade(move |websocket| api::stream_bosses::stream_bosses(websocket, state))
    });

  let routes = healthz_route
    .or(get_bosses_route)
    .or(get_persistence_boss)
    .or(stream_bosses_route);

  let addr = "0.0.0.0:50051".parse::<SocketAddr>().unwrap();

  info!("gRPC server listening on {}...", addr);

  tokio::spawn(async move {
    warp::serve(routes).run(addr).await;
  });
}
