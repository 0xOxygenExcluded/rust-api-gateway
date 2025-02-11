use std::time::{Duration, SystemTime};
use jsonwebtoken::{encode, Header, EncodingKey};
use hyper::{Client, Request};
use hyper_tls::HttpsConnector;
use serde::{Deserialize, Serialize};


const API_KEY: &'static str = "your_api_key_here";

#[tokio::main]
async fn main() {
    let client = {
        let https = HttpsConnector::new();
        Client::builder().build::<_, hyper::Body>(https)
    };

    let request = Request::builder()
        .method("GET")
        .uri("http://127.0.0.1:3030/hello_service")
        .header("Authorization", API_KEY)
        .body(hyper::Body::empty())
        .expect("Request builder failed.");

    let response = client.request(request).await.expect("Request failed.");
    println!("Response: {:?}", response.status());

    let bytes = hyper::body::to_bytes(response.into_body()).await.expect("Failed to read response.");
    let string = String::from_utf8_lossy(&bytes);
    println!("Response Body: {}", string);
}
