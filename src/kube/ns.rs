use k8s_openapi::api::core::v1::Namespace;
use kube::{Api, api::ListParams};

use super::*;

/// Get a list of namespace's name by using the kube-rs api
///
/// # Arguments
/// * `context` - &Option<String>
pub async fn get_namespace_list(context: &Option<String>) -> Result<Vec<String>, KubeErr> {
    let client = authenticate_with_cluster(context).await?;
    let namespaces: Api<Namespace> = Api::all(client);
    let mut names = Vec::new();

    for nss in namespaces.list(&ListParams::default()).await {
        for item in nss.items {
            if let Some(name) = item.metadata.name {
                names.push(name);
            }
        }
    }

    Ok(names)
}
