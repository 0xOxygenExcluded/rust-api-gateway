# rust-api-gateway
A sample API Gateway built in Rust

This API Gateway is built in Rust and serves as a foundational layer for directing incoming HTTP requests to appropriate services, along with providing several essential features for improving security, observability, and control.

##  Features

- **1. Routing**

    Requests can be routed to different mock services based on the endpoint path.
    Endpoints like /service1 and /service2 are directed to their respective mock handlers.

- **2. Rate Limiting**

    Implemented an IP-based rate limiter.
    Restricts the number of requests from a specific IP address.
    Responds with "Too many requests" if a limit is exceeded.

- **3. Logging**

    Logs incoming requests.
    Displays the source IP address of the requester.

- **4. Request & Response Transformation**

    Requests: Adds a custom header (X-Custom-Header) before forwarding.
    Responses: Appends custom JSON data to the responses from the services.

- **5. Authentication**

    Utilizes API-key authentication.
    Requests must present a valid API-key in the Authorization header.

- **6. HTTPS Client**

    Can forward requests to HTTPS services.

- **7. Error Handling**

    Provides appropriate HTTP status codes and error messages for specific scenarios, e.g., missing routes or authentication failures.

- **8. Dynamic Service Registry**

    Makes it possible to register and deregister services in api-gateway, 

##  Architecture

```
rust-api-gateway/
│
├── .env
├── .gitignore
│
├── client/
│   ├── Cargo.toml
│   └── src/
│       └── main.rs      # sending request script to test hello service
│
├── server/
│   ├── Cargo.toml
│   └── src/
│       └── main.rs      # logic of api-gateway
│
└── helloservice/
    ├── Cargo.toml
    └── src/
        └── main.rs      # test hello service
```

## Quick Start

### Prerequisites
- Rust 1.75 or higher
- Cargo package manager

### Installation

1. Clone the repository:
```bash
git clone https://github.com/miky-rola/api-gateway
cd api-gateway
```

2. Build the server (api-gateway):
```bash
cd server
cargo build --release
```

3. Run the gateway:
```bash
cargo run --release
```

The gateway will start on `http://127.0.0.1:3030`


### Endpoints

#### /register-service - add service name and service address into api-gateway

##### "your_api_key_here" - is value of the test API key (it should be changed to more safe)
##### "some_service_name" - is name of service inner api-gateway
##### "service_address" - url address where service are hosted 
```bash
curl --location 'http://127.0.0.1:3030/register_service' \
--header 'Content-Type: text/plain' \
--header 'Authorization: your_api_key_here \
--data 'some_service_name,service_address'
```

#### Deregister-service - delete service by its name from api-gateway

##### "some_service_name" - is name of service inner api-gateway
```bash
curl --location 'http://127.0.0.1:3030/deregister_service' \
--header 'Content-Type: text/plain' \
--header 'Authorization: your_api_key_here \
--data 'some_service_name'
```

