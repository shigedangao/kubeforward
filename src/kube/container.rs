use k8s_openapi::api::core::v1::Container;

#[derive(Debug)]
pub struct ContainerWrapper {
    containers: Vec<Container>,
    container: Option<Container>
}

impl ContainerWrapper {
    pub fn new(containers: Vec<Container>) -> ContainerWrapper {
        ContainerWrapper {
            containers,
            container: None
        }
    }

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
}