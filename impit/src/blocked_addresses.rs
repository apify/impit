//! Address blocklist: IP addresses and CIDR ranges a client never connects to.
//!
//! The blocklist is enforced in two places. IP literals in URLs are checked before the request is sent and on every
//! redirect hop. Hostnames are checked by a DNS resolver wrapper that fails the lookup when any resolved address is
//! blocked, so the connection can only go to an address the policy approved. That closes the gap between checking a
//! hostname and connecting to it, which a DNS record changing in between (DNS rebinding) would otherwise exploit.
//!
//! Requests sent through a proxy are not checked, because the proxy resolves and connects to the target.

use std::{
    error::Error,
    fmt,
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

use ipnet::IpNet;
use reqwest::dns::{Addrs, Name, Resolve, Resolving};
use url::{Host, Url};

use crate::errors::ImpitError;

type BoxError = Box<dyn Error + Send + Sync>;

/// IP addresses and CIDR ranges a client never sends a request to.
#[derive(Debug, Clone, Default)]
pub struct BlockedAddresses {
    networks: Vec<IpNet>,
}

impl BlockedAddresses {
    /// Parses entries such as `127.0.0.1`, `10.0.0.0/8` or `fc00::/7`.
    ///
    /// A single address blocks that one host. Host bits set in a range are ignored, so `10.1.2.3/8` means
    /// `10.0.0.0/8`.
    pub fn parse<I, S>(entries: I) -> Result<Self, ImpitError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut networks = Vec::new();
        for entry in entries {
            let entry = entry.as_ref().trim();
            let network = entry
                .parse::<IpNet>()
                .or_else(|_| entry.parse::<IpAddr>().map(IpNet::from))
                .map_err(|_| ImpitError::InvalidBlockedAddress(entry.to_string()))?;
            networks.push(network.trunc());
        }
        Ok(Self { networks })
    }

    pub fn is_empty(&self) -> bool {
        self.networks.is_empty()
    }

    /// Returns whether the address is blocked.
    ///
    /// An IPv4-mapped IPv6 address such as `::ffff:10.0.0.1` is also matched as the IPv4 address it embeds, so
    /// `10.0.0.0/8` blocks both spellings.
    pub fn contains(&self, address: IpAddr) -> bool {
        let mut spellings = vec![address];
        if let IpAddr::V6(v6) = address {
            if let Some(v4) = v6.to_ipv4_mapped() {
                spellings.push(IpAddr::V4(v4));
            }
        }
        spellings.iter().any(|spelling| {
            self.networks
                .iter()
                .any(|network| network.contains(spelling))
        })
    }

    /// Fails if the URL's host is an IP literal on the blocklist.
    ///
    /// Hostnames pass here and are checked by [`BlockingResolver`] when they are resolved. The URL parser already
    /// normalizes alternative IPv4 spellings such as `2130706433` or `0x7f.0.0.1`, so they are checked as the
    /// address they denote.
    pub fn check_url(&self, url: &Url) -> Result<(), BlockedAddressError> {
        if self.is_empty() {
            return Ok(());
        }
        let address = match url.host() {
            Some(Host::Ipv4(address)) => IpAddr::V4(address),
            Some(Host::Ipv6(address)) => IpAddr::V6(address),
            _ => return Ok(()),
        };
        if self.contains(address) {
            return Err(BlockedAddressError {
                host: url.host_str().unwrap_or_default().to_string(),
                address,
            });
        }
        Ok(())
    }
}

/// The error a blocked address produces, carried as the source of the transport error it causes.
#[derive(Debug, Clone)]
pub struct BlockedAddressError {
    /// The host of the request, either the IP literal or the hostname that resolved to the blocked address.
    pub host: String,
    /// The blocked address.
    pub address: IpAddr,
}

impl fmt::Display for BlockedAddressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Refusing to connect to `{}`: the address {} is blocked.",
            self.host, self.address
        )
    }
}

impl Error for BlockedAddressError {}

impl From<BlockedAddressError> for ImpitError {
    fn from(error: BlockedAddressError) -> Self {
        ImpitError::BlockedAddress(error.host, error.address.to_string())
    }
}

/// Finds a [`BlockedAddressError`] anywhere in an error's source chain.
pub(crate) fn find_blocked_address_error(
    error: &(dyn Error + 'static),
) -> Option<BlockedAddressError> {
    let mut current: Option<&(dyn Error + 'static)> = Some(error);
    while let Some(err) = current {
        if let Some(blocked) = err.downcast_ref::<BlockedAddressError>() {
            return Some(blocked.clone());
        }
        current = err.source();
    }
    None
}

/// A DNS resolver that fails the lookup of any name resolving to a blocked address.
///
/// The connector only ever sees addresses this resolver returned, so a name that passed cannot lead to a blocked
/// address through a later, different resolution.
pub(crate) struct BlockingResolver {
    inner: Arc<dyn Resolve>,
    blocked: BlockedAddresses,
}

impl BlockingResolver {
    pub(crate) fn new(inner: Arc<dyn Resolve>, blocked: BlockedAddresses) -> Self {
        Self { inner, blocked }
    }
}

impl Resolve for BlockingResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let inner = self.inner.clone();
        let blocked = self.blocked.clone();
        Box::pin(async move {
            let host = name.as_str().to_string();
            let addresses: Vec<SocketAddr> = inner.resolve(name).await?.collect();
            if let Some(address) = addresses
                .iter()
                .map(SocketAddr::ip)
                .find(|address| blocked.contains(*address))
            {
                return Err(Box::new(BlockedAddressError { host, address }) as BoxError);
            }
            Ok(Box::new(addresses.into_iter()) as Addrs)
        })
    }
}

/// The system resolver, which is what `reqwest` uses when no resolver is configured.
pub(crate) struct SystemResolver;

impl Resolve for SystemResolver {
    fn resolve(&self, name: Name) -> Resolving {
        Box::pin(async move {
            let addresses: Vec<SocketAddr> = tokio::net::lookup_host((name.as_str(), 0))
                .await
                .map_err(|error| Box::new(error) as BoxError)?
                .collect();
            Ok(Box::new(addresses.into_iter()) as Addrs)
        })
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    struct FakeResolver(Vec<IpAddr>);

    impl Resolve for FakeResolver {
        fn resolve(&self, _name: Name) -> Resolving {
            let addresses: Vec<SocketAddr> =
                self.0.iter().map(|ip| SocketAddr::new(*ip, 0)).collect();
            Box::pin(async move { Ok(Box::new(addresses.into_iter()) as Addrs) })
        }
    }

    fn blocklist() -> BlockedAddresses {
        BlockedAddresses::parse(["127.0.0.0/8", "10.1.2.3/8", "169.254.169.254", "fc00::/7"])
            .unwrap()
    }

    #[test]
    fn parses_addresses_and_ranges() {
        let networks: Vec<String> = blocklist()
            .networks
            .iter()
            .map(ToString::to_string)
            .collect();
        assert_eq!(
            networks,
            [
                "127.0.0.0/8",
                "10.0.0.0/8",
                "169.254.169.254/32",
                "fc00::/7"
            ]
        );
    }

    #[test]
    fn rejects_hostnames() {
        assert!(matches!(
            BlockedAddresses::parse(["localhost"]),
            Err(ImpitError::InvalidBlockedAddress(entry)) if entry == "localhost"
        ));
    }

    #[test]
    fn matches_addresses() {
        let blocked = blocklist();
        assert!(blocked.contains("127.0.0.1".parse().unwrap()));
        assert!(blocked.contains("10.255.0.1".parse().unwrap()));
        assert!(blocked.contains("169.254.169.254".parse().unwrap()));
        assert!(blocked.contains("fd12::1".parse().unwrap()));
        assert!(blocked.contains("::ffff:127.0.0.1".parse().unwrap()));
        assert!(!blocked.contains("169.254.169.253".parse().unwrap()));
        assert!(!blocked.contains("93.184.216.34".parse().unwrap()));
        assert!(!blocked.contains("2001:db8::1".parse().unwrap()));
    }

    #[test]
    fn checks_url_literals_in_every_spelling() {
        let blocked = blocklist();
        for url in [
            "http://127.0.0.1/",
            "http://2130706433/",
            "http://0x7f.0.0.1/",
            "http://127.1/",
            "http://[::ffff:127.0.0.1]/",
            "http://[fd12::1]:8080/",
        ] {
            assert!(
                blocked.check_url(&Url::parse(url).unwrap()).is_err(),
                "{url} should be blocked"
            );
        }
        assert!(blocked
            .check_url(&Url::parse("http://93.184.216.34/").unwrap())
            .is_ok());
        assert!(blocked
            .check_url(&Url::parse("http://localhost/").unwrap())
            .is_ok());
        assert!(BlockedAddresses::default()
            .check_url(&Url::parse("http://127.0.0.1/").unwrap())
            .is_ok());
    }

    #[tokio::test]
    async fn resolver_fails_when_any_address_is_blocked() {
        let resolver = BlockingResolver::new(
            Arc::new(FakeResolver(vec![
                "93.184.216.34".parse().unwrap(),
                "127.0.0.1".parse().unwrap(),
            ])),
            blocklist(),
        );
        let error = resolver
            .resolve(Name::from_str("rebind.example").unwrap())
            .await
            .err()
            .unwrap();
        let blocked = find_blocked_address_error(error.as_ref()).unwrap();
        assert_eq!(blocked.host, "rebind.example");
        assert_eq!(blocked.address, "127.0.0.1".parse::<IpAddr>().unwrap());
    }

    #[tokio::test]
    async fn resolver_passes_allowed_addresses_through() {
        let resolver = BlockingResolver::new(
            Arc::new(FakeResolver(vec!["93.184.216.34".parse().unwrap()])),
            blocklist(),
        );
        let addresses: Vec<SocketAddr> = resolver
            .resolve(Name::from_str("example.com").unwrap())
            .await
            .unwrap()
            .collect();
        assert_eq!(
            addresses,
            ["93.184.216.34:0".parse::<SocketAddr>().unwrap()]
        );
    }
}
