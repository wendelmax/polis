use crate::AuthRoutes;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use polis_auth::AuthManager;
use polis_core::{ContainerId, PolisError, Result};
use polis_image::ImageManager;
use polis_runtime::{ContainerRuntime, PolisRuntime};
use serde_json;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct RestServer {
    runtime: Arc<PolisRuntime>,
    image_manager: Arc<ImageManager>,
    auth_manager: Arc<RwLock<AuthManager>>,
}

impl RestServer {
    pub fn new(
        runtime: Arc<PolisRuntime>,
        image_manager: Arc<ImageManager>,
        auth_manager: Arc<RwLock<AuthManager>>,
    ) -> Self {
        Self {
            runtime,
            image_manager,
            auth_manager,
        }
    }

    pub async fn start(&self, port: u16) -> Result<()> {
        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        let runtime = Arc::clone(&self.runtime);
        let image_manager = Arc::clone(&self.image_manager);

        let auth_manager = Arc::clone(&self.auth_manager);
        let make_svc = make_service_fn(move |_conn| {
            let runtime = Arc::clone(&runtime);
            let image_manager = Arc::clone(&image_manager);
            let auth_manager = Arc::clone(&auth_manager);

            async move {
                Ok::<_, Infallible>(service_fn(move |req| {
                    handle_request(
                        req,
                        Arc::clone(&runtime),
                        Arc::clone(&image_manager),
                        Arc::clone(&auth_manager),
                    )
                }))
            }
        });

        let server = Server::bind(&addr).serve(make_svc);

        println!("🌐 API REST iniciada em http://0.0.0.0:{}", port);

        if let Err(e) = server.await {
            return Err(PolisError::Api(format!("Erro no servidor REST: {}", e)));
        }

        Ok(())
    }
}

async fn handle_request(
    req: Request<Body>,
    runtime: Arc<PolisRuntime>,
    image_manager: Arc<ImageManager>,
    auth_manager: Arc<RwLock<AuthManager>>,
) -> std::result::Result<Response<Body>, Infallible> {
    let path = req.uri().path();
    let method = req.method();

    match (method, path) {
        // Authentication endpoints
        (_, path) if path.starts_with("/auth/") => {
            let auth_routes = AuthRoutes::new(auth_manager);
            match auth_routes.handle_request(req).await {
                Ok(response) => Ok(response),
                Err(e) => {
                    let error_response = Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .header("content-type", "application/json")
                        .body(Body::from(format!(r#"{{"error":"{}"}}"#, e)))
                        .unwrap();
                    Ok(error_response)
                }
            }
        }

        // Health check
        (&Method::GET, "/health") => {
            let response = Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .body(Body::from(r#"{"status":"healthy","service":"polis"}"#))
                .unwrap();
            Ok(response)
        }

        // Container endpoints
        (&Method::GET, "/containers") => match runtime.list_containers().await {
            Ok(containers) => {
                let json = serde_json::to_string(&containers).unwrap_or_else(|_| "[]".to_string());
                let response = Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", "application/json")
                    .body(Body::from(json))
                    .unwrap();
                Ok(response)
            }
            Err(e) => {
                let error_response = format!(r#"{{"error":"{}"}}"#, e);
                let response = Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header("content-type", "application/json")
                    .body(Body::from(error_response))
                    .unwrap();
                Ok(response)
            }
        },

        (&Method::POST, "/containers") => {
            // Parse request body for container creation
            let body = hyper::body::to_bytes(req.into_body()).await.unwrap();
            let create_request: serde_json::Value = match serde_json::from_slice(&body) {
                Ok(req) => req,
                Err(_) => {
                    let error_response = r#"{"error":"Invalid JSON"}"#;
                    let response = Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .header("content-type", "application/json")
                        .body(Body::from(error_response))
                        .unwrap();
                    return Ok(response);
                }
            };

            let name = create_request["name"].as_str().unwrap_or("").to_string();
            let image = create_request["image"].as_str().unwrap_or("").to_string();
            let command = create_request["command"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_else(|| vec!["sh".to_string()]);

            match runtime.create_container(name.clone(), image, command).await {
                Ok(container_id) => {
                    let response_data = format!(
                        r#"{{"id":"{}","name":"{}","status":"created"}}"#,
                        container_id.0, name
                    );
                    let response = Response::builder()
                        .status(StatusCode::CREATED)
                        .header("content-type", "application/json")
                        .body(Body::from(response_data))
                        .unwrap();
                    Ok(response)
                }
                Err(e) => {
                    let error_response = format!(r#"{{"error":"{}"}}"#, e);
                    let response = Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .header("content-type", "application/json")
                        .body(Body::from(error_response))
                        .unwrap();
                    Ok(response)
                }
            }
        }

        (&Method::GET, path) if path.starts_with("/containers/") => {
            let container_id_str = &path[12..]; // Remove "/containers/"
            match ContainerId::from_string(container_id_str) {
                Ok(container_id) => match runtime.get_container(container_id).await {
                    Ok(container) => {
                        let json =
                            serde_json::to_string(&container).unwrap_or_else(|_| "{}".to_string());
                        let response = Response::builder()
                            .status(StatusCode::OK)
                            .header("content-type", "application/json")
                            .body(Body::from(json))
                            .unwrap();
                        Ok(response)
                    }
                    Err(_) => {
                        let error_response = r#"{"error":"Container not found"}"#;
                        let response = Response::builder()
                            .status(StatusCode::NOT_FOUND)
                            .header("content-type", "application/json")
                            .body(Body::from(error_response))
                            .unwrap();
                        Ok(response)
                    }
                },
                Err(_) => {
                    let error_response = r#"{"error":"Invalid container ID"}"#;
                    let response = Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .header("content-type", "application/json")
                        .body(Body::from(error_response))
                        .unwrap();
                    Ok(response)
                }
            }
        }

        (&Method::POST, path) if path.starts_with("/containers/") && path.ends_with("/start") => {
            let container_id_str = &path[12..path.len() - 6]; // Remove "/containers/" and "/start"
            match ContainerId::from_string(container_id_str) {
                Ok(container_id) => match runtime.start_container(container_id).await {
                    Ok(_) => {
                        let response_data = r#"{"status":"started"}"#;
                        let response = Response::builder()
                            .status(StatusCode::OK)
                            .header("content-type", "application/json")
                            .body(Body::from(response_data))
                            .unwrap();
                        Ok(response)
                    }
                    Err(e) => {
                        let error_response = format!(r#"{{"error":"{}"}}"#, e);
                        let response = Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .header("content-type", "application/json")
                            .body(Body::from(error_response))
                            .unwrap();
                        Ok(response)
                    }
                },
                Err(_) => {
                    let error_response = r#"{"error":"Invalid container ID"}"#;
                    let response = Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .header("content-type", "application/json")
                        .body(Body::from(error_response))
                        .unwrap();
                    Ok(response)
                }
            }
        }

        (&Method::POST, path) if path.starts_with("/containers/") && path.ends_with("/stop") => {
            let container_id_str = &path[12..path.len() - 5]; // Remove "/containers/" and "/stop"
            match ContainerId::from_string(container_id_str) {
                Ok(container_id) => match runtime.stop_container(container_id).await {
                    Ok(_) => {
                        let response_data = r#"{"status":"stopped"}"#;
                        let response = Response::builder()
                            .status(StatusCode::OK)
                            .header("content-type", "application/json")
                            .body(Body::from(response_data))
                            .unwrap();
                        Ok(response)
                    }
                    Err(e) => {
                        let error_response = format!(r#"{{"error":"{}"}}"#, e);
                        let response = Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .header("content-type", "application/json")
                            .body(Body::from(error_response))
                            .unwrap();
                        Ok(response)
                    }
                },
                Err(_) => {
                    let error_response = r#"{"error":"Invalid container ID"}"#;
                    let response = Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .header("content-type", "application/json")
                        .body(Body::from(error_response))
                        .unwrap();
                    Ok(response)
                }
            }
        }

        // Image endpoints
        (&Method::GET, "/images") => match image_manager.list_images().await {
            Ok(images) => {
                let json = serde_json::to_string(&images).unwrap_or_else(|_| "[]".to_string());
                let response = Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", "application/json")
                    .body(Body::from(json))
                    .unwrap();
                Ok(response)
            }
            Err(e) => {
                let error_response = format!(r#"{{"error":"{}"}}"#, e);
                let response = Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header("content-type", "application/json")
                    .body(Body::from(error_response))
                    .unwrap();
                Ok(response)
            }
        },

        (&Method::POST, "/images/pull") => {
            let body = hyper::body::to_bytes(req.into_body()).await.unwrap();
            let pull_request: serde_json::Value = match serde_json::from_slice(&body) {
                Ok(req) => req,
                Err(_) => {
                    let error_response = r#"{"error":"Invalid JSON"}"#;
                    let response = Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .header("content-type", "application/json")
                        .body(Body::from(error_response))
                        .unwrap();
                    return Ok(response);
                }
            };

            let image_name = pull_request["name"].as_str().unwrap_or("");
            if image_name.is_empty() {
                let error_response = r#"{"error":"Image name is required"}"#;
                let response = Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .header("content-type", "application/json")
                    .body(Body::from(error_response))
                    .unwrap();
                return Ok(response);
            }

            match image_manager.pull(image_name).await {
                Ok(image) => {
                    let json = serde_json::to_string(&image).unwrap_or_else(|_| "{}".to_string());
                    let response = Response::builder()
                        .status(StatusCode::OK)
                        .header("content-type", "application/json")
                        .body(Body::from(json))
                        .unwrap();
                    Ok(response)
                }
                Err(e) => {
                    let error_response = format!(r#"{{"error":"{}"}}"#, e);
                    let response = Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .header("content-type", "application/json")
                        .body(Body::from(error_response))
                        .unwrap();
                    Ok(response)
                }
            }
        }

        // System info
        (&Method::GET, "/system/info") => {
            let system_info = serde_json::json!({
                "service": "polis",
                "version": "0.3.0",
                "runtime": "rust",
                "architecture": std::env::consts::ARCH,
                "os": std::env::consts::OS,
                "containers": "running"
            });
            let json = serde_json::to_string(&system_info).unwrap();
            let response = Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .body(Body::from(json))
                .unwrap();
            Ok(response)
        }

        // 404 for all other routes
        _ => {
            let error_response = r#"{"error":"Not Found"}"#;
            let response = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header("content-type", "application/json")
                .body(Body::from(error_response))
                .unwrap();
            Ok(response)
        }
    }
}
