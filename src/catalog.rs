use std::collections::HashMap;
use std::sync::Arc;

use async_stream::try_stream;
use futures::stream::Stream;
use serde::Deserialize;

use crate::common::{Blocking, QueryMetadata, QueryOptions};
use crate::errors::Error;
use crate::health::HealthCheck;
use crate::http_client::HttpClient;

#[derive(Deserialize, Debug)]
pub struct Weights {
    #[serde(rename = "Passing")]
    pub passing: u64,
    #[serde(rename = "Warning")]
    pub warning: u64,
}

#[derive(Deserialize, Debug)]
pub struct CatalogNode {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "Node")]
    pub node: String,
    #[serde(rename = "Address")]
    pub address: String,
    #[serde(rename = "Datacenter")]
    pub datacenter: String,
    #[serde(rename = "TaggedAddresses")]
    pub tagged_addresses: HashMap<String, String>,
    #[serde(rename = "Meta")]
    pub meta: HashMap<String, String>,
    #[serde(rename = "CreateIndex")]
    pub create_index: u64,
    #[serde(rename = "ModifyIndex")]
    pub modify_index: u64,
}

#[derive(Deserialize, Debug)]
pub struct ServiceAddress {
    #[serde(rename = "Address")]
    pub address: String,
    #[serde(rename = "Port")]
    pub port: u16,
}

#[derive(Deserialize, Debug)]
pub struct CatalogServiceNode {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "Node")]
    pub node: String,
    #[serde(rename = "Address")]
    pub address: String,
    #[serde(rename = "Datacenter")]
    pub datacenter: String,
    #[serde(rename = "TaggedAddresses")]
    pub tagged_addresses: HashMap<String, String>,
    #[serde(rename = "NodeMeta")]
    pub node_meta: HashMap<String, String>,
    #[serde(rename = "ServiceID")]
    pub service_id: String,
    #[serde(rename = "ServiceName")]
    pub service_name: String,
    #[serde(rename = "ServiceAddress")]
    pub service_address: String,
    #[serde(rename = "ServiceTaggedAddresses")]
    pub service_tagged_addresses: Option<HashMap<String, ServiceAddress>>,
    #[serde(rename = "ServiceTags")]
    pub service_tags: Vec<String>,
    #[serde(rename = "ServiceMeta")]
    pub service_meta: HashMap<String, String>,
    #[serde(rename = "ServicePort")]
    pub service_port: u16,
    #[serde(rename = "ServiceWeights")]
    pub service_weights: Option<Weights>,
    #[serde(rename = "ServiceEnableTagOverride")]
    pub service_enable_tag_override: bool,
    // TODO: eventually add support for this
    // #[serde(rename = "ServiceProxy")]
    // pub service_proxy: AgentServiceConnectProxyConfig,
    // #[serde(rename = "ServiceConnect")]
    // pub service_connect: AgentServiceConnect,
    #[serde(rename = "CreateIndex")]
    pub create_index: u64,
    #[serde(rename = "Checks")]
    pub checks: Option<Vec<HealthCheck>>,
    #[serde(rename = "ModifyIndex")]
    pub modify_index: u64,
    #[serde(rename = "Namespace")]
    pub namespace: Option<String>,
}

/// Catalog operations.
///
/// This type can be used to interact with the "Catalog" portion of the Consul API.
#[derive(Clone, Debug)]
pub struct Catalog {
    http_client: Arc<HttpClient>,
}

impl Catalog {
    /// Creates a new [`Catalog`].
    pub(crate) fn new(http_client: Arc<HttpClient>) -> Catalog {
        Catalog { http_client }
    }

    /// Gets the nodes running the specified service.
    pub async fn get_service_nodes(
        &self,
        service: &str,
        options: Option<QueryOptions>,
    ) -> Result<(Vec<CatalogServiceNode>, QueryMetadata), Error> {
        let request = self.http_client.build_request(
            "get",
            &["v1", "catalog", "service", service],
            options.as_ref(),
            (),
        )?;
        let response = self
            .http_client
            .run_request(request, options.as_ref())
            .await?;
        let (parsed, meta) = self.http_client.parse_query_response(response).await?;
        Ok((parsed, meta))
    }

    /// Gets a stream of changes in nodes running the specified service.
    ///
    /// Each item in the response stream represents all nodes running in the service after a change
    /// to the service has occurred.  The stream will terminate if any error is hit during the
    /// background requests made to Consul.
    pub fn watch_service_nodes(
        &self,
        service: &str,
        options: Option<QueryOptions>,
    ) -> impl Stream<Item = Result<(Vec<CatalogServiceNode>, QueryMetadata), Error>> {
        let service = service.to_string();
        let http_client = self.http_client.clone();
        let mut options = options.or_else(|| Some(QueryOptions::default()));

        let mut blocking: Option<Blocking> = None;

        try_stream! {
            loop {
                // Override the blocking settings before every request.
                let options = options.as_mut().map(|opts| { opts.blocking = blocking.take(); &*opts });

                let request = http_client.build_request("GET", &["v1", "catalog", "service", &service], options, ())?;
                let response = http_client.run_request(request, options).await?;
                let (parsed, meta) = http_client.parse_query_response(response).await?;

                // Override our blocking configuration based on the metadata from this response.
                blocking = meta.as_blocking();

                yield (parsed, meta);
            }
        }
    }
}
