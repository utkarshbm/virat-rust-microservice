mod config;
mod constants;
mod routes;
mod state;

use crate::config::GatewayConfig;
use actix_web::{App, HttpMessage, HttpRequest, HttpResponse, HttpServer, Responder, get, post, web};
use logging::{AuditLoggingMiddleware, RequestMetadata, info, logger};

// A sample health route (unencrypted route)
#[get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "UP",
        "service": "api-gateway"
    }))
}

// A ping route demonstrating extracting the metadata our middleware saved!
#[get("/api/v1/ping")]
async fn ping(req: HttpRequest) -> impl Responder {
    // Pull the metadata that AuditLoggingMiddleware stored in req.extensions()!
    let metadata = req.extensions().get::<RequestMetadata>().cloned();

    let logger = logger::Logger::new("ping-service");
    logger.log("ping log");
    logger.warn("ping warn");
    logger.error("ping error");
    logger.debug("ping debug");

    HttpResponse::Ok().json(serde_json::json!({
        "message": "pong",
        "request_id": metadata.as_ref().map(|m| m.request_id.to_string()),
        "client_ip": metadata.as_ref().map(|m| m.client_ip.clone()),
    }))
}

// An encrypted endpoint demonstrating automatic inbound decryption and outbound response encryption!
#[post("/api/v1/user/test-encrypted")]
async fn test_encrypted_endpoint(
    req: HttpRequest,
    body: web::Json<serde_json::Value>,
) -> impl Responder {
    let metadata = req.extensions().get::<RequestMetadata>().cloned();

    HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "received_decrypted_payload": body.into_inner(),
        "request_id": metadata.as_ref().map(|m| m.request_id.to_string()),
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 1. Initialize structured logging
    let _guards = logging::init_tracing("api-gateway");

    // 2. Load environment configuration
    let config = GatewayConfig::load();

    info!(target: "RustFactory", "Starting API Gateway application...");
    info!(target: "InstanceLoader", "AppModule dependencies initialized");
    info!(target: "RoutesResolver", "HealthController {{/health}}");
    info!(target: "RouterExplorer", "Mapped {{/health, GET}} route");
    info!(target: "RouterExplorer", "Mapped {{/api/v1/ping, GET}} route");
    info!(target: "RouterExplorer", "Mapped {{/api/v1/user/test-encrypted, POST}} route");

    // 3. Bind Actix Web HTTP server
    let server = HttpServer::new(|| {
        App::new()
            .wrap(AuditLoggingMiddleware)
            .service(health_check)
            .service(ping)
            .service(test_encrypted_endpoint)
    })
    .bind((config.host.as_str(), config.port))?;

    info!(
        target: "RustApplication",
        env = %config.env,
        "API Gateway application successfully started on port {}",
        config.port
    );

    server.run().await
}
