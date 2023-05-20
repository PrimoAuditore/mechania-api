use crate::structs::*;
use axum::{extract::Path, response::IntoResponse, Json};
use chrono::Datelike;
use http::StatusCode;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use redis::Commands;
use redis::JsonCommands;
use redis::RedisResult;
use redis::Value;
use serde_json::json;
use std::ops::Range;
use std::{collections::HashMap, error::Error};
use uuid::Uuid;
//use crate::sql::create_vehicle;

pub async fn test_create() {
    //let res = create_vehicle().await;
}

pub async fn get_vehicle_data(Path(license_plate): Path<String>) -> impl IntoResponse {
    // Check if vehicle data for license plate already exists

    let vehicle_description = get_vehicle_data_api(&license_plate).await.unwrap();
    let redis_client = redis::Client::open("redis://default:@216.238.73.63:30001").unwrap();
    let mut con = redis_client.get_connection().unwrap();

    let key = format!("vehicle_data:{}", &license_plate);
    let res: RedisResult<Value> = con.json_set(
        key,
        "$",
        &serde_json::to_string(&vehicle_description).unwrap(),
    );

    if res.is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": res.as_ref().unwrap_err().to_string()})),
        );
    };

    (StatusCode::OK, Json(json!(&vehicle_description)))
}

async fn get_vehicle_data_api(license_plate: &str) -> Result<VehicleDescription, Box<dyn Error>> {
    println!("license_plate: {}", license_plate);
    //let resp = reqwest::Client::new()
    //    .get(format!("http://cl.matriculaapi.com/api/reg.asmx/CheckChile?RegistrationNumber={}&username=adminpescara", license_plate))
    //    .send()
    //    .await?;

    //println!("resp: {:?}", resp);

    //// reqwest to api that returns xml.

    //let xml: String = resp.text().await?;
    let xml = r#"
        <?xml version="1.0" encoding="utf-8"?>
<Vehicle xmlns:xsd="http://www.w3.org/2001/XMLSchema" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns="http://regcheck.org.uk">
  <vehicleJson>{
  "Description": "TOYOTA RAV4 4X4",
  "RegistrationYear": "2011",
  "CarMake": {
    "CurrentTextValue": "TOYOTA"
  },
  "CarModel": {
    "CurrentTextValue": "RAV4 4X4"
  },
  "MakeDescription": {
    "CurrentTextValue": "TOYOTA"
  },
  "ModelDescription": {
    "CurrentTextValue": "RAV4 4X4"
  },
  "ImageUrl": "http://cl.matriculaapi.com/image.aspx/@VE9ZT1RBIFJBVjQgNFg0",
  "ValidSince": "30-07-2022",
  "Expiry": "30-04-2023",
  "VehicleType": "STATION WAGON",
  "VIN": "JTMBD33V3B5268861",
  "EngineCode": "2AZB488919",
  "Fuel": "GASOLINA"
}</vehicleJson>
  <vehicleData>
    <Description>TOYOTA RAV4 4X4</Description>
    <RegistrationYear>2011</RegistrationYear>
    <CarMake>
      <CurrentTextValue>TOYOTA</CurrentTextValue>
    </CarMake>
    <CarModel>RAV4 4X4</CarModel>
    <FuelType>
      <CurrentValue>GASOLINA</CurrentValue>
    </FuelType>
  </vehicleData>
</Vehicle>
        "#;
    let mut reader = Reader::from_str(&xml);
    println!("{}", xml);

    loop {
        match reader.read_event().unwrap() {
            Event::Start(e) => {
                let tag_name = String::from_utf8(e.name().local_name().into_inner().to_vec())?;
                if &tag_name == "vehicleJson" {
                    let content = reader.read_text(e.name())?;
                    println!("content: {:?}", content);
                    return Ok(serde_json::from_str(&content)?);
                }
            }
            Event::Eof => break,
            _ => (),
        }
    }

    return Err("No vehicle data found".into());
}

pub async fn get_plan(Path(plan_quote_id): Path<String>) -> impl IntoResponse {
    let redis_client = redis::Client::open("redis://default:@216.238.73.63:30001").unwrap();
    let mut con = redis_client.get_connection().unwrap();
    let plan: RedisResult<Plan> = con.hgetall(format!("plan-quote:{}", plan_quote_id));

    if plan.is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": plan.as_ref().unwrap_err().to_string()})),
        );
    };

    (StatusCode::OK, Json(json!(plan.unwrap())))
}

fn calculate_price(plan_parameters: PlanParameters) -> Result<Plan, Box<dyn Error>> {
    let mut price = 15_000.0;

    // Check if vehicle is 5 years old or newer
    let current_year = chrono::Utc::now().year() as u32;
    let limit_year = current_year - 5;

    price *= if plan_parameters.year >= limit_year {
        1.1
    } else {
        1.0
    };

    // Vehicle type multiplier
    price *= match plan_parameters.vehicle_type {
        VehicleType::CityCar | VehicleType::Hatchback | VehicleType::Sedan => 1.0,
        VehicleType::SUV | VehicleType::Pickup | VehicleType::Van => 1.1,
        VehicleType::Truck => 1.2,
    };

    // Fuel consumption delta
    price += plan_parameters.fuel_consumption as f32 * 0.3;

    // IVA
    price *= 1.19;

    // Round price
    price = price.ceil();

    // Generate plan quoting uuid
    let id = Uuid::new_v4();

    let redis_client = redis::Client::open("redis://default:@216.238.73.63:30001").unwrap();
    let mut con = redis_client.get_connection().unwrap();

    let key = format!("plan-quote:{}", id);
    let res: RedisResult<String> = con.hset_multiple(
        key,
        &[("monthly_price", price.to_string()), ("id", id.to_string())],
    );

    // round to next entire number
    Ok(Plan {
        monthly_price: price as u32,
        id: id.to_string(),
    })
}
pub async fn calculate_plan_handler(
    Json(plan_parameters): Json<PlanParameters>,
) -> impl IntoResponse {
    // get current year

    let current_date = chrono::Utc::now();
    let year = current_date.year();

    // year contrains
    if plan_parameters.year < 1970 || plan_parameters.year > year as u32 {}

    let plan = calculate_price(plan_parameters);

    if plan.is_err() {}

    (StatusCode::OK, Json(json!(plan.unwrap())))
}

async fn reveniu_request(
    create_params: &PlanCreateParams,
) -> Result<ReveniuResponse, Box<dyn Error>> {
    println!("create_params: {:?}", create_params);
    let body: ReveniuPlan = ReveniuPlan {
        frequency: 3,
        cicles: 12,
        trial_cicles: 0,
        title: "Plan Mechania - Anual".to_string(),
        description: format!("Plan mensual para {}", create_params.assigned_name),
        price: create_params.price,
        rut_enterprise_field: true,
        comuna_field: true,
        region_field: true,
        phone_field: true,
        address_field: true,
        street_field: true,
        rsocial_field: true,
        redirect_to: "".to_string(),
        redirect_to_failure: "".to_string(),
    };

    let resp = reqwest::Client::new()
        .post("https://integration.reveniu.com/api/v1/plans/")
        .header("content-type", "application/json")
        .header("reveniu-secret-key", std::env::var("REVENIU_API_KEY").unwrap())
        .json(&body)
        .send()
        .await?;

    println!("resp: {:?}", resp);

    let resp = resp.json::<ReveniuResponse>().await?;

    Ok(resp)
}

pub async fn create_plan(
    Path(plan_quote_id): Path<String>,
    Json(create_params): Json<PlanCreateParams>,
) -> impl IntoResponse {
    let redis_client = redis::Client::open("redis://default:@216.238.73.63:30001").unwrap();
    let mut con = redis_client.get_connection().unwrap();

    // Check if quote exists
    let plan: RedisResult<Plan> = con.hgetall(format!("plan-quote:{}", plan_quote_id));

    if plan.is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": plan.as_ref().unwrap_err().to_string()})),
        );
    };

    let plan: Plan = plan.unwrap();

    // reveniu create plan
    let reveniu_res = reveniu_request(&create_params).await;

    if reveniu_res.is_err() {
        println!(
            "Error creating plan: {:?}",
            reveniu_res.as_ref().unwrap_err()
        );
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": reveniu_res.as_ref().unwrap_err().to_string()})),
        );
    };

    let reveniu_res = reveniu_res.unwrap();

    let key = format!("plan:{}", &plan_quote_id);
    let res: RedisResult<Value> = con.hset_multiple(
        key,
        &[
            ("vehicle_license_plate", &create_params.license_plate),
            ("reveniu_plan_id", &reveniu_res.id.to_string()),
        ],
    );
    let plan_link = format!(
        "https://sandbox.reveniu.com/checkout-custom-link/{}",
        reveniu_res.slug
    );

    let create_response = CreateResponse { plan_link };

    (StatusCode::OK, Json(json!(create_response)))
}
