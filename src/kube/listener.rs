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
use crate::utils;

pub struct Context {
    ctx: Arc<Mutex<SendRequest<Body>>>
}

impl Context {
    pub fn new(sender: SendRequest<Body>) -> Context {
        Context { ctx: Arc::new(Mutex::new(sender)) }
    }

    pub async fn run_server(&self, machine_port: u16) {
        let make_service = make_service_fn(move |_conn| {
            let context = self.ctx.clone();
            let service = service_fn(move |req| utils::handle(context.clone(), req));
            async move { Ok::<_, Infallible>(service) }
        });

        let (tx, rx) = tokio::sync::oneshot::channel::<()>();
        let addr = SocketAddr::from(([127, 0, 0, 1], machine_port));
        let server = Server::bind(&addr)
            .serve(make_service)
            .with_graceful_shutdown(async {
                rx.await.ok();
            });

        println!("Try opening http://{0} in a browser, or `curl http://{0}`", addr);
        println!("Use Ctrl-C to stop the server and delete the pod");    

        tokio::spawn(async move {
            tokio::signal::ctrl_c().map(|_| ()).await;
            println!("stopping the server");
            let _ = tx.send(());
        });

        if let Err(err) = server.await {
            println!("ok lol {err:?}");
        }
    }
}