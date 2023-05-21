FROM rust:slim as build

ARG DATABASE_URL
RUN echo ${DATABASE_URL}

RUN apt update && apt install -y libssl-dev pkg-config

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release


WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:11-slim
WORKDIR /app
COPY --from=build /app/target/release/mechania-api ./mechania-api
CMD ["./mechania-api"]
