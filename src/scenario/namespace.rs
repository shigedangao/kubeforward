use inquire::Select;
use crate::{kube::ns, error::KubeErr};

// Constant
const SELECT_NS: &str = "Select which namespace you want to use";

/// Run the scenario to get a list of namespaces
///
/// # Arguments
/// * `context` - &Option<String>
pub async fn trigger_scenario(context: &Option<String>) -> Result<String, KubeErr> {
    let namespaces = ns::get_namespace_list(context).await?;
    let selected_ns = Select::new(SELECT_NS, namespaces)
        .prompt()?;

    Ok(selected_ns)
}
