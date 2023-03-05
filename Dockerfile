FROM rust:1.67.1 as build
RUN apt update && apt -y install nodejs npm && rustup target add wasm32-unknown-unknown && cargo install trunk
WORKDIR /build
COPY *.html Cargo.* *.css .
COPY src src
COPY node/package* node/main.js node/
WORKDIR /build/node
RUN npm install
WORKDIR /build
RUN /usr/local/cargo/bin/trunk build --release
FROM lipanski/docker-static-website:latest
COPY --from=build /build/dist .