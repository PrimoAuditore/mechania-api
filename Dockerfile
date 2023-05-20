FROM rust:slim as build

ARG DATABASE_URL
RUN echo ${DATABASE_URL}

RUN apt update && apt install -y libssl-dev pkg-config

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:11-slim
WORKDIR /app
COPY --from=build /app/target/release/mechania-api ./mechania-api
CMD ["./mechania-api"]
