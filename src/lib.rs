//! A Tokio-based asynchronous client for the Consul API.
#![deny(missing_debug_implementations)]
use std::sync::Arc;
use url::Url;

mod agent;
mod catalog;
pub mod common;
mod errors;
mod health;
mod http_client;

pub use self::catalog::{Catalog, CatalogServiceNode};
pub use self::errors::*;
use self::http_client::HttpClient;

/// High-level client for interacting with the Consul API.
///
/// Exposes subclients for various areas of the API i.e. Catalog vs Agent.  These subclients expose
/// the relevant operations for those APIs, and both [`Client`] and the aforementioned subclients
/// can be cloned or otherwise used in an asynchronous fashion without issue.
#[derive(Debug, Clone)]
pub struct Client {
    http_client: Arc<HttpClient>,
}

impl Client {
    /// Create a new [`Client`].
    pub fn new(base_uri: &str) -> Result<Client, Error> {
        let base_uri = Url::parse(base_uri)?;
        let http_client = HttpClient::new(base_uri);

        Ok(Client {
            http_client: Arc::new(http_client),
        })
    }

    /// Gets a [`Catalog`] object for working with the catalog API.
    pub fn catalog(&self) -> Catalog {
        Catalog::new(self.http_client.clone())
    }
}
