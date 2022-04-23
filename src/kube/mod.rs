use kube::{
    Config,
    config::KubeConfigOptions,
    Client
};
use crate::error::KubeErr;

pub mod pod;
mod container;
mod listener;

/// Authenticate with the Kubernetes cluster based on the provided context
/// 
/// # Arguments
/// * `context` - Option<String>
pub async fn authenticate_with_cluster(context: &Option<String>) -> Result<Client, KubeErr> {
    // see if that work otherwise try with the whole struct
    let mut options = KubeConfigOptions::default();
    if let Some(ctx) = context {
        options.context = Some(ctx.to_owned());
    }

    let config = Config::from_kubeconfig(&options).await?;
    let client = Client::try_from(config)?;
    
    Ok(client)
}