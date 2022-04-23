use std::sync::Arc;
use std::convert::Infallible;
use std::net::SocketAddr;
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use futures_util::FutureExt;
use kube::{api::{
    Api,
    ListParams,
}, ResourceExt};
use k8s_openapi::api::core::v1::Pod;
use super::*;
use super::container::ContainerWrapper;
use tokio::sync::Mutex;
use crate::error::KubeErr;
use crate::utils;

pub struct PodsList {
    pods: Vec<Pod>,
    selected_pod: Option<Pod>,
    context: Option<String>,
    namespace: String,
}

impl PodsList {
    /// Create a new PodsList structure
    ///     
    /// # Arguments
    /// * `context` - Option<String>
    /// * `ns` - &str
    pub async fn new(context: Option<String>, ns: &str) -> Result<PodsList, KubeErr> {
        let client = authenticate_with_cluster(&context).await?;
        let pod_api: Api<Pod> = Api::namespaced(client, &ns);
        let list = pod_api.list(&ListParams::default()).await?;

        let pods = PodsList {
            pods: list.items,
            selected_pod: None,
            context,
            namespace: ns.to_owned()
        };

        Ok(pods)
    }

    /// Get a list of pod name for the list of pods that has been founded
    /// 
    /// # Arguments
    /// * `&self` - Self
    pub fn get_pod_name_list(&self) -> Vec<String> {
        self.pods
            .to_owned()
            .into_iter()
            .map(|p| p.metadata.name)
            .filter(|p| p.is_some())
            .map(|p| p.unwrap())
            .collect::<Vec<_>>()
    }

    /// Save the selected pod on the current struct
    /// 
    /// # Arguments
    /// * `&mut self` - Self
    /// * `pod_name` - String
    pub fn set_selected_pod(&mut self, pod_name: String) -> &mut Self {
        let mut pod: Vec<Pod> = self.pods
            .to_owned()
            .into_iter()
            .filter(|p| {
                if let Some(name) = &p.metadata.name {
                    if *name == pod_name {
                        return true;
                    }
                }

                false
            })
            .collect();

        self.selected_pod = pod.pop();

        self
    }

    /// Get a list of containers name for a selected pod
    /// 
    /// # Arguments
    /// * `&self` - Self
    pub fn list_containers(&self) -> Option<Vec<String>> {
        if let Some(pod) = self.selected_pod.to_owned() {
            if let Some(spec) = pod.spec {
                let names: Vec<String> = spec.containers
                    .into_iter()
                    .map(|c| c.name)
                    .collect();

                return Some(names);
            }
        }

        None
    }

    /// Get the port for the selected pod and the selected container
    /// 
    /// # Arguments
    /// * `&self` - Self
    /// * `selected_container` - Option<String>
    pub fn get_port_for_container(&self, selected_container: String) -> Option<Vec<i32>> {
        if let Some(pod) = self.selected_pod.to_owned() {
            if let Some(spec) = pod.spec {
                let mut containers = ContainerWrapper::new(spec.containers);
                let ports = containers
                    .set_selected_container(selected_container)
                    .get_port_for_container();
    
                return ports;
            }    
        }

        None
    }

    /// 
    /// Implementation is based on
    /// @link https://github.com/kube-rs/kube-rs/blob/master/examples/pod_portforward_bind.rs
    pub async fn expose_pod(&self, selected_port: i32, user_port: &str) -> Result<(), KubeErr> {
        let user_port_u16 = user_port.parse::<u16>()?;
        let selected_pod = self.selected_pod.to_owned().unwrap();
        
        let client = authenticate_with_cluster(&self.context).await?;
        let pod_api: Api<Pod> = Api::namespaced(client, &self.namespace);
        
        let mut forwarder = pod_api.portforward(&selected_pod.name(), &[selected_port as u16]).await?;
        let local_port = forwarder
            .take_stream(selected_port as u16)
            .expect("Return a stream from the port");
        
        let (sender, connection) = hyper::client::conn::handshake(local_port).await?;
        // listen to error with the connection
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                println!("error in connection: {}", e);
            }
        });

        // Task use to show error from the forwarder
        tokio::spawn(async move {
            if let Err(e) = forwarder.join().await {
                println!("forwarder error {}", e);
            }
        });

        let ctx = Arc::new(Mutex::new(sender));
        let make_service = make_service_fn(move |_conn| {
            let context = ctx.clone();
            let service = service_fn(move |req| utils::handle(context.clone(), req));
            async move { Ok::<_, Infallible>(service) }
        });

        let (tx, rx) = tokio::sync::oneshot::channel::<()>();
        let addr = SocketAddr::from(([127, 0, 0, 1], user_port_u16));
        let server = Server::bind(&addr)
            .serve(make_service)
            .with_graceful_shutdown(async {
                rx.await.ok();
            });

        println!("Forwarding http://{} to port {} in the pod", addr, user_port);
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

        Ok(())
    }
}