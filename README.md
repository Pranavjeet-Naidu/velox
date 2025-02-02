# Velox 

A high-performance URL shortening service built with Rust, Actix-web, and Redis.

[![Rust](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org)
[![Actix-web](https://img.shields.io/badge/actix--web-4.4-blue.svg)](https://actix.rs)
[![Redis](https://img.shields.io/badge/redis-latest-red.svg)](https://redis.io)

## Features

- Fast URL shortening using base62 encoding
- Redis-backed storage for high performance
- RESTful API endpoints
- Health check endpoint
- Input URL validation
- Configurable base URL through environment variables

## Getting Started

### Prerequisites

- Rust (https://www.rust-lang.org/tools/install)
- Redis (https://redis.io/download)

### Installation

1. Clone the repository:
    ```sh
    git clone https://github.com/yourusername/velox-url-shortener.git
    cd velox-url-shortener
    ```

2. Install dependencies:
    ```sh
    cargo build
    ```

3. Start Redis server:
    ```sh
    redis-server
    ```

4. Set the `BASE_URL` environment variable (optional):
    ```sh
    export BASE_URL="http://yourdomain.com"
    ```

5. Run the application:
    ```sh
    cargo run
    ```

### Usage

- **Shorten URL**: Send a POST request to `/shorten` with a JSON body containing the original URL.
    ```sh
    curl -X POST http://localhost:8082/shorten -H "Content-Type: application/json" -d '{"original": "https://example.com"}'
    ```

- **Redirect to Original URL**: Access the shortened URL in your browser or via a GET request.
    ```sh
    curl -X GET http://localhost:8082/{shortened_code}
    ```

- **Health Check**: Send a GET request to `/health` to check the service status.
    ```sh
    curl -X GET http://localhost:8082/health
    ```

### Configuration

- `BASE_URL`: The base URL for the shortened URLs. Default is `http://localhost:8082`.

### License

This project is licensed under the MIT License.
