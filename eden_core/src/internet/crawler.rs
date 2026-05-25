//! # Crawler - Web crawling engine
#![allow(dead_code)]
#![allow(non_snake_case)]

use super::{CrawlConfig, CrawlResult, Url};
use std::collections::{HashSet, VecDeque};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::{Duration, Instant};

/// Crawler principal
pub struct Crawler {
    config: CrawlConfig,
    visited: HashSet<String>,
    queue: VecDeque<(Url, usize)>, // (url, depth)
}

impl Crawler {
    pub fn new(config: CrawlConfig) -> Self {
        Self {
            config,
            visited: HashSet::new(),
            queue: VecDeque::new(),
        }
    }

    /// Start crawling from seed URLs
    pub fn crawl(&mut self, seeds: &[String]) -> Vec<CrawlResult> {
        Self::crawl_with_state(
            &self.config,
            &mut self.visited,
            &mut self.queue,
            seeds,
            |url| Self::crawl_url_with_config(&self.config, url),
        )
    }

    /// Start crawling with an injected fetcher.
    ///
    /// This keeps traversal, de-duplication, depth and same-host policy testable
    /// without opening network sockets.
    pub fn crawl_with_fetcher<F>(&mut self, seeds: &[String], mut fetcher: F) -> Vec<CrawlResult>
    where
        F: FnMut(&Url) -> CrawlResult,
    {
        Self::crawl_with_state(
            &self.config,
            &mut self.visited,
            &mut self.queue,
            seeds,
            |url| fetcher(url),
        )
    }

    fn crawl_with_state<F>(
        config: &CrawlConfig,
        visited: &mut HashSet<String>,
        queue: &mut VecDeque<(Url, usize)>,
        seeds: &[String],
        mut fetcher: F,
    ) -> Vec<CrawlResult>
    where
        F: FnMut(&Url) -> CrawlResult,
    {
        let mut results = Vec::new();

        // Seed the queue
        for seed in seeds {
            if let Ok(url) = Url::parse(seed) {
                queue.push_back((url, 0));
            }
        }

        // BFS crawl
        while let Some((url, depth)) = queue.pop_front() {
            // Check depth
            if depth > config.max_depth {
                continue;
            }

            // Check visited
            let url_str = url.to_string();
            if visited.contains(&url_str) {
                continue;
            }
            visited.insert(url_str.clone());

            // Check URL limit
            if results.len() >= config.max_urls {
                break;
            }

            // Crawl this URL
            let result = fetcher(&url);
            results.push(result);

            // Add new links to queue
            if let Some(last_result) = results.last() {
                if last_result.success {
                    for link in &last_result.links {
                        if let Some(resolved) = url.resolve(link) {
                            if config.follow_external || resolved.host == url.host {
                                queue.push_back((resolved, depth + 1));
                            }
                        }
                    }
                }
            }
        }

        results
    }

    fn crawl_url(&self, url: &Url) -> CrawlResult {
        Self::crawl_url_with_config(&self.config, url)
    }

    fn crawl_url_with_config(config: &CrawlConfig, url: &Url) -> CrawlResult {
        let start = Instant::now();

        // Connect to server
        let address_str = format!("{}:{}", url.host, url.port);
        let address: std::net::SocketAddr = match address_str.parse() {
            Ok(addr) => addr,
            Err(e) => {
                return CrawlResult::failure(
                    url.to_string(),
                    format!("Invalid address: {}", e),
                    start.elapsed().as_millis() as u64,
                )
            }
        };

        let mut stream =
            match TcpStream::connect_timeout(&address, Duration::from_millis(config.timeout_ms)) {
                Ok(s) => s,
                Err(e) => {
                    return CrawlResult::failure(
                        url.to_string(),
                        format!("Connection failed: {}", e),
                        start.elapsed().as_millis() as u64,
                    )
                }
            };

        // Set timeout
        stream
            .set_read_timeout(Some(Duration::from_millis(config.timeout_ms)))
            .ok();

        // Build HTTP request
        let request = format!(
            "GET {} HTTP/1.1\r\n\
            Host: {}\r\n\
            User-Agent: {}\r\n\
            Accept: text/html,application/xhtml+xml\r\n\
            Connection: close\r\n\
            \r\n",
            url.path, url.host, config.user_agent
        );

        // Send request
        if let Err(e) = stream.write_all(request.as_bytes()) {
            return CrawlResult::failure(
                url.to_string(),
                format!("Send failed: {}", e),
                start.elapsed().as_millis() as u64,
            );
        }

        // Read response
        let mut buffer = vec![0u8; 65536];
        let mut body_start = 0;

        // First read headers
        let mut headers = Vec::new();
        loop {
            let n = match stream.read(&mut buffer[headers.len()..]) {
                Ok(0) => break,
                Ok(n) => n,
                Err(e) => {
                    return CrawlResult::failure(
                        url.to_string(),
                        format!("Read failed: {}", e),
                        start.elapsed().as_millis() as u64,
                    )
                }
            };

            headers.extend_from_slice(&buffer[headers.len()..headers.len() + n]);

            // Check for end of headers
            if headers.len() > 4 {
                for i in 0..headers.len() - 3 {
                    if headers[i] == b'\r'
                        && headers[i + 1] == b'\n'
                        && headers[i + 2] == b'\r'
                        && headers[i + 3] == b'\n'
                    {
                        body_start = i + 4;
                        break;
                    }
                }
            }

            if body_start > 0 {
                break;
            }

            if headers.len() > 8192 {
                break; // Headers too long
            }
        }

        // Parse headers
        let header_str = String::from_utf8_lossy(&headers[..body_start]);
        let content_type = header_str
            .lines()
            .find(|l| l.to_lowercase().starts_with("content-type:"))
            .map(|l| l.split(':').nth(1).unwrap_or("").trim().to_string());

        // Check for redirects
        if let Some(redirect) = header_str.lines().find(|l| l.starts_with("Location:")) {
            let location = redirect.split(':').skip(1).collect::<Vec<_>>().join(":");
            return CrawlResult {
                url: url.to_string(),
                success: true,
                content_type: Some("redirect".to_string()),
                content: None,
                links: vec![location.trim().to_string()],
                title: None,
                error: None,
                duration_ms: start.elapsed().as_millis() as u64,
            };
        }

        // Get body
        let body = &buffer[body_start..];
        let links = Self::extract_links(&String::from_utf8_lossy(body));

        CrawlResult::success(
            url.to_string(),
            body.to_vec(),
            content_type.unwrap_or_else(|| "text/html".to_string()),
            links,
            start.elapsed().as_millis() as u64,
        )
    }

    fn extract_links(html: &str) -> Vec<String> {
        let mut links = Vec::new();

        // Simple link extraction
        let html_lower = html.to_lowercase();
        let mut search_start = 0;

        while let Some(start) = html_lower[search_start..].find("href=\"") {
            let start = search_start + start + 6;
            if let Some(end) = html[start..].find('"') {
                let link = &html[start..start + end];
                if !link.starts_with("javascript:") && !link.starts_with('#') {
                    links.push(link.to_string());
                }
                search_start = start + end;
            } else {
                break;
            }
        }

        links
    }
}

impl Default for Crawler {
    fn default() -> Self {
        Self::new(CrawlConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crawl_with_mock_fetcher_stays_local_and_dedupes() {
        let mut crawler = Crawler::new(CrawlConfig {
            max_depth: 2,
            max_urls: 4,
            follow_external: false,
            ..Default::default()
        });

        let results =
            crawler.crawl_with_fetcher(&["http://example.com/index".to_string()], |url| match url
                .path
                .as_str()
            {
                "/index" => CrawlResult::success(
                    url.to_string(),
                    b"root".to_vec(),
                    "text/html".to_string(),
                    vec![
                        "/next".to_string(),
                        "http://external.test/out".to_string(),
                        "/next".to_string(),
                    ],
                    1,
                ),
                "/next" => CrawlResult::success(
                    url.to_string(),
                    b"next".to_vec(),
                    "text/html".to_string(),
                    Vec::new(),
                    1,
                ),
                _ => CrawlResult::failure(url.to_string(), "unexpected URL".to_string(), 1),
            });

        let urls: Vec<_> = results.iter().map(|result| result.url.as_str()).collect();
        assert_eq!(
            urls,
            vec!["http://example.com/index", "http://example.com/next"]
        );
        assert!(results.iter().all(|result| result.success));
    }

    #[test]
    #[cfg_attr(not(feature = "external-tests"), ignore)]
    fn test_crawl_google() {
        let mut crawler = Crawler::new(CrawlConfig {
            max_depth: 1,
            max_urls: 1,
            ..Default::default()
        });

        let results = crawler.crawl(&["http://example.com".to_string()]);
        println!("Got {} results", results.len());
    }
}
