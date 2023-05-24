// Params

use serde::{Deserialize, Serialize};
use sqlx::types::time::OffsetDateTime;

use crate::helper_structs::{PaymentMethod, SignMethod, VehicleDescription};

// Query Params
#[derive(Deserialize)]
pub struct GetVehicleQP {
    pub license_plate: String,
}

// Post Params
#[derive(Deserialize)]
pub struct CreateQuoteBody {
    pub fuel_consumption: f32,
    pub email: String,
    pub license_plate: String,
    pub client_name: String,
}

#[derive(Deserialize, Debug)]
pub struct CreatePlanBody {
    pub quote_id: String,
    pub payment_method: PaymentMethod,
    pub sign_method: SignMethod,
}

#[derive(Deserialize, Debug)]
pub struct ManualVehicleCreation{
    pub make: String,
    pub model: String,
    pub year: String,
    pub license_plate: String,
    pub vehicle_type: String,
}

// Responses
#[derive(Debug, Serialize, Clone)]
pub struct Vehicle {
    pub license_plate: String,
    pub vehicle_type: Option<String>,
    pub vin: Option<String>,
    pub description: Option<String>,
    pub make: Option<String>,
    pub model: Option<String>,
    pub circulation_from: Option<String>,
    pub circulation_to: Option<String>,
    pub engine_code: Option<String>,
    pub fuel: Option<String>,
    pub year: Option<String>,
}

impl Vehicle {
    pub fn from_vehicle_description(
        vehicle_data: VehicleDescription,
        license_plate: String,
    ) -> Self {
        Vehicle {
            license_plate: license_plate,
            vin: Some(vehicle_data.vin),
            fuel: Some(vehicle_data.fuel),
            circulation_from: Some(vehicle_data.valid_since),
            circulation_to: Some(vehicle_data.expiry),
            engine_code: Some(vehicle_data.engine_code),
            make: Some(vehicle_data.make_description.current_text_value),
            model: Some(vehicle_data.model_description.current_text_value),
            description: Some(vehicle_data.description),
            year: Some(vehicle_data.registration_year),
            vehicle_type: Some(vehicle_data.vehicle_type),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Quote {
    pub id: String,
    pub labour_coverage: f32,
    pub monthly_cost: f32,
}

#[derive(Debug, Serialize)]
pub struct Plan {
    pub id: String,
    pub payment_link: Option<String>,
    pub payment_method: PaymentMethod,
    pub sign_method: SignMethod,
}
