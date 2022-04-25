use k8s_openapi::api::core::v1::Container;

// Struct used to improve works on container
#[derive(Debug, Default)]
pub struct ContainerWrapper {
    containers: Vec<Container>,
    container: Option<Container>
}

impl ContainerWrapper {
    /// Create a new ContainerWrapper
    ///
    /// # Arguments
    /// * `containers` - Vec<Container>
    pub fn new(containers: Vec<Container>) -> ContainerWrapper {
        ContainerWrapper {
            containers,
            container: None
        }
    }

    /// Set the selected container by the user, based on the given name
    ///
    /// # Arguments
    /// * `&mut self` - Self
    /// * `name` - String
    pub fn set_selected_container(&mut self, name: String) -> &mut Self {
        let mut containers: Vec<_> = self.containers
            .iter()
            .filter(|c| c.name == name)
            .collect();

        if let Some(container) = containers.pop() {
            self.container = Some(container.to_owned());
        }

        self
    }

    /// Retrieve a list of port for given saved container
    ///
    /// # Arguments
    /// * `&mut self` - Self
    pub fn get_port_for_container(&mut self) -> Option<Vec<i32>> {
        if let Some(container) = self.container.clone() {
            if let Some(ports) = container.ports {
                let port_list = ports
                    .into_iter()
                    .map(|p| p.container_port)
                    .collect::<Vec<_>>();

                return Some(port_list);
            }
        }

        None
    }

    /// Get a list of containers name
    ///
    /// # Arguments
    /// * `&self`
    pub fn get_containers_name(&self) -> Vec<String> {
        self.containers
            .to_owned()
            .into_iter()
            .map(|c| c.name)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use k8s_openapi::api::core::v1::ContainerPort;

    use super::*;

    fn setup() -> Vec<Container> {
        vec![
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
            },
            Container {
                name: "bar".to_owned(),
                ports: Some(vec![
                    ContainerPort {
                        container_port: 443,
                        host_ip: None,
                        host_port: Some(443),
                        name: Some("Https".to_owned()),
                        protocol: Some("Tcp".to_owned())
                    },
                    ContainerPort {
                        container_port: 80,
                        host_ip: None,
                        host_port: Some(80),
                        name: Some("Http".to_owned()),
                        protocol: Some("Tcp".to_owned())
                    }
                ]),
                ..Default::default()
            }
        ]
    }

    #[test]
    fn expect_to_get_containers_name() {
        let containers = setup();
        let names = ContainerWrapper::new(containers).get_containers_name();

        assert_eq!(names.get(0).unwrap(), "foo");
        assert_eq!(names.get(1).unwrap(), "bar");
    }

    #[test]
    fn expect_to_not_get_containers_name() {
        let names = ContainerWrapper::new(vec![]).get_containers_name();
        assert!(names.is_empty());
    }

    #[test]
    fn expect_to_get_port_for_one_exposed_port() {
        let containers = setup();
        let ports = ContainerWrapper::new(containers)
            .set_selected_container("foo".to_owned())
            .get_port_for_container().unwrap();

        assert_eq!(*ports.get(0).unwrap(), 3000);
    }

    #[test]
    fn expect_to_get_port_for_multiple_exposed_ports() {
        let containers = setup();
        let ports = ContainerWrapper::new(containers)
            .set_selected_container("bar".to_owned())
            .get_port_for_container().unwrap();

        assert_eq!(*ports.get(0).unwrap(), 443);
        assert_eq!(*ports.get(1).unwrap(), 80);
    }

    #[test]
    fn expect_to_not_get_ports() {
        let containers = setup();
        let ports = ContainerWrapper::new(containers)
            .set_selected_container("tada".to_owned())
            .get_port_for_container();

        assert!(ports.is_none());
    }
}
