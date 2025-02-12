# rust-api-gateway
A sample API Gateway built in Rust

This API Gateway is built in Rust and serves as a foundational layer for directing incoming HTTP requests to appropriate services, along with providing several essential features for improving security, observability, and control.

Features

1. Routing

    Requests can be routed to different mock services based on the endpoint path.
    Endpoints like /service1 and /service2 are directed to their respective mock handlers.

2. Rate Limiting

    Implemented an IP-based rate limiter.
    Restricts the number of requests from a specific IP address.
    Responds with "Too many requests" if a limit is exceeded.

3. Logging

    Logs incoming requests.
    Displays the source IP address of the requester.

4. Request & Response Transformation

    Requests: Adds a custom header (X-Custom-Header) before forwarding.
    Responses: Appends custom JSON data to the responses from the services.

5. Authentication

    Utilizes API-key authentication.
    Requests must present a valid API-key in the Authorization header.

6. HTTPS Client

    Can forward requests to HTTPS services.

7. Error Handling

    Provides appropriate HTTP status codes and error messages for specific scenarios, e.g., missing routes or authentication failures.

8. Dynamic Service Registry

Setup and Installation - Please, read the articles listed above

Check out some interesting hands-on Rust articles:

🌟 [Implementing a Network Traffic Analyzer] (https://medium.com/@luishrsoares/implementing-a-network-traffic-analyzer-in-rust-50a772bb6564) 
Ever wondered about the data packets zooming through your network? Unravel their mysteries with this deep dive into network analysis.

🌟 [Building an Application Container in Rust] (https://medium.com/@luishrsoares/implementing-an-application-container-in-rust-3bdde7531ae0) 
Join us in creating a lightweight, performant, and secure container from scratch! Docker’s got nothing on this. 😉

Stay with us for the next parts, where we’ll uncover even more exciting features and delve deeper into the vast world of microservices with Rust.

Happy coding, and keep those Rust gears turning! 🦀

Read more articles about Rust in my Rust Programming Library!

Visit my Blog for more articles, news, and software engineering stuff!

Follow me on Medium, LinkedIn, and Twitter.

Leave a comment, and drop me a message!

All the best,

Luis Soares

CTO | Tech Lead | Senior Software Engineer | Cloud Solutions Architect | Rust 🦀 | Golang | Java | ML AI & Statistics | Web3 & Blockchain
