# Simple Media Server

This is a Rust-based lightweight media server mainly intended for hosting home videos or downloaded YouTube videos.

## Building

Building is easiest using Docker Compose.
```shell
docker compose build
```

## Running

An example configuration is present in the `example` directory.
Place any videos that you want to serve inside of `example/example-library` and start
the server using
```shell
docker compose up
```
You can then navigate to http://localhost:8000/ and login with `test` for both the username and password.
