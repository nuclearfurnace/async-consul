//! Common types utilized throughout the crate.
use std::borrow::Cow;
use std::collections::HashMap;
use std::time::Duration;

use hyper::header::{HeaderMap, HeaderValue};

use crate::errors::ResponseError;

/// The consistency of a given operation.
///
/// Not all endpoints support adjusting consistency.  Users can refer to the Consul API documentation
/// on [consistency](https://www.consul.io/api-docs/features/consistency) to learn more.
#[derive(Clone, Debug, PartialEq)]
pub enum Consistency {
    /// Forces fully consistency, which is more expensive but avoids stale reads.
    Consistent,
    /// Allows any non-leader Consul server to service this read, which trades lower latency and
    /// higher throughput for staleness.
    Stale,
}

/// The blocking behavior of a given operation.
///
/// Not all endpoints support blocking.  Users can refer to the Consul API documentation on
/// [blocking](https://www.consul.io/api-docs/features/blocking) to learn more.
#[derive(Clone, Debug, PartialEq)]
pub enum Blocking {
    /// Block based on index.
    ///
    /// This mode is related to the `X-Consul-Index` header returned by Consul.
    Index(u64),
    /// Block based on hash.
    ///
    /// This mode is related to the `X-Consul-ContentHash` header returned by Consul.
    Hash(String),
}

/// An object that can be collected as a set of query parameters for a given Consul request.
///
/// This provides an interface such that configuration types can be easily queried to generate
/// any query parameters they would want to add to a request.
pub(crate) trait CollectQueryParameters {
    fn as_pairs(&self) -> Vec<(&'static str, Cow<'static, str>)>;
}

/// An object that can be collected as a set of headers for a given Consul request.
///
/// This provides an interface such that configuration types can be easily queried to generate
/// any headers they would want to add to a request.
pub(crate) trait CollectRequestHeaders {
    fn as_pairs(&self) -> Vec<(&'static str, Cow<'static, str>)>;
}

/// An object that can express a timeout.
///
/// Commonly used to derive a timeout value from a configuration object where there may be a single
/// timeout value, or complex logic used to derive the timeout value.  This allows callers to
/// generically pass options when building requests so a timeout can be retrieved.
pub(crate) trait AsTimeout {
    fn as_timeout(&self) -> Option<Duration>;
}

impl<'a, T> CollectQueryParameters for &'a T
where
    T: CollectQueryParameters,
{
    fn as_pairs(&self) -> Vec<(&'static str, Cow<'static, str>)> {
        CollectQueryParameters::as_pairs(*self)
    }
}

impl<'a, T> CollectRequestHeaders for &'a T
where
    T: CollectRequestHeaders + 'a,
{
    fn as_pairs(&self) -> Vec<(&'static str, Cow<'static, str>)> {
        CollectRequestHeaders::as_pairs(*self)
    }
}

impl<'a, T> AsTimeout for &'a T
where
    T: AsTimeout + 'a,
{
    fn as_timeout(&self) -> Option<Duration> {
        AsTimeout::as_timeout(*self)
    }
}

impl<T> CollectQueryParameters for Option<T>
where
    T: CollectQueryParameters,
{
    fn as_pairs(&self) -> Vec<(&'static str, Cow<'static, str>)> {
        match self {
            Some(inner) => CollectQueryParameters::as_pairs(inner),
            None => Vec::new(),
        }
    }
}

impl<T> CollectRequestHeaders for Option<T>
where
    T: CollectRequestHeaders,
{
    fn as_pairs(&self) -> Vec<(&'static str, Cow<'static, str>)> {
        match self {
            Some(inner) => CollectRequestHeaders::as_pairs(inner),
            None => Vec::new(),
        }
    }
}

impl<T> AsTimeout for Option<T>
where
    T: AsTimeout,
{
    fn as_timeout(&self) -> Option<Duration> {
        match self {
            Some(inner) => AsTimeout::as_timeout(inner),
            None => None,
        }
    }
}

/// Options specific to write operations.
#[derive(Clone, Debug, Default)]
pub struct WriteOptions {
    /// Namespace to execute this operation against.
    ///
    /// NOTE: Namespaces are available only in Consul Enterprise.
    pub namespace: Option<String>,
    /// Datacenter to execute this operation against.
    ///
    /// By default, operations will execute against whichever datacenter is configured at the
    /// endpoint you connect to, whether you're connected directly to a Consul cluster or to an
    /// agent.
    pub datacenter: Option<String>,
    /// Token to use for this operation.
    ///
    /// By default, operations will use the agent's default token if talking to an agent, but will
    /// not use a token if talking directly to a Consul cluster.
    pub token: Option<String>,
    /// Used in keyring operations to force responses to be relayed back to the send through N other
    /// random nodes.  Most be a value from 0 to 5 (inclusive).
    pub relay_factor: Option<u8>,
    /// Timeout for this operation overall.
    pub timeout: Option<Duration>,
}

impl CollectQueryParameters for WriteOptions {
    fn as_pairs(&self) -> Vec<(&'static str, Cow<'static, str>)> {
        let mut pairs = Vec::new();

        if let Some(namespace) = self.namespace.as_ref() {
            pairs.push(("ns", namespace.clone().into()));
        }

        if let Some(datacenter) = self.datacenter.as_ref() {
            pairs.push(("dc", datacenter.clone().into()));
        }

        if let Some(relay_factor) = self.relay_factor.as_ref() {
            pairs.push(("relay-factor", relay_factor.to_string().into()));
        }

        pairs
    }
}

impl CollectRequestHeaders for WriteOptions {
    fn as_pairs(&self) -> Vec<(&'static str, Cow<'static, str>)> {
        let mut pairs = Vec::new();

        if let Some(token) = self.token.as_ref() {
            pairs.push(("X-Consul-Token", token.clone().into()));
        }

        pairs
    }
}

impl AsTimeout for WriteOptions {
    fn as_timeout(&self) -> Option<Duration> {
        self.timeout.clone()
    }
}

/// Options specific to query operations.
#[derive(Clone, Debug, Default)]
pub struct QueryOptions {
    /// Namespace to execute this operation against.
    ///
    /// NOTE: Namespaces are available only in Consul Enterprise.
    pub namespace: Option<String>,
    /// Datacenter to execute this operation against.
    ///
    /// By default, operations will execute against whichever datacenter is configured at the
    /// endpoint you connect to, whether you're connected directly to a Consul cluster or to an
    /// agent.
    pub datacenter: Option<String>,
    /// Token to use for this operation.
    ///
    /// By default, operations will use the agent's default token if talking to an agent, but will
    /// not use a token if talking directly to a Consul cluster.
    pub token: Option<String>,
    /// Consistency level for this operation.
    pub consistency: Option<Consistency>,
    /// Blocking configuration for this operation.
    pub blocking: Option<Blocking>,
    /// Blocking timeout.
    ///
    /// Only used if `blocking` is set.  This should be set lower than the overall `timeout` in
    /// order to ensure that Consul hasa chance to complete the request and send back results.
    ///
    /// If `blocking` is configured, and `timeout` is set, but `blocking_timeout` is not set, then a
    /// timeout will be applied to the request overall, and callers may receive a timeout error
    /// rather than a successful response from the server.
    pub blocking_timeout: Option<Duration>,
    /// Asks the agent to cache results locally.
    ///
    /// Users can refer to the Consul API documentation,
    /// [https://www.consul.io/api/features/caching.html], for more information on caching.
    pub use_cache: bool,
    /// Controls how old of a cached response this operation will accept from the agent.
    ///
    /// If there is a cached response older than this, the agent treats it as a miss and will
    /// attempt to fetch the latest value.  If that fetch fails, the error is returned.
    ///
    /// For clients that wish to utilize a stale value in the case of an underlying error, they can
    /// set [`cache_stale_if_error`] to specify how old of a value they are willing to accept.
    ///
    /// This value is ignored if the endpoint supports background refresh caching.
    pub cache_max_age: Option<Duration>,
    /// Controls how old of a cached response this operation will accept from the agent, but only if
    /// attempt to refresh the value has failed.
    ///
    /// This value is ignored if the endpoint supports background refresh caching.
    pub cache_stale_if_error: Option<Duration>,
    /// Sorting based on network latency.
    ///
    /// Should be the name of a node, which will sort of the results in order of lowest to highest
    /// latency from the given node to each node in the results.
    ///
    /// Optionally, `_agent` can be specified to sort the results based on their latency to the agent.
    pub near: Option<String>,
    /// Filter results to nodes that match the specified node metadata values.
    pub node_meta: Option<HashMap<String, String>>,
    /// Filter results to nodes that have a tag matching the specified tag.
    pub tag: Option<String>,
    /// Filtering reduces network load by filtering results on the server prior to responding.
    ///
    /// Users can refer to the Consul API documentation,
    /// [https://www.consul.io/api-docs/features/filtering], to read about filtering.
    pub filtering: Option<String>,
    /// Used in keyring operations to force responses to be relayed back to the send through N other
    /// random nodes.  Most be a value from 0 to 5 (inclusive).
    pub relay_factor: Option<u8>,
    /// Used in keyring list operations to force the keyring query to only hit local servers i.e. no
    /// WAN traffic.
    pub local_only: bool,
    /// Only include Connect-capable services/nodes in the response.
    ///
    /// Currently only affects prepared query operations.
    pub connect: bool,
    /// Timeout for this operation overall.
    pub timeout: Option<Duration>,
}

impl CollectQueryParameters for QueryOptions {
    fn as_pairs(&self) -> Vec<(&'static str, Cow<'static, str>)> {
        let mut pairs = Vec::new();

        if let Some(namespace) = self.namespace.as_ref() {
            pairs.push(("ns", namespace.clone().into()));
        }

        if let Some(datacenter) = self.datacenter.as_ref() {
            pairs.push(("dc", datacenter.clone().into()));
        }

        if let Some(consistency) = self.consistency.as_ref() {
            match consistency {
                Consistency::Consistent => pairs.push(("consistent", "1".into())),
                Consistency::Stale => pairs.push(("stale", "1".into())),
            }
        }

        if let Some(blocking) = self.blocking.as_ref() {
            match blocking {
                Blocking::Index(idx) => {
                    let idxs = idx.to_string();
                    pairs.push(("index", idxs.into()));
                }
                Blocking::Hash(hash) => pairs.push(("hash", hash.clone().into())),
            }

            // Apply the blocking timeout, if configured.
            if let Some(timeout) = self.blocking_timeout {
                let durs = format!("{}ms", timeout.as_millis());
                pairs.push(("wait", durs.into()));
            }
        }

        if let Some(near) = self.near.as_ref() {
            pairs.push(("near", near.clone().into()));
        }

        if let Some(nodemeta) = self.node_meta.as_ref() {
            for (k, v) in nodemeta.iter() {
                pairs.push(("node-meta[]", format!("{}:{}", k, v).into()))
            }
        }

        if let Some(tag) = self.tag.as_ref() {
            pairs.push(("tag", tag.clone().into()));
        }

        if let Some(filtering) = self.filtering.as_ref() {
            pairs.push(("filter", filtering.clone().into()));
        }

        if let Some(relay_factor) = self.relay_factor.as_ref() {
            pairs.push(("relay-factor", relay_factor.to_string().into()));
        }

        if self.local_only {
            pairs.push(("local-only", "true".into()));
        }

        if self.connect {
            pairs.push(("connect", "true".into()));
        }

        // Can only send caching headers if enabled _and_ we aren't requesting fully consistency reads.
        if self.use_cache
            && self
                .consistency
                .as_ref()
                .map(|val| val != &Consistency::Consistent)
                .unwrap_or(true)
        {
            pairs.push(("cached", "1".into()));
        }

        pairs
    }
}

impl CollectRequestHeaders for QueryOptions {
    fn as_pairs(&self) -> Vec<(&'static str, Cow<'static, str>)> {
        let mut pairs = Vec::new();

        if let Some(token) = self.token.as_ref() {
            pairs.push(("X-Consul-Token", token.clone().into()));
        }

        // Can only send caching headers if enabled _and_ we aren't requesting fully consistency reads.
        if self.use_cache
            && self
                .consistency
                .as_ref()
                .map(|val| val != &Consistency::Consistent)
                .unwrap_or(true)
        {
            let mut parts = Vec::new();

            if let Some(max_age) = self.cache_max_age.as_ref() {
                if max_age.as_secs() > 0 {
                    parts.push(format!("max-age={}", max_age.as_secs()));
                }
            }

            if let Some(max_stale) = self.cache_stale_if_error {
                if max_stale.as_secs() > 0 {
                    parts.push(format!("stale-if-error={}", max_stale.as_secs()));
                }
            }

            if !parts.is_empty() {
                let val = parts.join(", ");
                pairs.push(("Cache-Control", val.into()));
            }
        }

        pairs
    }
}

impl AsTimeout for QueryOptions {
    fn as_timeout(&self) -> Option<Duration> {
        self.timeout
    }
}

/// Metadata about the request returned from a query operation.
#[derive(Debug, Default)]
pub struct QueryMetadata {
    /// The Consul index for the data in this response.
    ///
    /// Can be used to issue a blocking query.
    pub last_index: Option<u64>,
    /// The Consul content hash for the data in this response.
    ///
    /// Can be used to issue a blocking query, but is only available for endpoints that support
    /// hash-based blocking.
    pub last_content_hash: Option<String>,
    /// Whether or not the cluster has a known leader.
    pub known_leader: bool,
    /// Amount of time since the server which serviced this request has contacted the leader.
    pub last_contact: Duration,
    /// Whether or not address translation is enabled for HTTP responses on the agent being queried.
    pub addr_translate_enabled: bool,
    /// Whether or not this response was served from the agent's local cache.
    pub cache_hit: bool,
    /// The age of the cache value, if served from cache.
    pub cache_age: Option<Duration>,
}

impl QueryMetadata {
    pub(crate) fn from_headers(
        headers: &HeaderMap<HeaderValue>,
    ) -> Result<QueryMetadata, ResponseError> {
        let mut meta = QueryMetadata::default();
        let mut errors = Vec::new();

        if let Some(index_raw) = headers.get("X-Consul-Index") {
            match index_raw.to_str() {
                Ok(index_str) => match index_str.parse::<u64>() {
                    Ok(index) => meta.last_index = Some(index),
                    Err(_) => errors.push("X-Consul-Index"),
                },
                Err(_) => errors.push("X-Consul-Index"),
            }
        }

        if let Some(hash_raw) = headers.get("X-Consul-ContentHash") {
            match hash_raw.to_str() {
                Ok(hash_str) => meta.last_content_hash = Some(hash_str.to_string()),
                Err(_) => errors.push("X-Consul-ContentHash"),
            }
        }

        if let Some(known_raw) = headers.get("X-Consul-KnownLeader") {
            match known_raw.to_str() {
                Ok(known_str) => {
                    if known_str == "true" {
                        meta.known_leader = true;
                    }
                }
                Err(_) => errors.push("X-Consul-KnownLeader"),
            }
        }

        if let Some(translate_raw) = headers.get("X-Consul-Translate-Addresses") {
            match translate_raw.to_str() {
                Ok(translate_str) => {
                    if translate_str == "true" {
                        meta.addr_translate_enabled = true;
                    }
                }
                Err(_) => errors.push("X-Consul-Translate-Addresses"),
            }
        }

        if let Some(cache_raw) = headers.get("X-Cache") {
            match cache_raw.to_str() {
                Ok(cache_str) => {
                    if cache_str.to_lowercase() == "hit" {
                        meta.cache_hit = true;
                    }
                }
                Err(_) => errors.push("X-Cache"),
            }
        }

        if let Some(age_raw) = headers.get("Age") {
            match age_raw.to_str() {
                Ok(age_str) => match age_str.parse::<u64>() {
                    Ok(age) => meta.cache_age = Some(Duration::from_secs(age)),
                    Err(_) => errors.push("Age"),
                },
                Err(_) => errors.push("Age"),
            }
        }

        if !errors.is_empty() {
            Err(ResponseError::InvalidHeaders(errors))
        } else {
            Ok(meta)
        }
    }

    pub(crate) fn as_blocking(&self) -> Option<Blocking> {
        if let Some(last_content_hash) = &self.last_content_hash {
            return Some(Blocking::Hash(last_content_hash.clone()));
        }

        if let Some(last_index) = &self.last_index {
            return Some(Blocking::Index(last_index.clone()));
        }

        None
    }
}
