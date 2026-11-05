use reqwest::{
    Client, Response, Url,
    header::{AUTHORIZATION, HeaderValue},
};
use serde_json::{Value, json};
use std::time::Duration;

#[derive(Clone)]
pub struct GithubClient {
    client: Client,
    api: Url,
    web: Url,
}
impl GithubClient {
    #[cfg(test)]
    pub(crate) fn fixture(origin: &str) -> Self {
        let api = Url::parse(origin).unwrap();
        Self {
            client: Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .timeout(Duration::from_secs(2))
                .build()
                .unwrap(),
            web: api.clone(),
            api,
        }
    }
    pub fn new() -> Result<Self, &'static str> {
        let client = Client::builder()
            .https_only(true)
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(10))
            .connect_timeout(Duration::from_secs(5))
            .user_agent("AlexIA/0.1")
            .build()
            .map_err(|_| "http_unavailable")?;
        Ok(Self {
            client,
            api: Url::parse("https://api.github.com/").unwrap(),
            web: Url::parse("https://github.com/").unwrap(),
        })
    }

    fn endpoint(&self, segments: &[&str]) -> Result<Url, &'static str> {
        let mut url = self.api.clone();
        let mut path = url.path_segments_mut().map_err(|_| "invalid_input")?;
        path.clear();
        for segment in segments {
            if segment.is_empty() || matches!(*segment, "." | "..") {
                return Err("invalid_input");
            }
            path.push(segment);
        }
        drop(path);
        Ok(url)
    }

    pub async fn get(
        &self,
        token: &str,
        segments: &[&str],
        query: &[(&str, String)],
    ) -> Result<Value, &'static str> {
        let mut authorization =
            HeaderValue::from_str(&format!("Bearer {token}")).map_err(|_| "invalid_token")?;
        authorization.set_sensitive(true);
        let response = self
            .client
            .get(self.endpoint(segments)?)
            .header(AUTHORIZATION, authorization)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2026-03-10")
            .query(query)
            .send()
            .await
            .map_err(|_| "github_unavailable")?;
        read_json(response).await
    }

    pub async fn pages(
        &self,
        token: &str,
        segments: &[&str],
        query: &[(&str, String)],
        field: Option<&str>,
    ) -> Result<Vec<Value>, &'static str> {
        let mut items = Vec::new();
        for page in 1..=20 {
            let mut query = query.to_vec();
            query.extend([("per_page", "100".into()), ("page", page.to_string())]);
            let value = self.get(token, segments, &query).await?;
            let array = field
                .map_or(&value, |field| &value[field])
                .as_array()
                .ok_or("github_invalid_response")?;
            items.extend(array.iter().cloned());
            if array.len() < 100 {
                return Ok(items);
            }
        }
        Err("partial_coverage")
    }

    pub async fn device_code(&self, client_id: &str) -> Result<Value, &'static str> {
        let url = self.web.join("login/device/code").unwrap();
        let response = self
            .client
            .post(url)
            .header("Accept", "application/json")
            .json(&json!({"client_id":client_id}))
            .send()
            .await
            .map_err(|_| "github_unavailable")?;
        read_json(response).await
    }

    pub async fn device_token(
        &self,
        client_id: &str,
        device_code: &str,
    ) -> Result<Value, &'static str> {
        let response = self.client.post(self.web.join("login/oauth/access_token").unwrap()).header("Accept", "application/json")
            .json(&json!({"client_id":client_id,"device_code":device_code,"grant_type":"urn:ietf:params:oauth:grant-type:device_code"}))
            .send().await.map_err(|_| "github_unavailable")?;
        read_json(response).await
    }
}

async fn read_json(mut response: Response) -> Result<Value, &'static str> {
    match response.status().as_u16() {
        200..=299 => {}
        401 => return Err("connection_expired"),
        403 | 429 => return Err("permission_denied_or_rate_limited"),
        404 => return Err("resource_unavailable"),
        _ => return Err("github_error"),
    }
    let mut bytes = Vec::new();
    if response
        .content_length()
        .is_some_and(|length| length > 1024 * 1024)
    {
        return Err("output_too_large");
    }
    while let Some(chunk) = response.chunk().await.map_err(|_| "github_unavailable")? {
        if bytes.len() + chunk.len() > 1024 * 1024 {
            return Err("output_too_large");
        }
        bytes.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&bytes).map_err(|_| "github_invalid_response")
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, routing::get};
    #[tokio::test]
    async fn transport_rejects_redirects_unauthorized_and_oversized_json() {
        let router = Router::new()
            .route(
                "/redirect",
                get(|| async {
                    (
                        axum::http::StatusCode::FOUND,
                        [("location", "https://example.com")],
                        "",
                    )
                }),
            )
            .route("/huge", get(|| async { "a".repeat(1024 * 1024 + 1) }))
            .route(
                "/expired",
                get(|| async { axum::http::StatusCode::UNAUTHORIZED }),
            );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let api = Url::parse(&format!("http://{}/", listener.local_addr().unwrap())).unwrap();
        let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
        let client = GithubClient {
            client: Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .unwrap(),
            web: api.clone(),
            api,
        };
        assert_eq!(
            client
                .get("synthetic", &["redirect"], &[])
                .await
                .unwrap_err(),
            "github_error"
        );
        assert_eq!(
            client.get("synthetic", &["huge"], &[]).await.unwrap_err(),
            "output_too_large"
        );
        assert_eq!(
            client
                .get("synthetic", &["expired"], &[])
                .await
                .unwrap_err(),
            "connection_expired"
        );
        let encoded = client.endpoint(&["branches", "feature/a?x=1"]).unwrap();
        assert!(encoded.as_str().contains("feature%2Fa%3Fx=1"));
        server.abort();
    }
}
