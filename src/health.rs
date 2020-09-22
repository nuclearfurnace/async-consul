use std::collections::HashMap;
use std::time::Duration;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct HealthCheck {
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
    #[serde(rename = "ServiceTags")]
    pub service_tags: Vec<String>,
    #[serde(rename = "Type")]
    pub check_type: String,
    #[serde(rename = "Namespace")]
    pub namespace: Option<String>,
    #[serde(rename = "Definition")]
    pub definition: HealthCheckDefinition,
    #[serde(rename = "CreateIndex")]
    pub create_index: u64,
    #[serde(rename = "ModifyIndex")]
    pub modify_index: u64,
}

#[derive(Deserialize, Debug)]
pub struct HealthCheckDefinition {
    #[serde(rename = "HTTP")]
    pub http: String,
    #[serde(rename = "Header")]
    pub header: HashMap<String, String>,
    #[serde(rename = "Method")]
    pub method: String,
    #[serde(rename = "Body")]
    pub body: String,
    #[serde(rename = "TLSSkipVerify")]
    pub tls_skip_verify: bool,
    #[serde(rename = "TCP")]
    pub tcp: String,
    #[serde(rename = "Interval")]
    pub interval: Duration,
    #[serde(rename = "Timeout")]
    pub timeout: Duration,
    #[serde(rename = "DeregisterCriticalServiceAfterDuration")]
    pub deregister_critical_svc_after: Duration,
}
