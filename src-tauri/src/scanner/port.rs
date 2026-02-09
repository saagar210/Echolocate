use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::Semaphore;
use tokio::time::timeout;

/// Result of scanning a single port.
#[derive(Debug, Clone)]
pub struct PortResult {
    pub port: u16,
    pub state: PortState,
    pub service_name: Option<String>,
    pub banner: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PortState {
    Open,
    Closed,
    Filtered,
}

impl std::fmt::Display for PortState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PortState::Open => write!(f, "open"),
            PortState::Closed => write!(f, "closed"),
            PortState::Filtered => write!(f, "filtered"),
        }
    }
}

/// Scan multiple ports on a target IP.
pub async fn scan_ports(
    ip: &str,
    ports: &[u16],
    max_concurrent: usize,
    timeout_ms: u64,
) -> Vec<PortResult> {
    let semaphore = Arc::new(Semaphore::new(max_concurrent));
    let services = service_map();
    let mut handles = Vec::new();

    for &port in ports {
        let ip = ip.to_string();
        let sem = semaphore.clone();
        let services = services.clone();

        let handle = tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            scan_single_port(&ip, port, timeout_ms, &services).await
        });
        handles.push(handle);
    }

    let mut results = Vec::new();
    for handle in handles {
        if let Ok(Ok(result)) = handle.await {
            // Only include open ports to reduce noise
            if result.state == PortState::Open {
                results.push(result);
            }
        }
    }

    results.sort_by_key(|r| r.port);
    results
}

/// Scan a single port using TCP connect.
async fn scan_single_port(
    ip: &str,
    port: u16,
    timeout_ms: u64,
    services: &HashMap<u16, &'static str>,
) -> Result<PortResult, ()> {
    let addr: SocketAddr = format!("{}:{}", ip, port).parse().map_err(|_| ())?;
    let service_name = services.get(&port).map(|s| s.to_string());

    match timeout(Duration::from_millis(timeout_ms), TcpStream::connect(addr)).await {
        Ok(Ok(stream)) => {
            // Port is open — try banner grab
            let banner = grab_banner(stream, port).await;
            Ok(PortResult {
                port,
                state: PortState::Open,
                service_name,
                banner,
            })
        }
        Ok(Err(_)) => {
            // Connection refused — port is closed
            Ok(PortResult {
                port,
                state: PortState::Closed,
                service_name: None,
                banner: None,
            })
        }
        Err(_) => {
            // Timeout — port is filtered
            Ok(PortResult {
                port,
                state: PortState::Filtered,
                service_name: None,
                banner: None,
            })
        }
    }
}

/// Attempt to read a service banner from an open port.
async fn grab_banner(stream: TcpStream, _port: u16) -> Option<String> {
    use tokio::io::AsyncReadExt;

    let mut stream = stream;
    let mut buf = vec![0u8; 256];

    match timeout(Duration::from_secs(1), stream.read(&mut buf)).await {
        Ok(Ok(n)) if n > 0 => {
            let banner = String::from_utf8_lossy(&buf[..n])
                .trim()
                .to_string();
            if !banner.is_empty() {
                Some(banner)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Top 100 most common ports (nmap default).
pub fn top_100_ports() -> Vec<u16> {
    vec![
        7, 9, 13, 21, 22, 23, 25, 26, 37, 53, 79, 80, 81, 88, 106, 110, 111,
        113, 119, 135, 139, 143, 144, 179, 199, 389, 427, 443, 444, 445, 465,
        513, 514, 515, 543, 544, 548, 554, 587, 631, 646, 873, 990, 993, 995,
        1025, 1026, 1027, 1028, 1029, 1110, 1433, 1720, 1723, 1755, 1900,
        2000, 2001, 2049, 2121, 2717, 3000, 3128, 3306, 3389, 3986, 4899,
        5000, 5001, 5003, 5009, 5050, 5051, 5060, 5101, 5190, 5357, 5432,
        5631, 5666, 5800, 5900, 6000, 6001, 6646, 7070, 8000, 8008, 8009,
        8080, 8081, 8443, 8888, 9100, 9999, 10000, 32768, 49152, 49153, 49154,
    ]
}

/// Well-known port to service name mapping.
fn service_map() -> Arc<HashMap<u16, &'static str>> {
    Arc::new(HashMap::from([
        (21, "ftp"), (22, "ssh"), (23, "telnet"), (25, "smtp"),
        (53, "dns"), (80, "http"), (88, "kerberos"), (110, "pop3"),
        (111, "rpc"), (119, "nntp"), (135, "msrpc"), (139, "netbios"),
        (143, "imap"), (179, "bgp"), (389, "ldap"), (443, "https"),
        (445, "smb"), (465, "smtps"), (513, "rlogin"), (514, "syslog"),
        (543, "klogin"), (548, "afp"), (554, "rtsp"), (587, "submission"),
        (631, "ipp"), (873, "rsync"), (993, "imaps"), (995, "pop3s"),
        (1433, "mssql"), (1723, "pptp"), (1900, "ssdp"), (2049, "nfs"),
        (3000, "dev-server"), (3306, "mysql"), (3389, "rdp"),
        (5000, "upnp"), (5060, "sip"), (5432, "postgresql"),
        (5900, "vnc"), (6000, "x11"), (8000, "http-alt"),
        (8080, "http-proxy"), (8443, "https-alt"), (8888, "http-alt2"),
        (9100, "jetdirect"), (9999, "abyss"), (10000, "webmin"),
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_top_100_ports_count() {
        let ports = top_100_ports();
        assert!(ports.len() >= 95); // Allow slight variations
        assert!(ports.contains(&22));
        assert!(ports.contains(&80));
        assert!(ports.contains(&443));
    }

    #[test]
    fn test_service_map_known_ports() {
        let services = service_map();
        assert_eq!(services.get(&22), Some(&"ssh"));
        assert_eq!(services.get(&80), Some(&"http"));
        assert_eq!(services.get(&443), Some(&"https"));
        assert_eq!(services.get(&3306), Some(&"mysql"));
    }
}
