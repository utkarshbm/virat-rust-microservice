use actix_web::{
    Error, HttpMessage, dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use futures_util::future::LocalBoxFuture;
use std::future::{Ready, ready};
use std::time::Instant;
use uuid::Uuid;
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
}

#[derive(Default, Clone)]
pub struct AuditLoggingMiddleware;

pub struct AuditLoggingMiddlewareService<S> {
    service: S, // <-- The next middleware or route handler!
}

impl<S, B> Transform<S, ServiceRequest> for AuditLoggingMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuditLoggingMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        // Create the worker for this thread, wrapping the inner service!
        ready(Ok(AuditLoggingMiddlewareService { service }))
    }
}

impl<S, B> Service<ServiceRequest> for AuditLoggingMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    // 1. Is this service ready to receive a request?
    forward_ready!(service);

    // 2. Process an incoming HTTP request:
    fn call(&self, req: ServiceRequest) -> Self::Future {
    // 1. START STOPWATCH & GENERATE REQUEST ID
    let start_time = Instant::now();
    let request_id = Uuid::new_v4();

    let path = req.path().to_string();
    let method = req.method().to_string();
    let client_ip = extract_client_ip(&req);
    let user_agent = req.headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
    let auth_token = extract_auth_token(&req);
    let ga_uuid = req.headers()
        .get("ga-uuid")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // 2. SAVE IN EXTENSIONS FOR DOWNSTREAM HANDLERS
    let metadata = RequestMetadata {
        request_id,
        start_time,
        path: path.clone(),
        method: method.clone(),
        client_ip: client_ip.clone(),
        user_agent,
        auth_token,
        ga_uuid,
    };
    req.extensions_mut().insert(metadata);

    // 3. CALL THE NEXT SERVICE IN THE ONION!
    let fut = self.service.call(req);

    // 4. ASYNC POST-PROCESSING WHEN RESPONSE RETURNS
    Box::pin(async move {
        match fut.await {
            Ok(res) => {
                let latency_ms = start_time.elapsed().as_millis();
                let status = res.status().as_u16();

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

                Ok(res)
            }
            Err(err) => {
                let latency_ms = start_time.elapsed().as_millis();

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

                Err(err)
            }
        }
    })
}
}

fn extract_client_ip(req: &ServiceRequest) -> String {
    //true client ip

    if let Some(ip) = req.headers().get("true-client-ip") {
        if let Ok(ip_str) = ip.to_str() {
            return ip_str.to_string();
        }
    }
    //fallback - "x-forwared-for"
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
    // 1. Check "Authorization: Bearer <token>"
    if let Some(auth_val) = req.headers().get("authorization") {
        if let Ok(auth_str) = auth_val.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                return Some(token.to_string());
            }
            return Some(auth_str.to_string());
        }
    }

    // 2. Fallback to query parameter "?access_token=<token>"
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

