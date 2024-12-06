use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;

use crate::{
    model::DogModel,
    schema::{CreateDogSchema, FilterOptions, UpdateDogSchema},
    AppState,
};


pub async fn dog_list_handler(
    opts: Option<Query<FilterOptions>>, State(data): State<Arc<AppState>>
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let Query(opts) = opts.unwrap_or_default();
    
    let limit = opts.limit.unwrap_or(10);
    let offset = (opts.page.unwrap_or(1) - 1) * limit;
    let query_result = sqlx::query_as::<_, DogModel>(
        r#"SELECT * FROM dogs ORDER by id LIMIT $1 OFFSET $2"#
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

    let dogs = query_result.unwrap();

    let json_response = serde_json::json!({
        "status": "success",
        "results": dogs.len(),
        "dogs": dogs
    });
    Ok(Json(json_response))
}

pub async fn create_dog_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<CreateDogSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_result = sqlx::query_as::<_, DogModel>(
        r#"INSERT INTO dogs (name,breed,color, location, prooflevel) VALUES ($1, $2, $3, $4, $5) RETURNING *"#
    )
    .bind(body.name)
    .bind(body.breed)
    .bind(body.color)
    .bind(body.location)
    .bind(body.prooflevel)
    .fetch_one(&data.db)
    .await;

    match query_result {
        Ok(dog) => {
            let dog_response = json!({"status": "success","data": json!({
                "dog": dog
            })});

            return Ok((StatusCode::CREATED, Json(dog_response)));
        }
        Err(e) => {
            if e.to_string()
                .contains("duplicate key value violates unique constraint")
            {
                let error_response = serde_json::json!({
                    "status": "fail",
                    "message": "Note with that title already exists",
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

pub async fn get_dog_handler(
    Path(id): Path<uuid::Uuid>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_result = sqlx::query_as::<_, DogModel>( r#"SELECT * FROM dogs WHERE id = $1"#)
        .bind(id)
        .fetch_one(&data.db)
        .await;

    match query_result {
        Ok(dog) => {
            let dog_response = serde_json::json!({"status": "success","data": serde_json::json!({
                "dog": dog
            })});

            return Ok(Json(dog_response));
        }
        Err(_) => {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Dog with ID: {} not found", id)
            });
            return Err((StatusCode::NOT_FOUND, Json(error_response)));
        }
    }
}

pub async fn edit_dog_handler(
    Path(id): Path<uuid::Uuid>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<UpdateDogSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_result = sqlx::query_as::<_, DogModel>(r#"SELECT * FROM dogs WHERE id = $1"#)
        .bind(id)
        .fetch_one(&data.db)
        .await;

    if query_result.is_err() {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": format!("Dog with ID: {} not found", id)
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    let now = chrono::Utc::now();
    let dog = query_result.unwrap();

    let query_result = sqlx::query_as::<_, DogModel>(
        r#"UPDATE dogs SET name = $1, breed = $2, color = $3, location = $4, prooflevel = $5, updated_at = $6 WHERE id = $7 RETURNING *"#
    )
    .bind(body.name.to_owned().unwrap_or(dog.name))
    .bind(body.breed.clone().or(dog.breed))
    .bind(body.color.to_owned().unwrap_or(dog.color))
    .bind(body.location.clone().or(dog.location))
    .bind(body.prooflevel.or(dog.prooflevel))
    .bind(now)
    .bind(id)
    .fetch_one(&data.db)
    .await
    ;

    match query_result {
        Ok(dog) => {
            let dog_response = serde_json::json!({"status": "success","data": serde_json::json!({
                "dog": dog
            })});

            return Ok(Json(dog_response));
        }
        Err(err) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error","message": format!("{:?}", err)})),
            ));
        }
    }
}

pub async fn delete_dog_handler(
    Path(id): Path<uuid::Uuid>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let rows_affected = sqlx::query(r#"DELETE FROM dogs  WHERE id = $1"#)
    .bind(id)
        .execute(&data.db)
        .await
        .unwrap()
        .rows_affected();

    if rows_affected == 0 {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": format!("Note with ID: {} not found", id)
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    Ok(StatusCode::NO_CONTENT)
}