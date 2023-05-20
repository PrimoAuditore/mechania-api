use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use sqlx::types::BigDecimal;

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

#[derive(Debug, Deserialize, Serialize)]
pub struct TextValue {
    #[serde(rename = "CurrentTextValue")]
    pub current_text_value: String,
}

#[derive(Debug, Clone)]
pub enum PaymentMethod {
    Cash = 0,
    Wiring = 1,
    CreditCard = 2,
    DebitCard = 3,
}

impl PaymentMethod {
    pub fn value(&self) -> u8 {
        match self {
            Self::Cash => 0,
            Self::Wiring => 1,
            Self::CreditCard => 2,
            Self::DebitCard => 3,
        }
    }

    pub fn from_u8(val: u8) -> Result<PaymentMethod, &'static str> {
        PaymentMethod::try_from(val)
    }
}

impl TryFrom<u8> for PaymentMethod {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(PaymentMethod::Cash),
            1 => Ok(PaymentMethod::Wiring),
            2 => Ok(PaymentMethod::CreditCard),
            3 => Ok(PaymentMethod::DebitCard),
            _ => Err("Value not defined for a payment method"),
        }
    }
}

impl<'se> serde::Serialize for PaymentMethod {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(self.value())
    }
}

impl<'de> serde::Deserialize<'de> for PaymentMethod {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct PaymentVisitor;

        impl<'de> serde::de::Visitor<'de> for PaymentVisitor {
            type Value = PaymentMethod;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(
                    formatter,
                    "an integer or string representing a PaymentMethod"
                )
            }

            fn visit_u64<E: serde::de::Error>(self, n: u64) -> Result<PaymentMethod, E> {
                Ok(match n {
                    0 => PaymentMethod::Cash,
                    1 => PaymentMethod::Wiring,
                    2 => PaymentMethod::CreditCard,
                    3 => PaymentMethod::DebitCard,
                    _ => return Err(E::invalid_value(serde::de::Unexpected::Unsigned(n), &self)),
                })
            }
        }

        deserializer.deserialize_any(PaymentVisitor)
    }
}

#[derive(Debug, Clone)]
pub enum SignMethod {
    Deferred = 0,
    Digital = 1,
}

impl SignMethod {
    pub fn value(&self) -> u8 {
        match self {
            Self::Deferred => 0,
            Self::Digital => 1,
        }
    }
    pub fn from_u8(val: u8) -> Result<SignMethod, &'static str> {
        SignMethod::try_from(val)
    }
}

impl TryFrom<u8> for SignMethod {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Deferred),
            1 => Ok(Self::Digital),
            _ => Err("Value not defined for a sign method"),
        }
    }
}

impl<'se> serde::Serialize for SignMethod {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(self.value())
    }
}

impl<'de> serde::Deserialize<'de> for SignMethod {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct SignVisitor;

        impl<'de> serde::de::Visitor<'de> for SignVisitor {
            type Value = SignMethod;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "an integer or string representing a SignMethod")
            }

            fn visit_u64<E: serde::de::Error>(self, n: u64) -> Result<SignMethod, E> {
                Ok(match n {
                    0 => SignMethod::Deferred,
                    1 => SignMethod::Digital,
                    _ => return Err(E::invalid_value(serde::de::Unexpected::Unsigned(n), &self)),
                })
            }
        }

        deserializer.deserialize_any(SignVisitor)
    }
}

// SQL

#[derive(Debug, Clone)]
pub struct QuoteData {
    pub id: String,
    pub license_plate: Option<String>,
    pub monthly_price: Option<BigDecimal>,
    pub client_email: Option<String>,
    pub fuel_consumption: Option<BigDecimal>,
    pub creation_timestamp: Option<String>,
    pub client_name: Option<String>,
}

#[derive(Debug)]
pub struct SignData {
    pub id: String,
    pub sign_link: Option<String>,
    pub creation_timestamp: String,
    pub sign_method: u32,
    pub verified: bool,
}

#[derive(Debug)]
pub struct PlanData {
    pub id: String,
    pub quote_id: String,
    pub vehicle: String,
    pub sign: String,
    pub creation_timestamp: String,
    pub active: bool,
    pub client_email: String,
    pub reveniu_id: Option<String>,
    pub payment_link: Option<String>,
    pub payment_method: u8,
}
