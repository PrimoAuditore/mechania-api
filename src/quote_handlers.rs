use axum::{extract::Path, response::IntoResponse, Json};
use chrono::{Datelike, Utc};
use http::StatusCode;
use num_traits::ToPrimitive;
use sqlx::types::BigDecimal;
use std::error::Error;
use uuid::Uuid;

use crate::{
    api_structs::{CreateQuoteBody, Quote, Vehicle},
    sql::establish_connection,
    vehicle_handler::check_vehicle_exists,
};

#[axum_macros::debug_handler]
pub async fn get_quote(Path(quote_id): Path<String>) -> impl IntoResponse {
    let quote = get_quote_by_id(quote_id).await;

    return (StatusCode::OK, Json(quote)).into_response();
}

#[axum_macros::debug_handler]
pub async fn create_quote(create_params: Json<CreateQuoteBody>) -> impl IntoResponse {
    // Check if vehicle with specified license plate exists
    let vehicle = check_vehicle_exists(create_params.license_plate.clone()).await;

    if vehicle.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json("Vehicle with specified license plate doesnt exist"),
        )
            .into_response();
    }
    let vehicle = vehicle.unwrap();

    // Calculate monthly price for plan
    let quote: Quote = calculate_price(&create_params.0, &vehicle);

    // Save quote to DB
    let res = create_new_quote(&vehicle, &create_params.0, &quote).await;

    if res.is_err() {
        let err_msg = format!("Error creating quote: {}", res.unwrap_err());
        return (StatusCode::INTERNAL_SERVER_ERROR, err_msg).into_response();
    }

    return (StatusCode::CREATED, Json(quote)).into_response();
}

fn calculate_price(create_params: &CreateQuoteBody, vehicle: &Vehicle) -> Quote {
    let mut price = 15_000.0;

    // Check if vehicle is 5 years old or newer
    let current_year = chrono::Utc::now().year() as u32;
    let limit_year = current_year - 5;

    let parsed_year: u32 = vehicle.clone().year.unwrap().parse().unwrap();

    price *= if parsed_year >= limit_year { 1.1 } else { 1.0 };

    // Vehicle type multiplier
    price *= match vehicle.clone().vehicle_type.unwrap().as_str() {
        "STATION WAGON" => 1.1,
        "AUTOMOVIL" => 1.0,
        _ => {
            let sentry_res = sentry::capture_message(&format!("Received unmanaged vehicle type from external service: license plate: {} vehicle_type: {}",vehicle.clone().license_plate ,vehicle.clone().vehicle_type.unwrap()), sentry::Level::Error);
            println!("sentry_res: {sentry_res}");
            1.0
        }
    };

    // Fuel consumption delta
    price += create_params.fuel_consumption * 0.3;

    // Labour coverage
    let coverage = price * 15.0;
    // IVA
    price *= 1.19;

    // Generate quote uuid
    let id = Uuid::new_v4();
    Quote {
        id: id.to_string(),
        labour_coverage: coverage,
        monthly_cost: price,
    }
}

pub async fn create_new_quote(
    vehicle: &Vehicle,
    quote_params: &CreateQuoteBody,
    quote: &Quote,
) -> Result<(), Box<dyn Error>> {
    let mut conn = establish_connection().await;
    let timestamp = Utc::now().to_rfc3339();
    let datetime: Vec<&str> = timestamp.split(".").collect();
    println!("{}", datetime.get(0).unwrap());

    let res = sqlx::query!(
        r#"insert into Quote(id,license_plate, monthly_price, client_email, fuel_consumption, creation_timestamp, client_name, labour_coverage)
        values (?,?,?,?,?,STR_TO_DATE(?, '%Y-%m-%dT%H:%i:%s'), ?, ?)"#,
        quote.id,
        vehicle.license_plate,
        quote.monthly_cost,
        quote_params.email,
        quote_params.fuel_consumption,
        datetime.get(0).unwrap(),
        quote_params.client_name,
        quote.labour_coverage
        )
    .execute(&mut conn)
    .await?;

    Ok(())
}

pub async fn get_quote_by_id(quote_id: String) -> Quote {
    let mut conn = establish_connection().await;

    struct QuoteIR {
        pub id: String,
        pub monthly_cost: Option<BigDecimal>,
        pub labour_coverage: Option<BigDecimal>,
    }

    let res: Option<QuoteIR> = sqlx::query_as!(
        QuoteIR,
        "select id,monthly_price as monthly_cost,labour_coverage from Quote where id=?",
        quote_id
    )
    .fetch_optional(&mut conn)
    .await
    .unwrap();

    Quote {
        id: res.as_ref().unwrap().id.clone(),
        monthly_cost: res
            .as_ref()
            .unwrap()
            .monthly_cost
            .as_ref()
            .unwrap()
            .to_f32()
            .unwrap(),
        labour_coverage: res
            .as_ref()
            .unwrap()
            .labour_coverage
            .as_ref()
            .unwrap()
            .to_f32()
            .unwrap(),
    }
}
