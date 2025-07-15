use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use hickory_proto::rr::rdata::svcb::SvcParamValue;
use hickory_proto::rr::{Name, RData};
use hickory_proto::runtime::TokioRuntimeProvider;
use hickory_proto::ProtoError;

use hickory_client::client::{Client, ClientHandle};
use hickory_client::proto::runtime::iocompat::AsyncIoTokioAsStd;
use hickory_client::proto::tcp::TcpClientStream;
use tokio::net::TcpStream as TokioTcpStream;

/// A struct encapsulating the components required to make HTTP/3 requests.
pub struct H3Engine {
    /// The DNS client used to resolve DNS queries.
    client: Arc<Mutex<Client>>,
    /// The background task that processes DNS queries.
    bg_join_handle: tokio::task::JoinHandle<Result<(), ProtoError>>,
    /// A map of hosts that support HTTP/3.
    ///
    /// This is populated by the DNS queries and manual calls to `set_h3_support` (based on the `Alt-Svc` header).
    /// Implicitly used as a cache for the DNS queries.
    h3_alt_svc: Arc<RwLock<HashMap<String, bool>>>,
}

impl H3Engine {
    pub async fn init() -> Self {
        // todo: use the DNS server from the system config
        let (stream, sender) = TcpClientStream::<AsyncIoTokioAsStd<TokioTcpStream>>::new(
            ([8, 8, 8, 8], 53).into(),
            None,
            None,
            TokioRuntimeProvider::new(),
        );
        let (client, bg) = Client::new(stream, sender, None).await.unwrap();

        let bg_join_handle = tokio::spawn(bg);

        H3Engine {
            client: Arc::new(Mutex::new(client)),
            bg_join_handle,
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

        let domain_name = Name::from_utf8(host).unwrap();

        let response = {
            let mut client = self.client.lock().await;
            client
                .query(
                    domain_name,
                    hickory_proto::rr::DNSClass::IN,
                    hickory_proto::rr::RecordType::HTTPS,
                )
                .await
        };

        let dns_h3_support = response.is_ok_and(|response| {
            response.answers().iter().any(|answer| {
                if let RData::HTTPS(data) = answer.data() {
                    return data.svc_params().iter().any(|param| {
                        if let SvcParamValue::Alpn(alpn_protocols) = param.1.clone() {
                            return alpn_protocols.0.iter().any(|alpn| alpn == "h3");
                        }

                        false
                    });
                }
                false
            })
        });

        self.set_h3_support(host, dns_h3_support).await;

        dns_h3_support
    }

    pub async fn set_h3_support(&self, host: &String, supports_h3: bool) {
        let mut cache = self.h3_alt_svc.write().await;
        if cache.contains_key(host) {
            return;
        }

        cache.insert(host.to_owned(), supports_h3);
    }
}

impl Drop for H3Engine {
    fn drop(&mut self) {
        self.bg_join_handle.abort();
    }
}
