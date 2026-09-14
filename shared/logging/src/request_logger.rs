use actix_web::{
    body::{to_bytes, BoxBody, MessageBody},
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::header,
    web, Error, HttpMessage, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use futures_util::StreamExt;
use std::future::{ready, Ready};
use std::rc::Rc;
use std::time::Instant;
use uuid::Uuid;

/// Strongly typed request metadata extracted by the inbound interceptor.
#[derive(Clone, Debug)]
pub struct RequestMetadata {
    pub request_id: Uuid,
    pub start_time: Instant,
    pub path: String,
    pub method: String,
    pub client_ip: String,
    pub user_agent: String,
    pub auth_token: Option<String>,
    pub ga_uuid: Option<String>,
    pub timestamp: String,
}

/// Strongly typed container for decrypted request body injected into `req.extensions_mut()`.
#[derive(Clone, Debug)]
pub struct DecryptedBody(pub serde_json::Value);

#[derive(Default, Clone)]
pub struct AuditLoggingMiddleware;

pub struct AuditLoggingMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Transform<S, ServiceRequest> for AuditLoggingMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type InitError = ();
    type Transform = AuditLoggingMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuditLoggingMiddlewareService {
            service: Rc::new(service),
        }))
    }
}

impl<S, B> Service<ServiceRequest> for AuditLoggingMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // 1. START STOPWATCH & METADATA EXTRACTION
        let start_time = Instant::now();
        let request_id = Uuid::new_v4();

        let path = req.path().to_string();
        let method = req.method().to_string();
        let client_ip = extract_client_ip(&req);
        let user_agent = req
            .headers()
            .get(header::USER_AGENT)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown")
            .to_string();
        let auth_token = extract_auth_token(&req);
        let ga_uuid = req
            .headers()
            .get("ga-uuid")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let timestamp = req
            .headers()
            .get("x-timestamp")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        let x_ip_address = req
            .headers()
            .get("x-ip-address")
            .and_then(|v| v.to_str().ok())
            .unwrap_or(&client_ip)
            .to_string();

        // 2. CHECK ROUTE POLICIES
        let should_log = crypto::ROUTE_POLICY.should_log(&path);
        let is_encrypted_request = crypto::ROUTE_POLICY.is_encrypted_request(&path);
        let is_encrypted_response = crypto::ROUTE_POLICY.is_encrypted_response(&path);

        // Security flags & keys
        let is_vapt = std::env::var("VAPT")
            .map(|v| v.eq_ignore_ascii_case("yes"))
            .unwrap_or(false);

        let secret = std::env::var("AES_SECRET")
            .unwrap_or_else(|_| "5b6714126d3149fbab994747b2633287".to_string());
        let hash_key = std::env::var("AES_HASH_KEY")
            .unwrap_or_else(|_| "r4vcos0ejvndsow95n".to_string());
        let default_aes_key = crypto::derive_default_aes_key(&secret, &hash_key);

        // Save RequestMetadata in extensions
        let metadata = RequestMetadata {
            request_id,
            start_time,
            path: path.clone(),
            method: method.clone(),
            client_ip: client_ip.clone(),
            user_agent,
            auth_token,
            ga_uuid,
            timestamp: timestamp.clone(),
        };
        req.extensions_mut().insert(metadata);

        let svc = self.service.clone();

        Box::pin(async move {
            // 3. INBOUND PAYLOAD BUFFERING & DECRYPTION
            let (http_req, mut payload) = req.into_parts();

            let mut body_bytes = web::BytesMut::new();
            while let Some(chunk) = payload.next().await {
                match chunk {
                    Ok(c) => body_bytes.extend_from_slice(&c),
                    Err(e) => return Err(e.into()),
                }
            }
            let body_bytes = body_bytes.freeze();

            let decrypted_bytes = if !body_bytes.is_empty() && is_encrypted_request {
                if let Ok(json_val) = serde_json::from_slice::<serde_json::Value>(&body_bytes) {
                    if let Some(enc_data) = json_val.get("data").and_then(|d| d.as_str()) {
                        let decrypt_res = if is_vapt {
                            let priv_pem = std::fs::read_to_string("files/private.pem")
                                .unwrap_or_default();
                            crypto::decrypt_hybrid_rsa(enc_data, &priv_pem)
                        } else {
                            crypto::decrypt_aes_cryptojs(enc_data, &default_aes_key)
                        };

                        match decrypt_res {
                            Ok(plain_text) => {
                                if let Ok(parsed_json) =
                                    serde_json::from_str::<serde_json::Value>(&plain_text)
                                {
                                    http_req.extensions_mut().insert(DecryptedBody(parsed_json));
                                }
                                web::Bytes::from(plain_text)
                            }
                            Err(err) => {
                                tracing::warn!("Inbound payload decryption failed: {err}");
                                body_bytes
                            }
                        }
                    } else {
                        body_bytes
                    }
                } else {
                    body_bytes
                }
            } else {
                body_bytes
            };

            // Reconstruct the request payload stream so downstream handlers can parse it
            let (_, mut new_payload) = actix_http::h1::Payload::create(true);
            new_payload.unread_data(decrypted_bytes);
            let reconstructed_req = ServiceRequest::from_parts(http_req, new_payload.into());

            // 4. INVOKE NEXT SERVICE IN ONION PIPELINE
            let fut = svc.call(reconstructed_req);
            let res = fut.await;

            match res {
                Ok(srv_res) => {
                    let latency_ms = start_time.elapsed().as_millis();
                    let status = srv_res.status().as_u16();

                    // Emit structured console diagnostic log if URL is not excluded
                    if should_log {
                        tracing::info!(
                            target: "LoggerInterceptor",
                            request_id = %request_id,
                            client_ip = %client_ip,
                            "{} | [{}] {} - {}ms",
                            status,
                            method,
                            path,
                            latency_ms
                        );
                    }

                    // 5. OUTBOUND RESPONSE BODY TRANSFORMATION
                    let (http_req, http_res) = srv_res.into_parts();
                    let res_status = http_res.status();
                    let body_bytes = to_bytes(http_res.into_body()).await.unwrap_or_default();
                    let body_str = String::from_utf8_lossy(&body_bytes).to_string();

                    let transformed_res = if is_encrypted_response && !body_bytes.is_empty() {
                        // Dynamic key: HMAC-SHA256(secret + ip + timestamp, hashKey)
                        let effective_ts = if timestamp.is_empty() {
                            chrono::Utc::now().timestamp().to_string()
                        } else {
                            timestamp.clone()
                        };
                        let dynamic_key = crypto::derive_dynamic_aes_key(
                            &secret,
                            &x_ip_address,
                            &effective_ts,
                            &hash_key,
                        );

                        match crypto::encrypt_aes_cryptojs(&body_str, &dynamic_key) {
                            Ok(encrypted_b64) => {
                                let envelope = serde_json::json!({ "body": encrypted_b64 });
                                HttpResponse::build(res_status)
                                    .content_type("application/json")
                                    .json(envelope)
                                    .map_into_boxed_body()
                            }
                            Err(e) => {
                                tracing::error!("Failed to encrypt response body: {e}");
                                HttpResponse::build(res_status)
                                    .content_type("application/json")
                                    .body(body_bytes)
                                    .map_into_boxed_body()
                            }
                        }
                    } else if crypto::ROUTE_POLICY.is_not_encrypted_webhook(&path) {
                        // Webhook: Return raw body as is
                        HttpResponse::build(res_status)
                            .body(body_bytes)
                            .map_into_boxed_body()
                    } else {
                        // Unencrypted response: standard JSON / body
                        HttpResponse::build(res_status)
                            .content_type("application/json")
                            .body(body_bytes)
                            .map_into_boxed_body()
                    };

                    Ok(ServiceResponse::new(http_req, transformed_res))
                }
                Err(err) => {
                    let latency_ms = start_time.elapsed().as_millis();

                    if should_log {
                        tracing::error!(
                            target: "LoggerInterceptor",
                            request_id = %request_id,
                            client_ip = %client_ip,
                            error = %err,
                            "500 | [{}] {} - {}ms",
                            method,
                            path,
                            latency_ms
                        );
                    }

                    Err(err)
                }
            }
        })
    }
}

fn extract_client_ip(req: &ServiceRequest) -> String {
    if let Some(ip) = req.headers().get("true-client-ip") {
        if let Ok(ip_str) = ip.to_str() {
            return ip_str.to_string();
        }
    }
    if let Some(forwarded) = req.headers().get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            if let Some(first_ip) = forwarded_str.split(',').next() {
                return first_ip.trim().to_string();
            }
        }
    }
    req.connection_info()
        .realip_remote_addr()
        .unwrap_or("unknown")
        .to_string()
}

fn extract_auth_token(req: &ServiceRequest) -> Option<String> {
    if let Some(auth_val) = req.headers().get("authorization") {
        if let Ok(auth_str) = auth_val.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                return Some(token.to_string());
            }
            return Some(auth_str.to_string());
        }
    }

    let query_str = req.query_string();
    for param in query_str.split('&') {
        if let Some((key, val)) = param.split_once('=') {
            if key == "access_token" {
                return Some(val.to_string());
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{get, post, test, App};

    #[get("/health")]
    async fn health() -> HttpResponse {
        HttpResponse::Ok().json(serde_json::json!({ "status": "UP" }))
    }

    #[post("/api/v1/user/test-encrypted")]
    async fn test_encrypted_route(body: web::Json<serde_json::Value>) -> HttpResponse {
        HttpResponse::Ok().json(serde_json::json!({
            "received": body.into_inner()
        }))
    }

    #[actix_web::test]
    async fn test_unencrypted_health_route() {
        let app = test::init_service(App::new().wrap(AuditLoggingMiddleware).service(health)).await;

        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["status"], "UP");
    }

    #[actix_web::test]
    async fn test_encrypted_request_and_response_pipeline() {
        let app = test::init_service(
            App::new()
                .wrap(AuditLoggingMiddleware)
                .service(test_encrypted_route),
        )
        .await;

        // 1. Plaintext client payload
        let plain_payload = r#"{"name":"ViratUser","balance":50000}"#;
        let secret = "5b6714126d3149fbab994747b2633287";
        let hash_key = "r4vcos0ejvndsow95n";
        let default_key = crypto::derive_default_aes_key(secret, hash_key);

        // 2. Encrypt plaintext matching client CryptoJS AES-CBC behavior
        let encrypted_b64 = crypto::encrypt_aes_cryptojs(plain_payload, &default_key).unwrap();

        // 3. Send to encrypted route: { "data": "<encrypted_b64>" }
        let req_body = serde_json::json!({ "data": encrypted_b64 });
        let req = test::TestRequest::post()
            .uri("/api/v1/user/test-encrypted")
            .insert_header(("x-ip-address", "103.21.244.2"))
            .insert_header(("x-timestamp", "1726056789"))
            .set_json(&req_body)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // 4. Read response: must be encrypted envelope { "body": "<encrypted_b64>" }
        let resp_json: serde_json::Value = test::read_body_json(resp).await;
        assert!(resp_json.get("body").is_some(), "Response must contain 'body' envelope");
        let encrypted_resp = resp_json["body"].as_str().unwrap();

        // 5. Decrypt response using dynamic AES key (secret + ip + timestamp)
        let dynamic_key =
            crypto::derive_dynamic_aes_key(secret, "103.21.244.2", "1726056789", hash_key);
        let decrypted_resp = crypto::decrypt_aes_cryptojs(encrypted_resp, &dynamic_key).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&decrypted_resp).unwrap();
        assert_eq!(parsed["received"]["name"], "ViratUser");
        assert_eq!(parsed["received"]["balance"], 50000);
    }
}
