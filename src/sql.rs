use crate::helper_structs::QuoteData;
use sqlx::Connection;
use sqlx::MySqlConnection;
use std::env;

pub async fn establish_connection() -> MySqlConnection {
    let conn = MySqlConnection::connect(&env::var("DATABASE_URL").unwrap()).await;
    conn.unwrap()
}

pub async fn get_quote_by_id(quote_id: &str) -> Option<QuoteData> {
    let mut conn = establish_connection().await;

    let res: Option<QuoteData> = sqlx::query_as!(QuoteData, "select id,client_name, license_plate, monthly_price, client_email, fuel_consumption, DATE_FORMAT(creation_timestamp, '%Y-%m-%dT%TZ') as creation_timestamp  from Quote where id=?", quote_id)
        .fetch_optional(&mut conn)
        .await.unwrap();

    res
}
