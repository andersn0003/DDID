use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Default)]
pub struct FilterOptions {
    pub page: Option<usize>,
    pub limit: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateDogSchema {
    pub name: String,
    pub breed: String,
    pub color: String,
    pub location: String,
    pub prooflevel: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateDogSchema {
    pub name: Option<String>,
    pub breed: Option<String>,
    pub color: Option<String>,
    pub location: Option<String>,
    pub prooflevel: Option<i32>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct CreateMerchantSchema {
    pub last_data_hash: String
}
#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateMerchantSchema {
    pub last_data_hash: Option<String>
}
#[derive(Serialize, Deserialize, Debug)]
pub struct CreateMerchantRecordSchema {
    pub id: uuid::Uuid,
    pub merchant_id: uuid::Uuid,                 // Foreign key to merchants(id)
    pub valid_until: i64,
    pub prev_data_hash: String,         // Foreign key to merchants(last_data_hash)
    pub data_record: String, 
}
