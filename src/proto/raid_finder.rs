use crate::proto::{raid_boss::RaidBoss, raid_tweet::RaidTweet};

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetBossRequest {
  #[prost(uint32, tag = "1")]
  pub level: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetPersistenceBossRequest {
  #[prost(string, tag = "1")]
  pub boss_name: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetPersistenceBossResponse {
  #[prost(message, repeated, tag = "1")]
  pub tweets: ::prost::alloc::vec::Vec<RaidTweet>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetBossResponse {
  #[prost(message, repeated, tag = "1")]
  pub bosses: ::prost::alloc::vec::Vec<RaidBoss>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StreamRequest {
  #[prost(string, repeated, tag = "1")]
  pub boss_names: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[doc = r" Generated server implementations."]
pub mod raid_finder_server {
  #![allow(unused_variables, dead_code, missing_docs)]
  use tonic::codegen::*;
  use crate::proto::raid_tweet::RaidTweet;
  #[doc = "Generated trait containing gRPC methods that should be implemented for use with RaidFinderServer."]
  #[async_trait]
  pub trait RaidFinder: Send + Sync + 'static {
    async fn get_bosses(
      &self,
      request: tonic::Request<super::GetBossRequest>,
    ) -> Result<tonic::Response<super::GetBossResponse>, tonic::Status>;
    async fn get_persistence_boss(
      &self,
      request: tonic::Request<super::GetPersistenceBossRequest>,
    ) -> Result<tonic::Response<super::GetPersistenceBossResponse>, tonic::Status>;
    #[doc = "Server streaming response type for the start_stream method."]
    type StartStreamStream: futures_core::Stream<Item = Result<RaidTweet, tonic::Status>>
    + Send
    + Sync
    + 'static;
    async fn start_stream(
      &self,
      request: tonic::Request<super::StreamRequest>,
    ) -> Result<tonic::Response<Self::StartStreamStream>, tonic::Status>;
  }
  #[derive(Debug)]
  pub struct RaidFinderServer<T: RaidFinder> {
    inner: _Inner<T>,
  }
  struct _Inner<T>(Arc<T>, Option<tonic::Interceptor>);
  impl<T: RaidFinder> RaidFinderServer<T> {
    pub fn new(inner: T) -> Self {
      let inner = Arc::new(inner);
      let inner = _Inner(inner, None);
      Self { inner }
    }
    pub fn with_interceptor(inner: T, interceptor: impl Into<tonic::Interceptor>) -> Self {
      let inner = Arc::new(inner);
      let inner = _Inner(inner, Some(interceptor.into()));
      Self { inner }
    }
  }
  impl<T, B> Service<http::Request<B>> for RaidFinderServer<T>
    where
      T: RaidFinder,
      B: HttpBody + Send + Sync + 'static,
      B::Error: Into<StdError> + Send + 'static,
  {
    type Response = http::Response<tonic::body::BoxBody>;
    type Error = Never;
    type Future = BoxFuture<Self::Response, Self::Error>;
    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
      Poll::Ready(Ok(()))
    }
    fn call(&mut self, req: http::Request<B>) -> Self::Future {
      let inner = self.inner.clone();
      match req.uri().path() {
        "/raid_finder.service.RaidFinder/get_bosses" => {
          #[allow(non_camel_case_types)]
          struct get_bossesSvc<T: RaidFinder>(pub Arc<T>);
          impl<T: RaidFinder> tonic::server::UnaryService<super::GetBossRequest> for get_bossesSvc<T> {
            type Response = super::GetBossResponse;
            type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
            fn call(&mut self, request: tonic::Request<super::GetBossRequest>) -> Self::Future {
              let inner = self.0.clone();
              let fut = async move { (*inner).get_bosses(request).await };
              Box::pin(fut)
            }
          }
          let inner = self.inner.clone();
          let fut = async move {
            let interceptor = inner.1.clone();
            let inner = inner.0;
            let method = get_bossesSvc(inner);
            let codec = tonic::codec::ProstCodec::default();
            let mut grpc = if let Some(interceptor) = interceptor {
              tonic::server::Grpc::with_interceptor(codec, interceptor)
            } else {
              tonic::server::Grpc::new(codec)
            };
            let res = grpc.unary(method, req).await;
            Ok(res)
          };
          Box::pin(fut)
        }
        "/raid_finder.service.RaidFinder/get_persistence_boss" => {
          #[allow(non_camel_case_types)]
          struct get_persistence_bossSvc<T: RaidFinder>(pub Arc<T>);
          impl<T: RaidFinder> tonic::server::UnaryService<super::GetPersistenceBossRequest> for get_persistence_bossSvc<T> {
            type Response = super::GetPersistenceBossResponse;
            type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
            fn call(&mut self, request: tonic::Request<super::GetPersistenceBossRequest>) -> Self::Future {
              let inner = self.0.clone();
              let fut = async move { (*inner).get_persistence_boss(request).await };
              Box::pin(fut)
            }
          }
          let inner = self.inner.clone();
          let fut = async move {
            let interceptor = inner.1.clone();
            let inner = inner.0;
            let method = get_persistence_bossSvc(inner);
            let codec = tonic::codec::ProstCodec::default();
            let mut grpc = if let Some(interceptor) = interceptor {
              tonic::server::Grpc::with_interceptor(codec, interceptor)
            } else {
              tonic::server::Grpc::new(codec)
            };
            let res = grpc.unary(method, req).await;
            Ok(res)
          };
          Box::pin(fut)
        }
        "/raid_finder.service.RaidFinder/start_stream" => {
          #[allow(non_camel_case_types)]
          struct start_streamSvc<T: RaidFinder>(pub Arc<T>);
          impl<T: RaidFinder> tonic::server::ServerStreamingService<super::StreamRequest> for start_streamSvc<T> {
            type Response = RaidTweet;
            type ResponseStream = T::StartStreamStream;
            type Future = BoxFuture<tonic::Response<Self::ResponseStream>, tonic::Status>;
            fn call(&mut self, request: tonic::Request<super::StreamRequest>) -> Self::Future {
              let inner = self.0.clone();
              let fut = async move { (*inner).start_stream(request).await };
              Box::pin(fut)
            }
          }
          let inner = self.inner.clone();
          let fut = async move {
            let interceptor = inner.1;
            let inner = inner.0;
            let method = start_streamSvc(inner);
            let codec = tonic::codec::ProstCodec::default();
            let mut grpc = if let Some(interceptor) = interceptor {
              tonic::server::Grpc::with_interceptor(codec, interceptor)
            } else {
              tonic::server::Grpc::new(codec)
            };
            let res = grpc.server_streaming(method, req).await;
            Ok(res)
          };
          Box::pin(fut)
        }
        _ => Box::pin(async move {
          Ok(
            http::Response::builder()
              .status(200)
              .header("grpc-status", "12")
              .header("content-type", "application/grpc")
              .body(tonic::body::BoxBody::empty())
              .unwrap(),
          )
        }),
      }
    }
  }
  impl<T: RaidFinder> Clone for RaidFinderServer<T> {
    fn clone(&self) -> Self {
      let inner = self.inner.clone();
      Self { inner }
    }
  }
  impl<T: RaidFinder> Clone for _Inner<T> {
    fn clone(&self) -> Self {
      Self(self.0.clone(), self.1.clone())
    }
  }
  impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      write!(f, "{:?}", self.0)
    }
  }
  impl<T: RaidFinder> tonic::transport::NamedService for RaidFinderServer<T> {
    const NAME: &'static str = "raid_finder.service.RaidFinder";
  }
}
