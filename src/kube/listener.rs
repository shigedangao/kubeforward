use std::{
    sync::Arc,
    convert::Infallible,
    net::SocketAddr
};
use tokio::sync::Mutex;
use hyper::{
    service::{make_service_fn, service_fn},
    client::conn::SendRequest,
    Body,
    Server
};
use futures_util::FutureExt;
use crate::{utils, error::KubeErr};

// Constant
const STOP_SERVER_MSG: &str = "Stopping port forwarding";
const CTRL_C_MSG: &str = "Use Ctrl-C / Cmd-C to stop the server";
const LOCAL_ADDR: [u8; 4] = [127, 0, 0, 1];

pub struct Context {
    ctx: Arc<Mutex<SendRequest<Body>>>
}

impl Context {
    /// Create a new Context
    ///
    /// # Arguments
    /// * `sender` - SendRequest<Body>
    pub fn new(sender: SendRequest<Body>) -> Context {
        Context { ctx: Arc::new(Mutex::new(sender)) }
    }

    /// Forward the pod port to the local machine port by creating a new hyper server
    ///
    /// # Arguments
    /// * `&self` - Self
    /// * `machine_port` - u16
    pub async fn port_forward_local(&self, machine_port: u16) -> Result<(), KubeErr> {
        let make_service = make_service_fn(move |_conn| {
            let context = self.ctx.clone();
            let service = service_fn(move |req| utils::handle(context.clone(), req));
            async move { Ok::<_, Infallible>(service) }
        });

        // a oneshot channel is used only to listen for the ctrl-c command
        // when a ctrl-c command has been executed by the user
        // we're sending an unique message which will terminate gracefully terminate the server
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();
        let addr = SocketAddr::from((LOCAL_ADDR, machine_port));
        let server = Server::bind(&addr)
            .serve(make_service)
            .with_graceful_shutdown(async {
                rx.await.ok();
            });

        log::info!("Exposing the pod to the local port of:  {addr}");
        log::info!("{CTRL_C_MSG}");

        // Use to listen to the ctrl_c / cmd_c command
        tokio::spawn(async move {
            tokio::signal::ctrl_c().map(|_| ()).await;
            log::warn!("{STOP_SERVER_MSG}");
            let _ = tx.send(());
        });

        server
            .await
            .map_err(|err| KubeErr::Network(err.to_string()))
    }
}
