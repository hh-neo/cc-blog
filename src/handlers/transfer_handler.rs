use axum::{http::StatusCode, Json};
use ethers::{
    prelude::*,
    providers::{Http, Provider},
    signers::{LocalWallet, Signer},
    types::{TransactionRequest, U256},
    utils::parse_ether,
};
use std::env;
use std::sync::Arc;
use validator::Validate;

use crate::models::{ErrorResponse, TransferRequest, TransferResponse, TransferResult};

fn get_rpc_url(chain: &str) -> Result<String, String> {
    let env_key = format!("RPC_URL_{}", chain.to_uppercase());
    env::var(&env_key).map_err(|_| format!("RPC URL not configured for chain: {}", chain))
}

pub async fn batch_transfer(
    Json(payload): Json<TransferRequest>,
) -> Result<Json<TransferResponse>, (StatusCode, Json<ErrorResponse>)> {
    if let Err(errors) = payload.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(format!("Validation error: {}", errors))),
        ));
    }

    let rpc_url = get_rpc_url(&payload.chain).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(e)),
        )
    })?;

    let provider = Provider::<Http>::try_from(rpc_url).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Failed to connect to RPC: {}", e))),
        )
    })?;

    let provider = Arc::new(provider);

    let private_key = payload.private_key.trim_start_matches("0x");
    let wallet: LocalWallet = private_key.parse().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(format!("Invalid private key: {}", e))),
        )
    })?;

    let chain_id = provider.get_chainid().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Failed to get chain ID: {}", e))),
        )
    })?;

    let wallet = wallet.with_chain_id(chain_id.as_u64());
    let client = SignerMiddleware::new(provider.clone(), wallet);
    let client = Arc::new(client);

    let total = payload.transfers.len() as u32;
    let mut success = 0u32;
    let mut failed = 0u32;
    let mut results = Vec::with_capacity(payload.transfers.len());

    for transfer in payload.transfers {
        let to_address_str = transfer.to_address.clone();
        let amount_str = transfer.amount.clone();

        let to_address: Address = transfer.to_address.parse().map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(format!(
                    "Invalid address {}: {}",
                    transfer.to_address, e
                ))),
            )
        })?;

        let amount: U256 = parse_ether(&transfer.amount).map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(format!(
                    "Invalid amount {}: {}",
                    transfer.amount, e
                ))),
            )
        })?;

        let tx = TransactionRequest::new()
            .to(to_address)
            .value(amount);

        match client.send_transaction(tx, None).await {
            Ok(pending_tx) => {
                match pending_tx.await {
                    Ok(Some(receipt)) => {
                        success += 1;
                        tracing::info!(
                            "Transfer success: {} ETH to {}, tx: {:?}",
                            amount_str,
                            to_address_str,
                            receipt.transaction_hash
                        );
                        results.push(TransferResult {
                            to_address: to_address_str,
                            amount: amount_str,
                            success: true,
                            tx_hash: Some(format!("{:?}", receipt.transaction_hash)),
                            error: None,
                        });
                    }
                    Ok(None) => {
                        failed += 1;
                        results.push(TransferResult {
                            to_address: to_address_str,
                            amount: amount_str,
                            success: false,
                            tx_hash: None,
                            error: Some("Transaction receipt not found".to_string()),
                        });
                    }
                    Err(e) => {
                        failed += 1;
                        results.push(TransferResult {
                            to_address: to_address_str,
                            amount: amount_str,
                            success: false,
                            tx_hash: None,
                            error: Some(format!("Transaction failed: {}", e)),
                        });
                    }
                }
            }
            Err(e) => {
                failed += 1;
                results.push(TransferResult {
                    to_address: to_address_str,
                    amount: amount_str,
                    success: false,
                    tx_hash: None,
                    error: Some(format!("Failed to send transaction: {}", e)),
                });
            }
        }
    }

    Ok(Json(TransferResponse {
        total,
        success,
        failed,
        results,
    }))
}