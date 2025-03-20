use crate::{AppState, AppStateType};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use entity::user as User;
use entity::user::Entity as UserEntity;
use sea_orm::ActiveValue::Set;
use sea_orm::{ColumnTrait, DatabaseConnection};
use sea_orm::QueryFilter;
use sea_orm::{ActiveModelTrait, EntityTrait};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::helpers::auth::encode_jwt;

#[derive(Deserialize)]
pub struct CreateUser {
    username: String,
    licence: String,
    profile_url: String,
}

#[derive(Deserialize)]
pub struct LoginUser {
    username: String,
    licence: String,
}

#[derive(Serialize)]
pub struct CreatedUser {
    id: i32,
    username: String,
    licence: String,
    profile_url: String,
    payment_plan: bool,
}

#[derive(Serialize)]
pub struct AuthorizedUser {
    username: String,
    licence: String,
    profile_url: String,
    payment_plan: bool,
    token: String,
}

#[derive(Deserialize)]
pub struct UpdateUser {
    username: Option<String>,
    licence: Option<String>,
    profile_url: Option<String>,
    payment_plan: Option<bool>,
}

pub async fn create_user(
    State(state): State<AppStateType>,
    Json(payload): Json<CreateUser>,
) -> Json<Value> {
    let state = state.read().await;
    let conn = &state.conn;

    let existing_user = UserEntity::find()
        .filter(User::Column::LicensePlate.eq(&payload.licence))
        .one(conn)
        .await;

    match existing_user {
        Ok(Some(_)) => {
            return Json(json!({
                "status": "error",
                "message": "License plate already in use"
            }));
        }
        Ok(None) => {}
        Err(e) => {
            eprintln!("Error checking existing user: {}", e);
            return Json(json!({
                "status": "error",
                "message": "Failed to check existing user"
            }));
        }
    }

    let new_user = User::ActiveModel {
        username: Set(payload.username),
        license_plate: Set(payload.licence),
        payment_plan: Set(false),
        profile_url: Set(payload.profile_url),
        ..Default::default()
    };

    match new_user.save(conn).await {
        Ok(user_model) => Json(json!({
            "status": "success",
            "data": CreatedUser {
                id: user_model.id.unwrap(),
                username: user_model.username.unwrap(),
                licence: user_model.license_plate.unwrap(),
                profile_url: user_model.profile_url.unwrap(),
                payment_plan: user_model.payment_plan.unwrap(),
            }
        })),
        Err(e) => {
            eprintln!("Error creating user: {}", e);
            Json(json!({ "status": "error", "message": "Failed to fetch sensors" }))
        }
    }
}

pub async fn signin_user(State(state): State<AppStateType>, Json(payload): Json<LoginUser>,) -> Json<Value> {
    let state = state.read().await;
    let conn = &state.conn;

    match UserEntity::find()
        .filter(User::Column::Username.eq(&payload.username))
        .filter(User::Column::LicensePlate.eq(&payload.licence))
        .one(conn)
        .await
    {
        Ok(user) => match user {
            Some(user) => Json(json!({
                "status": "success",
                "data": AuthorizedUser {
                    username: String::from(&user.username),
                    licence: String::from(&user.license_plate),
                    profile_url: user.profile_url,
                    payment_plan: user.payment_plan,
                    token: encode_jwt(&user.license_plate, &user.username)
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR).expect("Unexpected error occurred"),
                }
            })),
            None => Json(json!({
                "status": "error",
                "message": "User not found. Please do register"
            })),
        },
        Err(e) => {
            eprintln!("Error fetching user: {}", e);
            Json(json!({
                "status": "error",
                "message": "Failed to fetch user"
            }))
        }
    }
}


pub async fn is_valid_user(conn: &DatabaseConnection, username: &str, licence: &str) -> bool {


    match UserEntity::find()
        .filter(User::Column::Username.eq(username))
        .filter(User::Column::LicensePlate.eq(licence))
        .one(conn)
        .await
    {
        Ok(user) => match user {
            Some(_) => true,
            None => false,
        },
        Err(e) => {
            eprintln!("Error fetching user: {}", e);
            false
        }
    }
}

pub async fn update_user(
    State(state): State<Arc<RwLock<AppState>>>,
    Path(user_id): Path<i32>, // Assuming user_id is an integer
    Json(payload): Json<UpdateUser>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let state = state.read().await;
    let conn = &state.conn;

    // Fetch the existing user
    let user = User::Entity::find()
        .filter(User::Column::Id.eq(user_id))
        .one(conn)
        .await
        .map_err(|e| {
            eprintln!("Error fetching user: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to fetch user".to_string(),
            )
        })?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "User not found".to_string()))?;

    // Convert to an ActiveModel to update
    let mut active_user: User::ActiveModel = user.into();

    // Update only the provided fields
    if let Some(username) = payload.username {
        active_user.username = Set(username);
    }
    if let Some(licence) = payload.licence {
        active_user.license_plate = Set(licence);
    }
    if let Some(profile_url) = payload.profile_url {
        active_user.profile_url = Set(profile_url);
    }
    if let Some(payment_plan) = payload.payment_plan {
        active_user.payment_plan = Set(payment_plan);
    }

    // Save the updated user
    let updated_user = active_user.update(conn).await.map_err(|e| {
        eprintln!("Error updating user: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to update user".to_string(),
        )
    })?;

    // Convert to JSON response
    let response = json!({
        "status": "success",
        "data": CreatedUser {
            id: updated_user.id,
            username: updated_user.username,
            licence: updated_user.license_plate,
            profile_url: updated_user.profile_url,
            payment_plan: updated_user.payment_plan,
        }
    });

    Ok(Json(response))
}
