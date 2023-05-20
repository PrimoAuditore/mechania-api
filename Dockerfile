FROM alexmiron/ubuntu-cdev:22.04_1.1 as build


ENV META_TOKEN=""
ENV REDIS_URL=""

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:11-slim
WORKDIR /app
COPY --from=build /app/target/release/mechania-api ./mechania-api
CMD ["./mechania-api"]
