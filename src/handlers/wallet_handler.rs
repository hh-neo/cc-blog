use axum::{http::StatusCode, Json};
use ethers_core::k256::ecdsa::SigningKey;
use ethers_core::utils::{hex, secret_key_to_address};
use rand::thread_rng;
use validator::Validate;

use crate::models::{ErrorResponse, GenerateWalletsRequest, GenerateWalletsResponse, WalletInfo};

pub async fn generate_wallets(
    Json(payload): Json<GenerateWalletsRequest>,
) -> Result<Json<GenerateWalletsResponse>, (StatusCode, Json<ErrorResponse>)> {
    if let Err(errors) = payload.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(format!("Validation error: {}", errors))),
        ));
    }

    let mut wallets = Vec::with_capacity(payload.count as usize);
    let mut rng = thread_rng();

    for _ in 0..payload.count {
        let signing_key = SigningKey::random(&mut rng);
        let address = secret_key_to_address(&signing_key);

        let address_str = format!("{:?}", address);
        let private_key = hex::encode(signing_key.to_bytes());

        wallets.push(WalletInfo {
            address: address_str,
            private_key,
        });
    }

    Ok(Json(GenerateWalletsResponse {
        count: payload.count,
        wallets,
    }))
}