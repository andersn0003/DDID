use std::sync::Arc;
use chrono::Utc;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;

use crate::{
    model::MerchantRecordModel,
    schema::{CreateMerchantRecordSchema, FilterOptions},
    AppState,
};

pub async fn create_merchant_record_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<CreateMerchantRecordSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let valid_until = Utc::now().naive_utc() + chrono::Duration::days( body.valid_until);
    let query_result = sqlx::query_as::<_, MerchantRecordModel>(
        r#"INSERT INTO merchantsrecord (merchant_id, valid_until, prev_data_hash, data_record) VALUES ($1, $2, $3, $4) RETURNING *"#
    )
    .bind(body.merchant_id)
    .bind(valid_until)
    .bind(body.prev_data_hash)
    .bind(body.data_record)
    .fetch_one(&data.db)
    .await;

    match query_result {
        Ok(merchant_record) => {
            let merchant_record_response = json!({"status": "success","data": json!({
                "merchant_record": merchant_record
            })});

            return Ok((StatusCode::CREATED, Json(merchant_record_response)));
        }
        Err(e) => {
            if e.to_string()
                .contains("duplicate key value violates unique constraint")
            {
                let error_response = serde_json::json!({
                    "status": "fail",
                    "message": "Merchant with that title already exists",
                });
                return Err((StatusCode::CONFLICT, Json(error_response)));
            }
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error","message": format!("{:?}", e)})),
            ));
        }
    }
}

pub async fn merchant_record_list_handler(
    Path(merchant_id): Path<uuid::Uuid>,
    opts: Option<Query<FilterOptions>>, State(data): State<Arc<AppState>>
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let Query(opts) = opts.unwrap_or_default();
    
    let limit = opts.limit.unwrap_or(10);
    let offset = (opts.page.unwrap_or(1) - 1) * limit;
    let query_result = sqlx::query_as::<_, MerchantRecordModel>(
        r#"SELECT * FROM merchantsrecord WHERE merchant_id = $3 ORDER by id LIMIT $1 OFFSET $2"#
    )
    .bind(limit as i32)
    .bind(offset as i32)
    .bind(merchant_id)
    .fetch_all(&data.db)
    .await;
    println!("result==========> {:?}", query_result);
    if query_result.is_err() {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": "Something bad happened while fetching all note items",
        });
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
    }

    let merchant_records = query_result.unwrap();

    let json_response = serde_json::json!({
        "status": "success",
        "results": merchant_records.len(),
        "merchant records": merchant_records
    });
    Ok(Json(json_response))
}

pub async fn get_merchant_record_handler(
    Path(merchant_record_id): Path<uuid::Uuid>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_result = sqlx::query_as::<_, MerchantRecordModel>( r#"SELECT * FROM merchantsrecord WHERE id = $1"#)
        .bind(merchant_record_id)
        .fetch_one(&data.db)
        .await;

    match query_result {
        Ok(merchant_record) => {
            let merchant_record_response = serde_json::json!({"status": "success","data": serde_json::json!({
                "merchant_record": merchant_record
            })});

            return Ok(Json(merchant_record_response));
        }
        Err(_) => {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Merchant Record with ID: {} not found", merchant_record_id)
            });
            return Err((StatusCode::NOT_FOUND, Json(error_response)));
        }
    }
}

pub async fn delete_merchant_record_handler(
    Path(merchant_record_id): Path<uuid::Uuid>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let rows_affected = sqlx::query(r#"DELETE FROM merchantsrecord  WHERE id = $1"#)
    .bind(merchant_record_id)
        .execute(&data.db)
        .await
        .unwrap()
        .rows_affected();

    if rows_affected == 0 {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": format!("Merchant Record with ID: {} not found", merchant_record_id)
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    Ok(StatusCode::NO_CONTENT)
}