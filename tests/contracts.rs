use axum::{body::{Body, to_bytes}, http::{Request, StatusCode}};
use gemilexa::{app, config::Config, providers::{LanguageModel, ProviderError, gemini::GeminiProvider}, voice::normalize_for_voice};
use tower::ServiceExt;

#[tokio::test]
async fn health_reports_liveness_and_disabled_integrations() {
    let response = app().oneshot(Request::get("/health").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 4096).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
    assert_eq!(json["alexa"], "blocked");
    assert_eq!(json["gemini"], "not_configured");
}

#[tokio::test]
async fn alexa_never_accepts_requests_even_with_signature_headers() {
    for headers in [false, true] {
        for payload in ["", "not json", r#"{"request":{"timestamp":"2000-01-01T00:00:00Z"}}"#] {
            let mut request = Request::post("/alexa");
            if headers {
                request = request.header("Signature", "fake")
                    .header("Signature-256", "fake-sha256").header("SignatureCertChainUrl", "https://s3.amazonaws.com/echo.api/fake.pem");
            }
            let response = app().oneshot(request.body(Body::from(payload)).unwrap()).await.unwrap();
            assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
            let body = to_bytes(response.into_body(), 4096).await.unwrap();
            let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
            assert_eq!(json["error"], "alexa_verification_not_implemented");
        }
    }
}

#[tokio::test]
async fn alexa_is_blocked_for_other_methods_too() {
    let response = app().oneshot(Request::get("/alexa").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn gemini_is_explicitly_unavailable() {
    let provider: Box<dyn LanguageModel> = Box::new(GeminiProvider);
    assert_eq!(provider.generate("Hola").await, Err(ProviderError::NotConfigured));
}

#[test]
fn voice_removes_markdown_links_images_and_code_blocks() {
    let input = "# Hola\n\n**mundo** y [Alexa](https://example.com).\n\n![foto](photo.png)\n\n```rust\nsecret_code();\n```\n\n- Uno\n- Dos";
    assert_eq!(normalize_for_voice(input, 800), "Hola mundo y Alexa. Uno Dos");
}

#[test]
fn voice_outputs_plain_text_and_preserves_unicode() {
    assert_eq!(normalize_for_voice("Hola <em>Colombia</em> & amigos", 800), "Hola Colombia & amigos");
    assert_eq!(normalize_for_voice("áéíóú 🦀", 3), "áéí");
    assert_eq!(normalize_for_voice("hola", 0), "");
    assert_eq!(normalize_for_voice("   \n\t", 800), "");
    assert!(!normalize_for_voice("&lt;speak&gt;hola&lt;/speak&gt;", 800).contains('<'));
}

#[test]
fn config_rejects_invalid_addresses_and_zero_ports() {
    assert!(Config::from_bind_addr("not-an-address").is_err());
    assert!(Config::from_bind_addr("127.0.0.1:0").is_err());
    assert_eq!(Config::from_bind_addr("127.0.0.1:3000").unwrap().bind_addr.port(), 3000);
}
