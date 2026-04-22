use clap::ValueEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum LogLevel {
    Info,
    Debug,
}

impl LogLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Debug => "debug",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProxyRequest {
    pub method: String,
    pub path_and_query: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProxyResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

pub fn sanitize_outbound_headers(headers: &[(String, String)]) -> Vec<(String, String)> {
    headers
        .iter()
        .filter(|(name, _)| {
            !name.eq_ignore_ascii_case("authorization")
                && !name.eq_ignore_ascii_case("host")
                && !name.eq_ignore_ascii_case("content-length")
                && !name.eq_ignore_ascii_case("connection")
                && !name.eq_ignore_ascii_case("proxy-authenticate")
                && !name.eq_ignore_ascii_case("proxy-authorization")
                && !name.eq_ignore_ascii_case("te")
                && !name.eq_ignore_ascii_case("trailers")
                && !name.eq_ignore_ascii_case("transfer-encoding")
                && !name.eq_ignore_ascii_case("upgrade")
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::sanitize_outbound_headers;

    #[test]
    fn strips_authorization_and_hop_by_hop_headers() {
        let headers = vec![
            ("Authorization".to_string(), "Bearer local".to_string()),
            ("Host".to_string(), "localhost:11434".to_string()),
            ("Connection".to_string(), "keep-alive".to_string()),
            ("Content-Type".to_string(), "application/json".to_string()),
        ];

        let sanitized = sanitize_outbound_headers(&headers);

        assert_eq!(
            sanitized,
            vec![("Content-Type".to_string(), "application/json".to_string())]
        );
    }
}
