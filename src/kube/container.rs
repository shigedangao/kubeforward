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
