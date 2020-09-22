use hyper::client::{Client as HyperClient, HttpConnector};
use hyper::header::{HeaderName, HeaderValue};
use hyper::{Body, Request, Response};
use hyper_tls::HttpsConnector;
use serde::{de::DeserializeOwned, Serialize};
use tokio::time::timeout;
use url::Url;

use std::borrow::Cow;
use std::collections::HashMap;

use crate::common::{AsTimeout, CollectQueryParameters, CollectRequestHeaders, QueryMetadata};
use crate::errors::{Error, ResponseError};

#[derive(Clone, Debug)]
pub(crate) struct HttpClient {
    client: HyperClient<HttpsConnector<HttpConnector>, Body>,
    base_uri: Url,
}

impl HttpClient {
    /// Creates a new [`HttpClient`].
    pub fn new(base_uri: Url) -> HttpClient {
        let connector = HttpsConnector::new();
        let client = HyperClient::builder().build(connector);

        HttpClient { client, base_uri }
    }

    pub fn build_request<I, O, B>(
        &self,
        method: &str,
        url_parts: I,
        options: Option<O>,
        body: B,
    ) -> Result<Request<Body>, Error>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
        O: CollectQueryParameters + CollectRequestHeaders,
        B: Serialize,
    {
        let mut new_path = self.base_uri.clone();
        new_path
            .path_segments_mut()
            .expect("URL not in suitable format for extending")
            .extend(url_parts);

        let pairs = CollectQueryParameters::as_pairs(&options);
        if !pairs.is_empty() {
            // we need to add query params but also make sure that we handle overrides
            // correctly so that per-operation things can be tweaked.  since query params
            // aren't represented as a map, but instead of a single value that is append-only,
            // we'll have to figure out a good way to handle this.  probably just take existing,
            // make a hashmap out of it, upsert our per-operation pairs, and then reassemble?
            let parse = new_path.query_pairs();
            let mut existing = parse
                .into_owned()
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect::<HashMap<Cow<'static, str>, Cow<'static, str>>>();

            for (k, v) in pairs.into_iter() {
                existing.insert(k.into(), v);
            }

            new_path.query_pairs_mut().clear().extend_pairs(existing);
        }

        let headers = CollectRequestHeaders::as_pairs(&options);
        let serialized = serde_json::to_vec(&body).map_err(Error::InvalidRequestBody)?;
        let body = Body::from(serialized);

        let mut req = Request::builder()
            .method(method)
            .uri(new_path.to_string())
            .body(body)
            .map_err(Error::InvalidRequest)?;

        let headers = headers
            .into_iter()
            .map(|(k, v)| {
                let name = HeaderName::from_bytes(k.as_bytes())
                    .expect("should be impossible for us to use an invalid header name");
                let value = HeaderValue::from_bytes(v.as_ref().as_bytes())
                    .expect("should be impossible for us to use an invalid header value");
                (name, value)
            })
            .collect::<Vec<_>>();
        req.headers_mut().extend(headers);
        Ok(req)
    }

    pub async fn run_request<O>(
        &self,
        request: Request<Body>,
        options: Option<O>,
    ) -> Result<Response<Body>, Error>
    where
        O: AsTimeout,
    {
        let timeout_dur = options.as_timeout();

        if let Some(dur) = timeout_dur {
            let result = timeout(dur, self.client.request(request)).await?;
            Ok(result?)
        } else {
            Ok(self.client.request(request).await?)
        }
    }

    pub async fn parse_query_response<T>(
        &self,
        response: Response<Body>,
    ) -> Result<(T, QueryMetadata), ResponseError>
    where
        T: DeserializeOwned,
    {
        let status = response.status();
        if !status.is_success() {
            return Err(ResponseError::UnexpectedStatus(status));
        }

        let meta = QueryMetadata::from_headers(response.headers())?;

        let body = response.into_body();
        let data = hyper::body::to_bytes(body).await?;
        let parsed: T = serde_json::from_slice(&data)?;
        Ok((parsed, meta))
    }
}
