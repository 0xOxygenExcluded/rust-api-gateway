use hyper::client::HttpConnector;
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use hyper_tls::HttpsConnector;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use dotenvy::dotenv;
use std::env;


#[derive(Debug, Serialize, Deserialize)]
struct ServiceConfig {
    name: String,
    address: String,
}

struct ServiceRegistry {
    services: Arc<RwLock<HashMap<String, String>>>, // Service Name -> Service Address (URL/URI)
}

impl ServiceRegistry {
    fn new() -> Self {
        ServiceRegistry {
            services: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn register(&self, name: String, address: String) {
        let mut services = self.services.write().unwrap();
        services.insert(name, address);
    }

    fn deregister(&self, name: &str) {
        let mut services = self.services.write().unwrap();
        services.remove(name);
    }

    fn get_address(&self, name: &str) -> Option<String> {
        let services = self.services.read().unwrap();
        services.get(name).cloned()
    }
}

async fn register_service(
    req: Request<Body>,
    registry: Arc<ServiceRegistry>,
) -> Result<Response<Body>, hyper::Error> {
    let body_bytes = hyper::body::to_bytes(req.into_body()).await?;
    let body_str = String::from_utf8_lossy(&body_bytes);
    let parts: Vec<&str> = body_str.split(',').collect();

    if parts.len() != 2 {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(
                "Invalid format. Expecting 'name,address'",
            ))
            .unwrap());
    }

    let name = parts[0].to_string();
    let address = parts[1].to_string();

    registry.register(name, address);

    Ok(Response::new(Body::from("Service registered successfully")))
}

async fn deregister_service(
    req: Request<Body>,
    registry: Arc<ServiceRegistry>,
) -> Result<Response<Body>, hyper::Error> {
    let body_bytes = hyper::body::to_bytes(req.into_body()).await?;
    let name = String::from_utf8_lossy(&body_bytes).to_string();

    registry.deregister(&name);

    Ok(Response::new(Body::from(
        "Service deregistered successfully",
    )))
}

struct RateLimiter {
    visitors: Arc<Mutex<HashMap<SocketAddr, u32>>>,
}

impl RateLimiter {
    fn new() -> Self {
        RateLimiter {
            visitors: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn allow(&self, addr: SocketAddr) -> bool {
        dotenv().ok();
        let mut visitors = self.visitors.lock().unwrap();
        let counter = visitors.entry(addr).or_insert(0);

        let number_of_requests: u32 = env::var("NUMBER_OF_REQUESTS").unwrap().parse().unwrap();

        if *counter >= number_of_requests {
            false
        } else {
            *counter += 1;
            true
        }
    }
}

fn authenticate(api_key: &str) -> bool {
    dotenv().ok();
    let api_key_actual = env::var("API_KEY").expect("API_KEY must be set");
    api_key == api_key_actual
}

async fn service_handler(
    req: Request<Body>,
    client: &hyper::Client<HttpsConnector<HttpConnector>>,
) -> Result<Response<Body>, hyper::Error> {
    // Example of request transformation: Adding a custom header
    let req = Request::builder()
        .method(req.method())
        .uri(req.uri())
        .header("X-Custom-Header", "My API Gateway")
        .body(req.into_body())
        .unwrap();

    // Forward the transformed request to the mock service
    println!("Sending request to {}", req.uri());
    let resp = client.request(req).await?;

    // Example of response transformation: Append custom JSON
    let body_bytes = hyper::body::to_bytes(resp.into_body()).await?;
    let data_result: Result<serde_json::Value, _> = serde_json::from_slice(&body_bytes);

    let mut data = match data_result {
        Ok(d) => d,
        Err(_) => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .body(Body::from("Failed to parse upstream response"))
                .unwrap())
        }
    };

    data["custom"] = json!("This data is added by the gateway");
    Ok(Response::new(Body::from(data.to_string())))
}

/*async fn handle_request(req: Request<Body>, rate_limiter: Arc<RateLimiter>, client: Arc<hyper::Client<HttpsConnector<HttpConnector>>>, service_registry: &ServiceRegistry) -> Result<Response<Body>, hyper::Error> {*/
async fn handle_request(
    mut req: Request<Body>,
    remote_addr: SocketAddr,
    rate_limiter: Arc<RateLimiter>,
    client: Arc<hyper::Client<HttpsConnector<HttpConnector>>>,
    registry: Arc<ServiceRegistry>,
) -> Result<Response<Body>, hyper::Error> {
    if !rate_limiter.allow(remote_addr) {
        return Ok(Response::builder()
            .status(StatusCode::TOO_MANY_REQUESTS)
            .body(Body::from("Too many requests"))
            .unwrap());
    }

    println!(
        "Received request from {}:{}",
        remote_addr.ip(),
        remote_addr.port()
    );

    match req.headers().get("Authorization") {
        Some(value) => {
            let api_key = value.to_str().unwrap_or("");
            println!("API Key: {}", api_key);
            if !authenticate(api_key) {
                return Ok(Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .body(Body::from("Unauthorized"))
                    .unwrap());
            }
        }
        None => {
            return Ok(Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(Body::from("Unauthorized"))
                .unwrap());
        }
    }

    let path = req.uri().path();

    // Let's assume the first path segment is the service name.
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() < 2 {
        return Ok(Response::new(Body::from("Invalid request URI")));
    }

    let service_name = parts[1];
    println!("Service Name: {}", service_name);

    let endpoint = parts[2..].join("/");
    println!("Endpoint: {}", endpoint);

    match registry.get_address(service_name) {
        Some(address) => {
            // Here, use the address to forward the request.

            // Create a new URI based on the resolved address
            let mut address = address;
            if !address.starts_with("http://") && !address.starts_with("https://") {
                address = format!("http://{}", address);
            }

            let forward_uri = format!("{}/{}", address.trim_end_matches('/'), endpoint);

            if let Ok(uri) = forward_uri.parse() {
                *req.uri_mut() = uri;
            } else {
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(Body::from("Invalid service URI"))
                    .unwrap());
            }

            // Send the request to the service handler
            service_handler(req, &client).await
        }
        None => return Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Service not found"))
            .unwrap()),
    }
}

async fn router(
    req: Request<Body>,
    remote_addr: SocketAddr,
    rate_limiter: Arc<RateLimiter>,
    client: Arc<hyper::Client<HttpsConnector<HttpConnector>>>,
    registry: Arc<ServiceRegistry>,
) -> Result<Response<Body>, hyper::Error> {
    let path = req.uri().path();

    if path == "/register_service" {
        return register_service(req, Arc::clone(&registry)).await;
    }

    if path == "/deregister_service" {
        return deregister_service(req, Arc::clone(&registry)).await;
    }

    // Handle other requests using the previously defined handler
    handle_request(req, remote_addr, rate_limiter, client, registry).await
}
#[tokio::main]
async fn main() {

    dotenv().ok();

    let api_key = env::var("API_KEY").expect("API_KEY must be set");
    println!("API Key: {}", api_key);

    let rate_limiter = Arc::new(RateLimiter::new());
    let https = HttpsConnector::new();
    let client = hyper::Client::builder().build::<_, hyper::Body>(https);
    let client = Arc::new(client);

    let registry = Arc::new(ServiceRegistry::new());

    // Handle Requests
    let make_svc = make_service_fn(move |conn: &AddrStream| {
        let remote_addr = conn.remote_addr();
        let rate_limiter = Arc::clone(&rate_limiter);
        let client = Arc::clone(&client);
        let registry_clone = Arc::clone(&registry);

        let service = service_fn(move |req| {
            router(
                req,
                remote_addr,
                Arc::clone(&rate_limiter),
                Arc::clone(&client),
                Arc::clone(&registry_clone),
            )
        });

        async { Ok::<_, hyper::Error>(service) }
    });

    let addr = ([127, 0, 0, 1], 3030).into();

    let server = Server::bind(&addr)
        .http1_keepalive(true)
        .http2_keep_alive_timeout(Duration::from_secs(120))
        .serve(make_svc);

    println!("API Gateway running on http://{}", addr);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
