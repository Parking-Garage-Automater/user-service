use std::{env, sync::Arc};

use crate::{AppState};
use axum::{
    extract::{Json, Request, State}, http::{self, StatusCode}
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde_json::{json, Value};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use crate::helpers::users::is_valid_user;

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub exp: usize,
    pub iat: usize,
    pub licence: String,
    pub username: String,
}

pub fn encode_jwt(licence_plate: &String, user_name: &String) -> Result<String, StatusCode>{
    let jwt_secret_key = env::var("JWT_SECRET_KEY").expect("JWT_SECRET_KEY is not set in the environment");
    let now = Utc::now();
    let expire: chrono::TimeDelta = Duration::days(5);
    let exp: usize = (now +  expire).timestamp() as usize;
    let iat: usize = now.timestamp() as usize;
    let licence: String = licence_plate.to_string();
    let username: String = user_name.to_string();
    let claim = Claims { exp, iat, licence, username};

    encode(&Header::default(), 
        &claim, &EncodingKey::from_secret(jwt_secret_key.as_ref()))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub fn decode_jwt(jwt_token: String) -> Result<TokenData<Claims>, StatusCode> {
    let jwt_secret_key = env::var("JWT_SECRET_KEY").expect("JWT_SECRET_KEY is not set in the environment");
    let result: Result<TokenData<Claims>, StatusCode> = decode(&jwt_token, 
        &DecodingKey::from_secret(jwt_secret_key.as_ref()), &Validation::default(),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    result
}

pub async fn authorize(State(state): State<Arc<RwLock<AppState>>>, mut req: Request) -> Json<Value> {
    let unauthorized_status_code = StatusCode::UNAUTHORIZED.to_string();
    let ok_status_code = StatusCode::OK.to_string();
    let forbidden_status_code = StatusCode::FORBIDDEN.to_string();
    let auth_header = req.headers_mut().get(http::header::AUTHORIZATION);
    let auth_header = match auth_header {
        Some(header) => header.to_str().map_err(|_| Json(json!({
            "message": "Empty header is not allowed",
            "status_code": forbidden_status_code
        }))),
        None => Err(Json(json!({
            "message": "Please add the JWT token to the header",
            "status_code": forbidden_status_code
        }))),
    };
    let mut header = auth_header.expect("Expected a valid string").split_whitespace();
    let (_bearer, token) = (header.next(), header.next());
    let token_data = match decode_jwt(token.unwrap().to_string()) {
        Ok(data) => data,
        Err(_) => return Json(json!({
            "message": "Please add the JWT token to the header",
            "status_code": unauthorized_status_code
        })),
    };
    let state = state.read().await;
    let conn = &state.conn;
    let validity_of_user = is_valid_user(conn, &token_data.claims.username, &token_data.claims.licence).await;
    if validity_of_user {
        let response  =json!({
            "message": "Valid User",
            "status_code": ok_status_code
        });
        return Json(response)
    } else {
        let response  =json!({
            "message": "Unauthorized User",
            "status_code": unauthorized_status_code
        });
        return Json(response)
    }
    
}
