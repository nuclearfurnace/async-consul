use std::collections::HashMap;

use serde::Deserialize;

use crate::health::HealthCheckDefinition;

#[derive(Deserialize, Debug)]
pub enum AgentServiceKind {
    #[serde(rename = "")]
    Default,
    #[serde(rename = "connect-proxy")]
    ConnectProxy,
    #[serde(rename = "mesh-gateway")]
    MeshGateway,
    #[serde(rename = "terminating-gateway")]
    TerminatingGateway,
    #[serde(rename = "ingress-gateway")]
    IngressGateway,
}

#[derive(Deserialize, Debug)]
pub struct AgentWeights {
    #[serde(rename = "Passing")]
    pub passing: u64,
    #[serde(rename = "Warning")]
    pub warning: u64,
}

#[derive(Deserialize, Debug)]
pub struct AgentCheck {
    #[serde(rename = "Node")]
    pub node: String,
    #[serde(rename = "CheckID")]
    pub check_id: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Status")]
    pub status: String,
    #[serde(rename = "Notes")]
    pub notes: String,
    #[serde(rename = "Output")]
    pub output: String,
    #[serde(rename = "ServiceID")]
    pub service_id: String,
    #[serde(rename = "ServiceName")]
    pub service_name: String,
    #[serde(rename = "Type")]
    pub check_type: String,
    #[serde(rename = "Namespace")]
    pub namespace: Option<String>,
    #[serde(rename = "Definition")]
    pub definition: HealthCheckDefinition,
}

#[derive(Deserialize, Debug)]
pub struct AgentService {
    #[serde(rename = "Kind")]
    pub kind: AgentServiceKind,
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "Service")]
    pub service: String,
    #[serde(rename = "Tags")]
    pub tags: Vec<String>,
    #[serde(rename = "Meta")]
    pub meta: HashMap<String, String>,
    #[serde(rename = "Port")]
    pub port: u16,
    #[serde(rename = "Address")]
    pub address: String,
    #[serde(rename = "TaggedAddresses")]
    pub tagged_addresses: HashMap<String, String>,
    #[serde(rename = "Weights")]
    pub weights: AgentWeights,
    #[serde(rename = "EnableTagOverride")]
    pub enable_tag_override: bool,
    #[serde(rename = "CreateIndex")]
    pub create_index: u64,
    #[serde(rename = "ModifyIndex")]
    pub modify_index: u64,
    #[serde(rename = "ContentHash")]
    pub content_hash: String,
    // TODO: implement this stuff, I'm too lazy to do it right now.
    //#[serde(rename = "Proxy")]
    //pub proxy: AgentServiceConnectProxyConfig,
    //#[serde(rename = "Connect")]
    //pub connect: AgentServiceConnect,
    #[serde(rename = "Namespace")]
    pub namespace: Option<String>,
}
