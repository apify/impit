use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use hickory_proto::rr::rdata::svcb::SvcParamValue;
use hickory_proto::rr::{Name, RData, RecordType};
use hickory_resolver::TokioResolver;
use log::debug;

/// A struct encapsulating the components required to make HTTP/3 requests.
pub struct H3Engine {
    /// The DNS resolver used to query HTTPS records for h3 discovery.
    resolver: Option<TokioResolver>,
    /// A map of hosts that support HTTP/3.
    ///
    /// This is populated by the DNS queries and manual calls to `set_h3_support` (based on the `Alt-Svc` header).
    /// Implicitly used as a cache for the DNS queries.
    h3_alt_svc: Arc<RwLock<HashMap<String, bool>>>,
}

impl H3Engine {
    pub async fn init() -> Self {
        let resolver = match TokioResolver::builder_tokio().and_then(|builder| builder.build()) {
            Ok(resolver) => Some(resolver),
            Err(err) => {
                debug!("Failed to create DNS resolver for HTTP3 resolution: {}", err);
                None
            }
        };

        H3Engine {
            resolver,
            h3_alt_svc: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn host_supports_h3(&self, host: &String) -> bool {
        {
            let cache = self.h3_alt_svc.read().await;
            if let Some(&supports_h3) = cache.get(host) {
                return supports_h3;
            }
        }

        let domain_name = match Name::from_utf8(host) {
            Ok(name) => name,
            Err(_) => return false,
        };

        if let Some(resolver) = self.resolver.as_ref() {
            let response = resolver.lookup(domain_name, RecordType::HTTPS).await;

            let dns_h3_support = response.is_ok_and(|response| {
                response.answers().iter().any(|answer| {
                    if let RData::HTTPS(data) = &answer.data {
                        return data.0.svc_params.iter().any(|param| {
                            if let SvcParamValue::Alpn(alpn_protocols) = &param.1 {
                                return alpn_protocols.0.iter().any(|alpn| alpn == "h3");
                            }

                            false
                        });
                    }
                    false
                })
            });

            self.set_h3_support(host, dns_h3_support).await;

            return dns_h3_support;
        }

        false
    }

    pub async fn set_h3_support(&self, host: &String, supports_h3: bool) {
        let mut cache = self.h3_alt_svc.write().await;
        if cache.contains_key(host) {
            return;
        }

        cache.insert(host.to_owned(), supports_h3);
    }
}
