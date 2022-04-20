use kube::api::{
    Api,
    ListParams
};
use k8s_openapi::api::core::v1::Pod;
use super::*;
use crate::error::KubeErr;

#[derive(Debug)]
pub struct PodsList {
    pods: Vec<Pod>
}

impl PodsList {
    /// Create a new PodsList structure
    ///     
    /// # Arguments
    /// * `context` - Option<String>
    /// * `ns` - &str
    pub async fn new(context: Option<String>, ns: &str) -> Result<PodsList, KubeErr> {
        let client = authenticate_with_cluster(context).await?;
        let pod_api: Api<Pod> = Api::namespaced(client, &ns);
        let list = pod_api.list(&ListParams::default()).await?;

        let pods = PodsList {
            pods: list.items
        };

        Ok(pods)
    }

    /// Get a list of pod name for the list of pods that has been founded
    /// 
    /// # Arguments
    /// * `&self` - Self
    pub fn get_pod_name_list(&self) -> Vec<String> {
        self.pods.to_owned()
            .into_iter()
            .map(|p| p.metadata.name)
            .filter(|p| p.is_some())
            .map(|p| p.unwrap())
            .collect::<Vec<_>>()
    }
}