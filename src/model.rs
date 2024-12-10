use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
#[allow(non_snake_case)]
pub struct DogModel {
    pub id: Uuid,
    pub name: String,
    pub breed: Option<String>,
    pub color: String,
    pub location: Option<String>,
    pub prooflevel: Option<i32>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
#[allow(non_snake_case)]
pub struct MerchantModel {
    pub id: Uuid,
    pub last_data_hash: String,
    pub last_updated: Option<NaiveDateTime>
}
#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
#[allow(non_snake_case)]
pub struct MerchantRecordModel {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub data_issued: Option<NaiveDateTime>,
    pub valid_until: NaiveDateTime,
    pub prev_data_hash: String,
    pub data_record: Option<String>,
}