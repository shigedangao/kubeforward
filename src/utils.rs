use std::sync::Arc;
use tokio::sync::Mutex;
use std::convert::Infallible;
use tower::util::ServiceExt;
use hyper::{
    Request,
    Body,
    Response
};

pub async fn handle(
    context: Arc<Mutex<hyper::client::conn::SendRequest<hyper::Body>>>,
    req: Request<Body>,
) -> Result<Response<Body>, Infallible> {
    let mut sender = context.lock().await;
    let response = sender.ready().await.unwrap().send_request(req).await.unwrap();
    Ok(response)
}