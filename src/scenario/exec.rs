use inquire::Select;

use crate::kube::list::PodsList;
use crate::error::KubeErr;

/// Trigger Scenario
///     List a set of pod and with the provided context and exec into it
/// 
/// # Arguments
/// * `context` - Option<String>
pub async fn trigger_scenario(context: Option<String>, ns: String) -> Result<(), KubeErr> {
    let pod_list = PodsList::new(context, &ns).await?;
    let pod_list_name = pod_list.get_pod_name_list();

    if pod_list_name.is_empty() {
        return Err(KubeErr::EmptyPods(ns))
    }

    // prompt the selection of the pod for the user
    let selected_pods = Select::new("Select pod", pod_list_name)
        .prompt()?;

    // exec in pod

    Ok(())
}