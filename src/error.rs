use std::num::ParseIntError;
use kube::{
    config::KubeconfigError,
    error::Error as KError
};
use inquire::error::InquireError;

#[derive(Debug)]
pub enum KubeErr {
    Kubeconfig(String),
    Kube(String),
    Prompt(String),
    EmptyPods(String),
    EmptyContainers,
    EmptyPorts,
    WrongPort,
    Network(String)
}

impl std::error::Error for KubeErr {}

impl std::fmt::Display for KubeErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KubeErr::Kubeconfig(msg) => write!(f, "Error while reading Kubeconfig: {msg}"),
            KubeErr::Prompt(msg) => write!(f, "Something unexpected happened with the prompt: {msg}"),
            KubeErr::Kube(msg) => write!(f, "Error while querying with kubernetes {msg}"),
            KubeErr::EmptyPods(ns) => write!(f, "Could not found pod in the selected context and namespace: {ns}"),
            KubeErr::EmptyContainers => write!(f, "The pod does not contain any container !"),
            KubeErr::EmptyPorts => write!(f, "The selected container does not have any exposed ports"),
            KubeErr::WrongPort => write!(f, "The input port is not a numeric value"),
            KubeErr::Network(msg) => write!(f, "Error while sending / receiving data with pod {msg}")
        }
    }
}

impl From<KubeconfigError> for KubeErr {
    fn from(err: KubeconfigError) -> Self {
        KubeErr::Kubeconfig(err.to_string())
    }
}

impl From<KError> for KubeErr {
    fn from(err: KError) -> Self {
        KubeErr::Kube(err.to_string())
    }
}

impl From<InquireError> for KubeErr {
    fn from(err: InquireError) -> Self {
        KubeErr::Prompt(err.to_string())
    }
}

impl From<ParseIntError> for KubeErr {
    fn from(_: ParseIntError) -> Self {
        KubeErr::WrongPort
    }
}

impl From<hyper::Error> for KubeErr {
    fn from(err: hyper::Error) -> Self {
        KubeErr::Network(err.to_string())
    }
}