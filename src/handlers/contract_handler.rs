use axum::{http::StatusCode, Json};
use ethers::{
    abi::{Abi, Token},
    prelude::*,
    providers::{Http, Provider},
    signers::{LocalWallet, Signer},
    types::{transaction::eip2718::TypedTransaction, TransactionRequest},
};
use std::env;
use std::sync::Arc;
use validator::Validate;

use crate::models::{ContractCallRequest, ContractCallResponse, ErrorResponse};

fn get_rpc_url(chain: &str) -> Result<String, String> {
    let env_key = format!("RPC_URL_{}", chain.to_uppercase());
    env::var(&env_key).map_err(|_| format!("RPC URL not configured for chain: {chain}"))
}

fn parse_abi(abi_str: &str) -> Result<Abi, String> {
    serde_json::from_str(abi_str).map_err(|e| format!("Invalid ABI JSON: {e}"))
}

fn convert_json_to_token(
    value: &serde_json::Value,
    param_type: &ethers::abi::ParamType,
) -> Result<Token, String> {
    match (value, param_type) {
        (serde_json::Value::String(s), ethers::abi::ParamType::Address) => {
            let addr: Address = s.parse().map_err(|e| format!("Invalid address: {e}"))?;
            Ok(Token::Address(addr))
        }
        (serde_json::Value::String(s), ethers::abi::ParamType::Uint(_)) => {
            let num: U256 = s.parse().map_err(|e| format!("Invalid uint: {e}"))?;
            Ok(Token::Uint(num))
        }
        (serde_json::Value::Number(n), ethers::abi::ParamType::Uint(_)) => {
            if let Some(num) = n.as_u64() {
                Ok(Token::Uint(U256::from(num)))
            } else {
                Err("Invalid uint number".to_string())
            }
        }
        (serde_json::Value::String(s), ethers::abi::ParamType::String) => {
            Ok(Token::String(s.clone()))
        }
        (serde_json::Value::Bool(b), ethers::abi::ParamType::Bool) => Ok(Token::Bool(*b)),
        (serde_json::Value::String(s), ethers::abi::ParamType::Bytes) => {
            let bytes = hex::decode(s.trim_start_matches("0x"))
                .map_err(|e| format!("Invalid bytes: {e}"))?;
            Ok(Token::Bytes(bytes))
        }
        (serde_json::Value::Array(arr), ethers::abi::ParamType::Array(inner_type)) => {
            let tokens: Result<Vec<Token>, String> = arr
                .iter()
                .map(|v| convert_json_to_token(v, inner_type.as_ref()))
                .collect();
            Ok(Token::Array(tokens?))
        }
        (serde_json::Value::Array(arr), ethers::abi::ParamType::FixedArray(inner_type, _size)) => {
            let tokens: Result<Vec<Token>, String> = arr
                .iter()
                .map(|v| convert_json_to_token(v, inner_type.as_ref()))
                .collect();
            Ok(Token::FixedArray(tokens?))
        }
        _ => Err(format!(
            "Unsupported type conversion: {value:?} to {param_type:?}"
        )),
    }
}

pub async fn call_contract(
    Json(payload): Json<ContractCallRequest>,
) -> Result<Json<ContractCallResponse>, (StatusCode, Json<ErrorResponse>)> {
    if let Err(errors) = payload.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(format!("Validation error: {errors}"))),
        ));
    }

    let rpc_url = get_rpc_url(&payload.chain)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(ErrorResponse::new(e))))?;

    let provider = Provider::<Http>::try_from(rpc_url).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!(
                "Failed to connect to RPC: {e}"
            ))),
        )
    })?;

    let contract_address: Address = payload.contract_address.parse().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(format!(
                "Invalid contract address: {e}"
            ))),
        )
    })?;

    let abi = parse_abi(&payload.abi)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(ErrorResponse::new(e))))?;

    let function = abi
        .functions()
        .find(|f| f.name == payload.function_name)
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(format!(
                    "Function '{}' not found in ABI",
                    payload.function_name
                ))),
            )
        })?;

    if function.inputs.len() != payload.params.len() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(format!(
                "Expected {} parameters, got {}",
                function.inputs.len(),
                payload.params.len()
            ))),
        ));
    }

    let mut tokens = Vec::new();
    for (i, param) in payload.params.iter().enumerate() {
        let param_type = &function.inputs[i].kind;
        let token = convert_json_to_token(param, param_type).map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(format!("Parameter {i} error: {e}"))),
            )
        })?;
        tokens.push(token);
    }

    if let Some(private_key) = payload.private_key {
        let private_key = private_key.trim_start_matches("0x");
        let wallet: LocalWallet = private_key.parse().map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(format!("Invalid private key: {e}"))),
            )
        })?;

        let chain_id = provider.get_chainid().await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(format!("Failed to get chain ID: {e}"))),
            )
        })?;

        let wallet = wallet.with_chain_id(chain_id.as_u64());
        let client = SignerMiddleware::new(Arc::new(provider), wallet);

        let data = function.encode_input(&tokens).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(format!(
                    "Failed to encode function input: {e:?}"
                ))),
            )
        })?;

        let mut tx = TransactionRequest::new().to(contract_address).data(data);

        if let Some(value_str) = payload.value {
            let value: U256 = value_str.parse().map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse::new(format!("Invalid value: {e}"))),
                )
            })?;
            tx = tx.value(value);
            tracing::info!("Transaction value: {}", value);
        }

        let pending_tx = client.send_transaction(tx, None).await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(format!(
                    "Failed to send transaction: {e:?}"
                ))),
            )
        })?;

        let receipt = pending_tx
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::new(format!("Transaction failed: {e}"))),
                )
            })?
            .ok_or_else(|| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::new("Transaction receipt not found")),
                )
            })?;

        tracing::info!(
            "Contract call success: {} on {}, tx: {:?}",
            payload.function_name,
            payload.contract_address,
            receipt.transaction_hash
        );
        Ok(Json(ContractCallResponse {
            success: true,
            tx_hash: Some(format!("{:?}", receipt.transaction_hash)),
            result: None,
            error: None,
        }))
    } else {
        let data = function.encode_input(&tokens).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(format!(
                    "Failed to encode function input: {e:?}"
                ))),
            )
        })?;

        let call_tx: TypedTransaction = TransactionRequest::new()
            .to(contract_address)
            .data(data)
            .into();

        match provider.call(&call_tx, None).await {
            Ok(result_bytes) => {
                let result_tokens = function.decode_output(&result_bytes).map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse::new(format!(
                            "Failed to decode output: {e:?}"
                        ))),
                    )
                })?;

                let result_json: Vec<String> = result_tokens
                    .into_iter()
                    .map(|token| format!("{token:?}"))
                    .collect();

                Ok(Json(ContractCallResponse {
                    success: true,
                    tx_hash: None,
                    result: Some(serde_json::json!(result_json)),
                    error: None,
                }))
            }
            Err(e) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(format!("Contract call failed: {e:?}"))),
            )),
        }
    }
}
