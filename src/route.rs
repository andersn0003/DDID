use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};

use crate::{
    doghandler::{create_dog_handler, delete_dog_handler, dog_list_handler, edit_dog_handler, get_dog_handler},
    merchanthandler::{create_merchant_handler, delete_merchant_handler, edit_merchant_handler, get_merchant_handler, merchant_list_handler},
    merchantrecordhandler::{create_merchant_record_handler, delete_merchant_record_handler, get_merchant_record_handler, merchant_record_list_handler},
    AppState,
};

pub fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/dogs", get(dog_list_handler))
        .route("/api/dogs/", post(create_dog_handler))
        .route(
            "/api/dogs/:id",
            get(get_dog_handler)
            .patch(edit_dog_handler)
            .delete(delete_dog_handler)
        )
        .route("/api/merchants", get(merchant_list_handler))
        .route("/api/merchants/", post(create_merchant_handler))
        .route(
            "/api/merchants/:id",
            get(get_merchant_handler)
            .patch(edit_merchant_handler)
            .delete(delete_merchant_handler)
        )
        .route("/api/merchantrecords/:merchant_id", get(merchant_record_list_handler))
        .route(
            "/api/merchantrecords/:merchant_record_id", 
            get(get_merchant_record_handler)
            .delete(delete_merchant_record_handler)
        )
        .route("/api/merchantrecords/", post(create_merchant_record_handler))
        .with_state(app_state)
}