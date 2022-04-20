use kube::config::Kubeconfig;
use crate::error::KubeErr;
use inquire::Select;

/// Trigger Scenario
///     Run the scenario to select which context the user want to use
pub fn trigger_scenario() -> Result<String, KubeErr> {
    let config = Kubeconfig::read()?;
    let context_name: Vec<_> = config.contexts
        .into_iter()
        .map(|c| c.name)
        .collect();

    // prompt the selection of the context for the user
    let selected_context = Select::new("Kubernetes context", context_name)
        .prompt()?;

    Ok(selected_context)
}