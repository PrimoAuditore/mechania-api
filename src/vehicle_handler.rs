use crate::api_structs::Vehicle;
use axum::Json;
use axum::{extract::Query, response::IntoResponse};
use http::StatusCode;
use quick_xml::events::Event;
use quick_xml::Reader;
use regex::Regex;
use std::error::Error;

use crate::helper_structs::VehicleDescription;
use crate::{api_structs::GetVehicleQP, sql::establish_connection};

#[axum_macros::debug_handler]
pub async fn get_vehicle_data(query_params: Query<GetVehicleQP>) -> impl IntoResponse {
    // Check if received license plate is in a valid format.
    let rg = Regex::new(r"^[A-Z]{2}[A-Z0-9]{2}\d{2}(\d{2})?$").unwrap();
    let valid = rg.is_match(&query_params.license_plate);

    // Fails if license plate is not valid.
    if !valid {
        let err_msg = format!(
            "License plate '{}' is not a valid license plate",
            &query_params.license_plate
        );
        return (StatusCode::BAD_REQUEST, err_msg).into_response();
    }

    // Check if vehicle with specified license plate is already on db.
    let vehicle = check_vehicle_exists(query_params.license_plate.clone()).await;

    //if vehicle.is_err() {
    //    return (StatusCode::INTERNAL_SERVER_ERROR, "").into_response();
    //}

    //let vehicle = vehicle.unwrap();
    if vehicle.is_none() {
        // Tries to get license plate data
        let vehicle_data = get_vehicle_data_api(query_params.license_plate.clone()).await;

        if vehicle_data.is_err() {
            let err_msg = format!(
                "Couldn't retrieve data for license plate {}",
                query_params.license_plate.clone()
            );
            return (StatusCode::INTERNAL_SERVER_ERROR, err_msg).into_response();
        }

        // Get vehicle object form api data
        let new_vehicle = Vehicle::from_vehicle_description(
            vehicle_data.unwrap(),
            query_params.license_plate.clone(),
        );

        // Insert new vehicle to db.
        let new_vehicle = create_new_vehicle(new_vehicle).await;

        if new_vehicle.is_err() {
            let err_msg = format!(
                "Couldn't save data for license plate {}: {:?}",
                query_params.license_plate.clone(),
                new_vehicle.as_ref().unwrap_err()
            );
            return (StatusCode::INTERNAL_SERVER_ERROR, err_msg).into_response();
        }

        return (StatusCode::CREATED, Json(new_vehicle.unwrap())).into_response();
    }

    return (StatusCode::OK, Json(vehicle.unwrap())).into_response();
}

pub async fn get_vehicle_data_api(
    license_plate: String,
) -> Result<VehicleDescription, (StatusCode, String)> {
    println!("license_plate: {}", license_plate);
    let resp = reqwest::Client::new()
        .get(format!("http://cl.matriculaapi.com/api/reg.asmx/CheckChile?RegistrationNumber={}&username=adminpescara", license_plate))
        .send()
        .await.unwrap();

    println!("resp: {:?}", resp);

    // reqwest to api that returns xml.

    let xml: String = resp.text().await.unwrap();
    //let xml = r#"
    //            <?xml version="1.0" encoding="utf-8"?>
    //    <Vehicle xmlns:xsd="http://www.w3.org/2001/XMLSchema" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns="http://regcheck.org.uk">
    //      <vehicleJson>{
    //      "Description": "TOYOTA RAV4 4X4",
    //      "RegistrationYear": "2011",
    //      "CarMake": {
    //        "CurrentTextValue": "TOYOTA"
    //      },
    //      "CarModel": {
    //        "CurrentTextValue": "RAV4 4X4"
    //      },
    //      "MakeDescription": {
    //        "CurrentTextValue": "TOYOTA"
    //      },
    //      "ModelDescription": {
    //        "CurrentTextValue": "RAV4 4X4"
    //      },
    //      "ImageUrl": "http://cl.matriculaapi.com/image.aspx/@VE9ZT1RBIFJBVjQgNFg0",
    //      "ValidSince": "30-07-2022",
    //      "Expiry": "30-04-2023",
    //      "VehicleType": "STATION WAGON",
    //      "VIN": "JTMBD33V3B5268861",
    //      "EngineCode": "2AZB488919",
    //      "Fuel": "GASOLINA"
    //    }</vehicleJson>
    //      <vehicleData>
    //        <Description>TOYOTA RAV4 4X4</Description>
    //        <RegistrationYear>2011</RegistrationYear>
    //        <CarMake>
    //          <CurrentTextValue>TOYOTA</CurrentTextValue>
    //        </CarMake>
    //        <CarModel>RAV4 4X4</CarModel>
    //        <FuelType>
    //          <CurrentValue>GASOLINA</CurrentValue>
    //        </FuelType>
    //      </vehicleData>
    //    </Vehicle>
    //            "#;
    let mut reader = Reader::from_str(&xml);
    println!("{}", xml);

    loop {
        match reader.read_event().unwrap() {
            Event::Start(e) => {
                let tag_name =
                    String::from_utf8(e.name().local_name().into_inner().to_vec()).unwrap();
                if &tag_name == "vehicleJson" {
                    let content = reader.read_text(e.name()).unwrap();
                    println!("content: {:?}", content);
                    return Ok(serde_json::from_str(&content).unwrap());
                }
            }
            Event::Eof => break,
            _ => (),
        }
    }

    //return Err("No vehicle data found".into());
    sentry::capture_message("XML for vehicle data invalid", sentry::Level::Error);
    return Err((StatusCode::INTERNAL_SERVER_ERROR, String::from("test")));
}

pub async fn check_vehicle_exists(license_plate: String) -> Option<Vehicle> {
    let mut conn = establish_connection().await;

    let res: Option<Vehicle> = sqlx::query_as!(Vehicle,
        "select license_plate, vehicle_type , make, model, registration_year as year, engine_code, DATE_FORMAT(circulation_from, '%Y-%m-%dT%TZ') circulation_from, DATE_FORMAT(circulation_to, '%Y-%m-%dT%TZ') as circulation_to, description, fuel, vin from
                              Vehicle where license_plate = ?",
        license_plate
    )
    .fetch_optional(&mut conn)
    .await.unwrap();

    res
}

pub async fn create_new_vehicle(vehicle: Vehicle) -> Result<Vehicle, Box<dyn Error>> {
    let mut conn = establish_connection().await;

    let res = sqlx::query!(
        r#"insert into Vehicle(license_plate, vin, make, model, registration_year, engine_code, circulation_to, circulation_from,  description, fuel, vehicle_type) values(?,?,?,?,?,?,STR_TO_DATE(?, '%d-%m-%Y'),STR_TO_DATE(?, '%d-%m-%Y'),?,?,?)"#,
        vehicle.license_plate,
        vehicle.vin,
        vehicle.make,
        vehicle.model,
        vehicle.year,
        vehicle.engine_code,
        vehicle.circulation_to,
        vehicle.circulation_from,
        vehicle.description,
        vehicle.fuel,
        vehicle.vehicle_type,
        )
    .execute(&mut conn)
    .await?;

    Ok(vehicle)
}
