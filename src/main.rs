mod api_structs;
mod handlers;
mod helper_structs;
mod plan_handlers;
mod quote_handlers;
mod sql;
mod structs;
mod vehicle_handler;
use plan_handlers::{create_plan_handler, get_plan_by_id_handler};
use quote_handlers::{create_quote, get_quote};
use sentry::integrations::panic::PanicIntegration;
use tower_http::cors::CorsLayer;
use vehicle_handler::{get_vehicle_data, get_vehicle_types, vehicle_manual_creation};

use axum::{
    http::{HeaderValue, Method},
    routing::{get, post},
    Router,
};

#[tokio::main]
async fn main() {
    let _guard = sentry::init((
        "https://7f2f98fe51504514a5ad0eceba3c9d03@o1166558.ingest.sentry.io/4505213614161920",
        sentry::ClientOptions {
            environment: Some(std::env::var("SENTRY_ENVIRONMENT").unwrap().into()),
            release: sentry::release_name!(),
            ..Default::default()
        }
        .add_integration(PanicIntegration::new()),
    ));

    // build our application with a single route
    let app = Router::new()
        .route("/health", get(|| async { "Hello, World!" }))
        .layer(
            CorsLayer::new()
                .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
                .allow_headers([http::header::CONTENT_TYPE])
                .allow_methods([Method::GET, Method::POST]),
        )
        .route("/vehicle", get(get_vehicle_data))
        .layer(
            CorsLayer::new()
                .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
                .allow_headers([http::header::CONTENT_TYPE])
                .allow_methods([Method::GET, Method::POST]),
        )
        .route("/vehicle-type", get(get_vehicle_types))
        .layer(
            CorsLayer::new()
                .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
                .allow_headers([http::header::CONTENT_TYPE])
                .allow_methods([Method::GET, Method::POST]),
        )
        .route("/vehicle/manual", post(vehicle_manual_creation))
        .layer(
            CorsLayer::new()
                .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
                .allow_headers([http::header::CONTENT_TYPE])
                .allow_methods([Method::GET, Method::POST]),
        )
        .route("/quote/:quote_id", get(get_quote))
        .layer(
            CorsLayer::new()
                .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
                .allow_headers([http::header::CONTENT_TYPE])
                .allow_methods([Method::GET, Method::POST]),
        )
        .route("/quote", post(create_quote))
        .layer(
            CorsLayer::new()
                .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
                .allow_headers([http::header::CONTENT_TYPE])
                .allow_methods([Method::GET, Method::POST]),
        )
        .route("/plan", post(create_plan_handler))
        .layer(
            CorsLayer::new()
                .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
                .allow_headers([http::header::CONTENT_TYPE])
                .allow_methods([Method::GET, Method::POST]),
        )
        .route("/plan/:plan_id", get(get_plan_by_id_handler))
        .layer(
            CorsLayer::new()
                .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
                .allow_headers([http::header::CONTENT_TYPE])
                .allow_methods([Method::GET, Method::POST]),
        );

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
