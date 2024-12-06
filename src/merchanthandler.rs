use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;

use crate::{
    model::MerchantModel,
    schema::{CreateMerchantSchema, FilterOptions, UpdateMerchantSchema},
    AppState,
};


pub async fn merchant_list_handler(
    opts: Option<Query<FilterOptions>>, State(data): State<Arc<AppState>>
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let Query(opts) = opts.unwrap_or_default();
    
    let limit = opts.limit.unwrap_or(10);
    let offset = (opts.page.unwrap_or(1) - 1) * limit;
    let query_result = sqlx::query_as::<_, MerchantModel>(
        r#"SELECT * FROM merchants ORDER by id LIMIT $1 OFFSET $2"#
    )
    .bind(limit as i32)
    .bind(offset as i32)
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

    let merchants = query_result.unwrap();

    let json_response = serde_json::json!({
        "status": "success",
        "results": merchants.len(),
        "merchants": merchants
    });
    Ok(Json(json_response))
}

pub async fn create_merchant_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<CreateMerchantSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_result = sqlx::query_as::<_, MerchantModel>(
        r#"INSERT INTO merchants (last_data_hash) VALUES ($1) RETURNING *"#
    )
    .bind(body.last_data_hash)
    .fetch_one(&data.db)
    .await;

    match query_result {
        Ok(merchant) => {
            let merchant_response = json!({"status": "success","data": json!({
                "merchant": merchant
            })});

            return Ok((StatusCode::CREATED, Json(merchant_response)));
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

pub async fn get_merchant_handler(
    Path(id): Path<uuid::Uuid>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_result = sqlx::query_as::<_, MerchantModel>( r#"SELECT * FROM merchants WHERE id = $1"#)
        .bind(id)
        .fetch_one(&data.db)
        .await;

    match query_result {
        Ok(merchant) => {
            let merchant_response = serde_json::json!({"status": "success","data": serde_json::json!({
                "merchant": merchant
            })});

            return Ok(Json(merchant_response));
        }
        Err(_) => {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Merchant with ID: {} not found", id)
            });
            return Err((StatusCode::NOT_FOUND, Json(error_response)));
        }
    }
}


pub async fn edit_merchant_handler(
    Path(id): Path<uuid::Uuid>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<UpdateMerchantSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_result = sqlx::query_as::<_, MerchantModel>(r#"SELECT * FROM merchants WHERE id = $1"#)
        .bind(id)
        .fetch_one(&data.db)
        .await;

    if query_result.is_err() {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": format!("Merchant with ID: {} not found", id)
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    let now = chrono::Utc::now();
    let merchant: MerchantModel = query_result.unwrap();

    let query_result = sqlx::query_as::<_, MerchantModel>(
        r#"UPDATE merchants SET last_data_hash = $1 last_updated = $2 WHERE id = $3 RETURNING *"#
    )
    .bind(body.last_data_hash.to_owned().unwrap_or(merchant.last_data_hash))
    .bind(now)
    .bind(id)
    .fetch_one(&data.db)
    .await
    ;

    match query_result {
        Ok(merchant) => {
            let merchant_response = serde_json::json!({"status": "success","data": serde_json::json!({
                "merchant": merchant
            })});

            return Ok(Json(merchant_response));
        }
        Err(err) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error","message": format!("{:?}", err)})),
            ));
        }
    }
}

pub async fn delete_merchant_handler(
    Path(id): Path<uuid::Uuid>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let rows_affected = sqlx::query(r#"DELETE FROM merchants  WHERE id = $1"#)
    .bind(id)
        .execute(&data.db)
        .await
        .unwrap()
        .rows_affected();

    if rows_affected == 0 {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": format!("Merchant with ID: {} not found", id)
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    Ok(StatusCode::NO_CONTENT)
}
