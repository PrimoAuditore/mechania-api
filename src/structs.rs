use num_traits::FromPrimitive;
use redis::{FromRedisValue, RedisResult, Value};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, types::{time::OffsetDateTime, BigDecimal}};

#[derive(Debug, Deserialize, Serialize)]
pub struct TextValue {
    #[serde(rename = "CurrentTextValue")]
    current_text_value: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VehicleDescription {
    #[serde(rename = "Description")]
    pub description: String,
    #[serde(rename = "RegistrationYear")]
    pub registration_year: String,
    #[serde(rename = "CarMake")]
    pub car_make: TextValue,
    #[serde(rename = "CarModel")]
    pub car_model: TextValue,
    #[serde(rename = "MakeDescription")]
    pub make_description: TextValue,
    #[serde(rename = "ModelDescription")]
    pub model_description: TextValue,
    #[serde(rename = "ImageUrl")]
    pub image_url: String,
    #[serde(rename = "ValidSince")]
    pub valid_since: String,
    #[serde(rename = "Expiry")]
    pub expiry: String,
    #[serde(rename = "VehicleType")]
    pub vehicle_type: String,
    #[serde(rename = "VIN")]
    pub vin: String,
    #[serde(rename = "EngineCode")]
    pub engine_code: String,
    #[serde(rename = "Fuel")]
    pub fuel: String,
}

#[derive(Debug, Deserialize)]
pub struct PlanParameters {
    pub year: u32,
    pub vehicle_type: VehicleType,
    pub fuel_consumption: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateResponse {
    pub plan_link: String,
}

#[derive(Debug, Deserialize)]
pub enum VehicleType {
    CityCar,
    Sedan,
    Hatchback,
    SUV,
    Pickup,
    Van,
    Truck,
}


#[derive(Debug, Deserialize, Serialize)]
pub struct ReveniuPlan {
    pub frequency: u32,
    pub cicles: u32,
    pub trial_cicles: u32,
    pub title: String,
    pub description: String,
    pub price: f32,
    pub rut_enterprise_field: bool,
    pub comuna_field: bool,
    pub region_field: bool,
    pub phone_field: bool,
    pub address_field: bool,
    pub street_field: bool,
    pub rsocial_field: bool,
    pub redirect_to: String,
    pub redirect_to_failure: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ReveniuResponse {
    pub id: u32,
    pub created_on: String,
    pub currency: String,
    pub subs_counter: u32,
    pub frequency: String,
    pub slug: String,
    pub active: bool,
    pub price: f32,
    pub title: String,
    pub description: String,
    pub is_custom_link: bool,
    pub is_custom_amount: bool,
    pub custom_amount_min: Option<f32>,
    pub custom_amount_max: Option<f32>,
    pub total_cicles: u32,
    pub rut_enterprise_field: bool,
    pub comuna_field: bool,
    pub region_field: bool,
    pub phone_field: bool,
    pub address_field: bool,
    pub street_field: bool,
    pub rsocial_field: bool,
    pub rut_field: bool,
    pub bday_field: bool,
    pub deliverytimeslot_field: bool,
    pub country_field: bool,
    pub success_message: String,
    pub comments_field: bool,
    pub redirect_to: String,
    pub redirect_to_failure: String,
    pub is_uf: bool,
    pub accepting_new_enrollments: bool,
    pub accepting_new_enrollments_date: Option<String>,
    pub auto_renew: bool,
    pub notify_termination: bool,
    pub coupon: Option<String>,
    pub prefferred_due_day: u32,
    pub is_send_dte: bool,
    pub dte_types: Vec<String>,
}

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct Plan {
    pub id: String,
    pub monthly_price: u32,
}

#[derive(Serialize, Debug, Deserialize, Clone)]

pub struct PlanCreateParams {
    pub price: f32,
    pub assigned_name: String,
    pub license_plate: String,
}

impl Plan {
    pub fn from_values_list(values: Vec<(String, String)>) -> Self {
        let mut plan = Plan {
            id: String::new(),
            monthly_price: 0,
        };

        for (field, value) in values {
            match field.as_str() {
                "id" => plan.id = value,
                "monthly_price" => plan.monthly_price = value.parse::<u32>().unwrap(),
                _ => {}
            }
        }
        dbg!(&plan);

        plan
    }
}

impl FromRedisValue for Plan {
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        // Detect bulk
        println!("v: {:?}", v);
        let bulk = match v {
            Value::Bulk(bulk) => bulk,
            Value::Nil => {
                return Err(redis::RedisError::from((
                    redis::ErrorKind::TypeError,
                    "Not found",
                )));
            }
            _ => {
                return Err(redis::RedisError::from((
                    redis::ErrorKind::TypeError,
                    "Cannot convert to Plan",
                )));
            }
        };

        if bulk.is_empty() {
            println!("error: Cannot convert to Plan: Empty value");
            return Err(redis::RedisError::from((
                redis::ErrorKind::TypeError,
                "Cannot convert to Plan: Empty value",
            )));
        }

        // Get fields and its values

        let mut has_error = false;

        let x = bulk
            .chunks(2)
            .map(|value| match (value[0].clone(), value[1].clone()) {
                (Value::Data(field), Value::Data(value)) => {
                    let field = String::from_utf8(field).unwrap();
                    let value = String::from_utf8(value).unwrap();

                    println!("field: {}, value: {}", field, value);
                    let u: Result<(String, String), redis::RedisError> = match field.as_str() {
                        "id" | "monthly_price" => Ok((field, value)),
                        _ => {
                            has_error = true;
                            println!("error: Cannot convert to Plan: Unexpected field {}", field);
                            Err(redis::RedisError::from((
                                redis::ErrorKind::TypeError,
                                "Cannot convert to Plan: Unexpected field",
                            )))
                        }
                    };
                    u
                }
                _ => {
                    has_error = true;
                    println!("error: Cannot convert to Plan: Invalid value type");
                    return Err(redis::RedisError::from((
                        redis::ErrorKind::TypeError,
                        "Cannot convert to Plan: Invalid value type",
                    )));
                }
            })
            .collect::<Vec<Result<(String, String), redis::RedisError>>>();

        // If no errors return plan
        if has_error {
            return Err(redis::RedisError::from((
                redis::ErrorKind::TypeError,
                "error",
            )));
        }

        let ok_values = x
            .iter()
            .filter(|x| x.is_ok())
            .map(|x| (x.as_ref().unwrap().0.clone(), x.as_ref().unwrap().1.clone()))
            .collect::<Vec<(String, String)>>();

        let plan = Plan::from_values_list(ok_values);
        Ok(plan)
    }
}

// SQL

#[derive(Debug)]
pub struct Vehicle {
    pub license_plate: String,
    pub vin: Option<String>,
    pub description: Option<String>,
    pub make: Option<String>,
    pub model: Option<String>,
    pub circulation_from: Option<OffsetDateTime>,
    pub circulation_to: Option<OffsetDateTime>,
    pub engine_code: Option<String>,
    pub fuel: Option<String>,
    pub year: Option<String>
}
