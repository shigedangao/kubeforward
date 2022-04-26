use kube::{
    api::{Api, ListParams},
    ResourceExt
};
use k8s_openapi::api::core::v1::Pod;
use super::{
    container::ContainerWrapper,
    listener::Context,
    *
};
use crate::error::KubeErr;
use crate::utils;

// Struct used to improve work on pods
pub struct PodsList {
    client: Option<Client>,
    namespace: String,
    pods: Vec<Pod>,
    selected_pod: Option<Pod>,
    container_wrapper: ContainerWrapper
}

impl PodsList {
    /// Create a new PodsList structure
    ///
    /// # Arguments
    /// * `context` - Option<String>
    /// * `ns` - &str
    pub async fn new(context: Option<String>, ns: &str) -> Result<PodsList, KubeErr> {
        let client = authenticate_with_cluster(&context).await?;
        let pod_api: Api<Pod> = Api::namespaced(client.clone(), &ns);
        let list = pod_api.list(&ListParams::default()).await?;

        let pods = PodsList {
            client: Some(client),
            namespace: ns.to_owned(),
            pods: list.items,
            selected_pod: None,
            container_wrapper: ContainerWrapper::default()
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
            .filter_map(|p| p.metadata.name)
            .collect::<Vec<_>>()
    }

    /// Save the selected pod on the current struct
    ///
    /// # Arguments
    /// * `&mut self` - Self
    /// * `pod_name` - String
    pub fn set_selected_pod(&mut self, pod_name: String) -> &mut Self {
        let mut pod: Vec<_> = self.pods
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
        self.set_containers_for_selected_pod();

        self
    }

    /// Get a list of containers name for a selected pod
    ///
    /// # Arguments
    /// * `&self` - Self
    pub fn list_containers(&mut self) -> Vec<String> {
        self.container_wrapper.get_containers_name()
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

    /// Expose the pod based on the container pord and the given user port
    /// Implementation is highly inspired by the link below
    /// @link https://github.com/kube-rs/kube-rs/blob/master/examples/pod_portforward_bind.rs
    ///
    /// # Arguments
    /// * `&self` - Self
    /// * `selected_port` - u16
    /// * `user_port` - u16
    pub async fn expose_pod(&self, selected_port: u16, user_port: u16) -> Result<(), KubeErr> {
        if self.selected_pod.is_none() {
            return Err(KubeErr::SelectedPod);
        }

        if self.client.is_none() {
            return Err(KubeErr::Kube("Could not connect retrieve client handler".to_owned()));
        }

        let selected_pod = self.selected_pod.to_owned().unwrap();
        let client = self.client.clone().unwrap();
        let pod_api: Api<Pod> = Api::namespaced(client, &self.namespace);

        let mut forwarder = pod_api.portforward(&selected_pod.name(), &[selected_port as u16]).await?;
        let local_port = forwarder
            .take_stream(selected_port as u16)
            .ok_or_else(|| KubeErr::ForwardPort)?;

        let (sender, connection) = hyper::client::conn::handshake(local_port).await?;

        // listen to errors by spawning a new task
        utils::listen_conn_error(connection);
        utils::listen_forwarder_error(forwarder);

        Context::new(sender)
            .port_forward_local(user_port)
            .await
    }

    /// Set the containers for a selected pod to expose
    ///
    /// # Arguments
    /// * `&mut self` - Self
    fn set_containers_for_selected_pod(&mut self) {
        if let Some(pod) = self.selected_pod.to_owned() {
            if let Some(spec) = pod.spec {
                self.container_wrapper = ContainerWrapper::new(spec.containers);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use k8s_openapi::api::core::v1::{PodSpec, Container, ContainerPort};
    use kube::core::ObjectMeta;

    use super::*;

    fn setup() -> PodsList {
        let pod = Pod {
            spec: Some(PodSpec {
                containers: vec![
                    Container {
                        name: "foo".to_owned(),
                        ports: Some(vec![ContainerPort {
                            container_port: 3000,
                            host_ip: None,
                            host_port: Some(80),
                            name: Some("Http".to_owned()),
                            protocol: Some("Tcp".to_owned())
                        }]),
                        ..Default::default()
                    }
                ],
                ..Default::default()
            }),
            ..Default::default()
        };

        PodsList {
            client: None,
            namespace: "default".to_owned(),
            pods: vec![
                Pod {
                    metadata: ObjectMeta {
                        name: Some("foo".to_owned()),
                        ..Default::default()
                    },
                    ..pod
                }
            ],
            selected_pod: None,
            container_wrapper: ContainerWrapper::default()
        }
    }

    #[test]
    fn expect_to_get_pod_names() {
        let pod_list = setup();
        let names = pod_list.get_pod_name_list();

        assert_eq!(names.get(0).unwrap(), "foo");
    }

    #[test]
    fn expect_to_get_pod_port_container() {
        let mut pod_list = setup();
        let container_port = pod_list
            .set_selected_pod("foo".to_owned())
            .get_port_for_container("foo".to_owned());

        assert!(container_port.is_some());
        let container_port = container_port.unwrap();

        assert_eq!(*container_port.get(0).unwrap(), 3000);
    }

    #[test]
    fn expect_to_not_get_pod_port() {
        let pod_list = setup();
        let container_port = pod_list
            .get_port_for_container("foo".to_owned());

        assert!(container_port.is_none());
    }
}
