use inquire::{Select, Text};
use crate::kube::{
    list::PodsList,
};
use crate::error::KubeErr;

// constant
const POD_SELECT_PROMPT: &str = "Select which pod you want to connect with";
const CONTAINER_SELECT_PROMPT: &str = "Select which container you want to port forward";
const SELECTED_PORT: &str = "Select which port to expose";
const USER_PORT: &str = "Input which port you want to use";

/// Trigger Scenario
///     List a set of pod and with the provided context and exec into it
/// 
/// # Arguments
/// * `context` - Option<String>
/// * `ns` - String
pub async fn trigger_scenario(context: Option<String>, ns: String) -> Result<(), KubeErr> {
    let mut pod_list = PodsList::new(context, &ns).await?;
    let pod_list_name = pod_list.get_pod_name_list();

    if pod_list_name.is_empty() {
        return Err(KubeErr::EmptyPods(ns))
    }

    // prompt the selection of the pod for the user
    let selected_pod = Select::new(POD_SELECT_PROMPT, pod_list_name)
        .prompt()?;

    // set the selected pod on the pod_list
    pod_list.set_selected_pod(selected_pod);

    // get a list of container name
    let containers_name = pod_list.list_containers();
    if containers_name.is_none() {
        return Err(KubeErr::EmptyContainers)
    }

    let containers_name = containers_name.unwrap();
    // propose a set of command to the user
    let selected_container = Select::new(CONTAINER_SELECT_PROMPT, containers_name)
        .prompt()?
        .to_owned();

    // get a list of port for the selected container
    let ports = pod_list.get_port_for_container(selected_container);
    if ports.is_none() {
        return Err(KubeErr::EmptyPorts);
    }

    let ports = ports.unwrap();
    let selected_port = Select::new(SELECTED_PORT, ports).prompt()?;
    let user_port = Text::new(USER_PORT).prompt()?;

    pod_list.expose_pod(selected_port, &user_port).await
}
