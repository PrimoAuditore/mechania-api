use axum::{extract::Path, response::IntoResponse, Json};
use chrono::Utc;
use http::StatusCode;
use num_traits::ToPrimitive;
use std::env;
use std::error::Error;
use uuid::Uuid;

use crate::{
    api_structs::{CreatePlanBody, Plan},
    helper_structs::{PaymentMethod, PlanData, QuoteData, SignData, SignMethod},
    sql::{establish_connection, get_quote_by_id},
    structs::{ReveniuPlan, ReveniuResponse},
};

#[axum_macros::debug_handler]
pub async fn create_plan_handler(plan: Json<CreatePlanBody>) -> impl IntoResponse {
    // Get quote by quote id
    let quote: Option<QuoteData> = get_quote_by_id(&plan.quote_id).await;

    if quote.is_none() {
        //return (StatusCode::NOT_FOUND, "Specified quote doesn't exist".to_string()).into_response();
    }

    let quote = quote.unwrap();

    // Create reveniu plan
    let reveniu_plan = match plan.payment_method {
        PaymentMethod::CreditCard | PaymentMethod::DebitCard => {
            Some(create_reveniu_plan(&quote).await.unwrap())
        }
        _ => None,
    };

    // Create new sign
    let sign = create_sign(&reveniu_plan, &plan.sign_method).await;

    // TODO: Generate contract and send it to client

    //if sign.is_err() {}

    //let sign = sign.unwrap();

    // Create plan
    let (reveniu_id, payment_link) = match &reveniu_plan {
        Some(plan) => (
            Some(plan.id.to_string()),
            Some(format!(
                "{}/checkout-custom-link/{}",
                env::var("REVENIU_HOST").unwrap(),
                reveniu_plan.as_ref().unwrap().slug.clone()
            )),
        ),

        None => (None, None),
    };
    let plan_res = create_plan(
        &quote,
        &sign,
        reveniu_id,
        payment_link.clone(),
        &plan.payment_method,
    )
    .await;

    //if plan_res.is_err() {}

    let plan = Plan {
        id: plan_res.id,
        payment_link,
        payment_method: plan.payment_method.clone(),
        sign_method: plan.sign_method.clone(),
    };

    (StatusCode::CREATED, Json(plan))
}

async fn create_plan(
    quote: &QuoteData,
    sign: &SignData,
    reveniu_id: Option<String>,
    payment_link: Option<String>,
    payment_method: &PaymentMethod,
) -> PlanData {
    let mut conn = establish_connection().await;

    let id = Uuid::new_v4().to_string();
    let timestamp = Utc::now().to_rfc3339();
    let datetime: Vec<&str> = timestamp.split(".").collect();

    let plan = PlanData {
        id,
        quote_id: quote.id.clone(),
        client_email: quote.client_email.as_ref().unwrap().clone(),
        vehicle: quote.license_plate.as_ref().unwrap().clone(),
        sign: sign.id.clone(),
        creation_timestamp: datetime.get(0).unwrap(),
        active: false,
        reveniu_id,
        payment_link,
        payment_method: payment_method.value(),
    };

    let res = sqlx::query!(
        r#"insert into Plan(id,quote_id, client_email, vehicle, sign, creation_timestamp, active, reveniu_id, payment_link, payment_method)
                           values (?,?,?,?,?,STR_TO_DATE(?, '%Y-%m-%dT%H:%i:%s.%f+00:00'),?,?,?,?)"#,
        plan.id,
        plan.quote_id,
        plan.client_email,
        plan.vehicle,
        plan.sign,
        plan.creation_timestamp,
        plan.active,
        plan.reveniu_id,
        plan.payment_link,
        plan.payment_method
    )
    .execute(&mut conn)
    .await
    .unwrap();

    plan
}

#[axum_macros::debug_handler]
pub async fn get_plan_by_id_handler(Path(plan_id): Path<String>) -> impl IntoResponse {
    let plan_res = get_plan_by_id(&plan_id).await;

    match plan_res {
        Some(plan) => (StatusCode::OK, Json(plan)).into_response(),
        None => (StatusCode::NOT_FOUND, Json(String::from("Plan not found"))).into_response(),
    }
}

async fn get_plan_by_id(plan_id: &str) -> Option<Plan> {
    struct Data {
        pub plan_id: String,
        pub payment_link: Option<String>,
        pub sign_method: i8,
        pub payment_method: i16,
    }
    let mut conn = establish_connection().await;

    let res = sqlx::query_as!(
        Data,
        "select P.id as plan_id, P.payment_link, S.sign_method, P.payment_method from Plan P join Sign S on P.sign=S.id where P.id=?",
        plan_id
    )
    .fetch_optional(&mut conn)
    .await
    .unwrap();

    if res.is_none() {
        return None;
    };

    let res = res.unwrap();
    let pm = PaymentMethod::from_u8(res.payment_method as u8).unwrap();
    let sm = SignMethod::from_u8(res.sign_method as u8).unwrap();

    Some(Plan {
        id: res.plan_id,
        payment_link: res.payment_link,
        payment_method: pm,
        sign_method: sm,
    })
}

async fn create_sign(reveniu_plan: &Option<ReveniuResponse>, sign_method: &SignMethod) -> SignData {
    let id = Uuid::new_v4().to_string();

    let sign_link = None;
    let sign_method_parsed = sign_method.value() as u32;
    let creation_timestamp = Utc::now().to_rfc3339();

    let sign = SignData {
        id,
        sign_link,
        sign_method: sign_method_parsed,
        creation_timestamp,
        verified: false,
    };

    let mut conn = establish_connection().await;

    let res = sqlx::query!(
        r#"insert into Sign(id,sign_link, sign_method, creation_timestamp, verified)
        values (?,?,?,STR_TO_DATE(?, '%Y-%m-%dT%H:%i:%s.%f+00:00'), ?)"#,
        sign.id,
        sign.sign_link,
        sign.sign_method,
        sign.creation_timestamp,
        sign.verified
    )
    .execute(&mut conn)
    .await
    .unwrap();

    sign
}

async fn create_reveniu_plan(quote: &QuoteData) -> Result<ReveniuResponse, Box<dyn Error>> {
    let price = quote.monthly_price.clone().unwrap().to_f32().unwrap();
    let client_name = quote.client_name.clone().unwrap();

    let body: ReveniuPlan = ReveniuPlan {
        frequency: 3,
        cicles: 12,
        trial_cicles: 0,
        title: "Plan Mechania - Anual".to_string(),
        description: format!("Plan mensual para {}", client_name),
        price: price,
        rut_enterprise_field: true,
        comuna_field: true,
        region_field: true,
        phone_field: true,
        address_field: true,
        street_field: true,
        rsocial_field: true,
        redirect_to: "https://k8s.pescarauto.cl/mechania".to_string(),
        redirect_to_failure: "https://www.google.com".to_string(),
    };

    let resp = reqwest::Client::new()
        .post("https://integration.reveniu.com/api/v1/plans/")
        .header("content-type", "application/json")
        .header(
            "reveniu-secret-key",
            std::env::var("REVENIU_API_KEY").unwrap(),
        )
        .json(&body)
        .send()
        .await?;

    println!("resp: {:?}", resp);

    let resp = resp.json::<ReveniuResponse>().await?;

    Ok(resp)
}
