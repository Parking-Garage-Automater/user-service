use crate::{AppState, AppStateType};
use axum::{
    extract::{Request,Json},
    http::{Response, StatusCode},
    middleware::Next,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use crate::helpers::users::get_user_by_username_and_licence;

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub exp: usize,
    pub iat: usize,
    pub licence: String,
    pub username: String,
}
pub struct AuthError {
    message: String,
    status_code: StatusCode,
}

pub fn encode_jwt(licence_plate: &String, user_name: &String) -> Result<String, StatusCode>{
    let secret: String = "randomString".to_string();
    let now = Utc::now();
    let expire: chrono::TimeDelta = Duration::days(5);
    let exp: usize = (now +  expire).timestamp() as usize;
    let iat: usize = now.timestamp() as usize;
    let licence: String = licence_plate.to_string();
    let username: String = user_name.to_string();
    let claim = Claims { exp, iat, licence, username};

    encode(&Header::default(), 
        &claim, &EncodingKey::from_secret(secret.as_ref()))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub fn decode_jwt(jwt_token: String) -> Result<TokenData<Claims>, StatusCode> {
    let secret: String = "randomString".to_string();
    let result: Result<TokenData<Claims>, StatusCode> = decode(&jwt_token, 
        &DecodingKey::from_secret(secret.as_ref()), &Validation::default(),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    result
}

// pub async fn authorize_middleware(mut req: Request, next: Next) -> Result<Response<Body>, AuthError> {
//     let auth_header = req.headers_mut().get(http::header::AUTHORIZATION);
//     let auth_header = match auth_header {
//         Some(header) => header.to_str().map_err(|_| AuthError {
//             message: "Empty header is not allowed".to_string(),
//             status_code: StatusCode::FORBIDDEN
//         })?,
//         None => return Err(AuthError {
//             message: "Please add the JWT token to the header".to_string(),
//             status_code: StatusCode::FORBIDDEN
//         }),
//     };
//     let mut header = auth_header.split_whitespace();
//     let (bearer, token) = (header.next(), header.next());
//     let token_data = match decode_jwt(token.unwrap().to_string()) {
//         Ok(data) => data,
//         Err(_) => return Err(AuthError {
//             message: "Unable to decode token".to_string(),
//             status_code: StatusCode::UNAUTHORIZED
//         }),
//     };

//     let current_user = match get_user_by_username_and_licence(&token_data.claims.licence, &token_data.claims.username) {
//         Some(user) => user,
//         None => return Err(AuthError {
//             message: "You are not an authorized user".to_string(),
//             status_code: StatusCode::UNAUTHORIZED
//         }),
//     };
//     req.extensions_mut().insert(current_user);
//     Ok(next.run(req).await)
// }
