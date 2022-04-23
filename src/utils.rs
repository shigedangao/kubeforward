use std::sync::Arc;
use tokio::{sync::Mutex, io::AsyncRead, io::AsyncWrite};
use std::convert::Infallible;
use tower::util::ServiceExt;
use kube::api::Portforwarder;
use hyper::{
    Request,
    Body,
    Response,
    client::conn::Connection,
    body::HttpBody
};

pub async fn handle(
    context: Arc<Mutex<hyper::client::conn::SendRequest<hyper::Body>>>,
    req: Request<Body>,
) -> Result<Response<Body>, Infallible> {
    let mut sender = context.lock().await;
    let response = sender.ready().await.unwrap().send_request(req).await.unwrap();
    Ok(response)
}

/// Listen to the connection error in a tokio task. The signature come from the hyper crate
/// 
/// # Arguments
/// * `conn` - Connection<T, B>
pub fn listen_conn_error<T, B>(conn: Connection<T, B>) where 
    T: Send + AsyncWrite + AsyncRead + Unpin,
    B: HttpBody + 'static + Send,
    B::Data: Send,
    B::Error: Into<Box<dyn std::error::Error + Send + Sync>>
{
    // listen to error with the connection
    tokio::spawn(async move {
        if let Err(e) = conn.await {
            println!("error in connection: {}", e);
        }
    });
}

/// Listen to forwarder error (Usually come from the client)
///
/// # Arguments
/// * `f` - PortForwarder
pub fn listen_forwarder_error(f: Portforwarder) {
    tokio::spawn(async move {
        if let Err(e) = f.join().await {
            println!("forwarder error {}", e);
        }
    });
}