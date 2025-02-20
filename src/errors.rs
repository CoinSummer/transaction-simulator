use eyre::Report;
use serde::{Deserialize, Serialize};
use std::{convert::Infallible, error::Error};

use warp::{body::BodyDeserializeError, hyper::StatusCode, reject::Reject, Rejection, Reply};

#[derive(Serialize, Deserialize)]
pub struct ErrorMessage {
    pub code: u16,
    pub message: String,
}

#[derive(Debug)]
pub struct FromHexError;

impl Reject for FromHexError {}

#[derive(Debug)]
pub struct FromDecStrError;

impl Reject for FromDecStrError {}

#[derive(Debug)]
pub struct NoURLForChainIdError;

impl Reject for NoURLForChainIdError {}

#[derive(Debug)]
pub struct MultipleChainIdsError();

impl Reject for MultipleChainIdsError {}

#[derive(Debug)]
pub struct MultipleBlockNumbersError();

impl Reject for MultipleBlockNumbersError {}

#[derive(Debug)]
pub struct EvmError(pub Report);

impl Reject for EvmError {}

pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let code;
    let message: String;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "NOT_FOUND".to_string();
    } else if let Some(FromHexError) = err.find() {
        code = StatusCode::BAD_REQUEST;
        message = "FROM_HEX_ERROR".to_string();
    } else if let Some(FromDecStrError) = err.find() {
        code = StatusCode::BAD_REQUEST;
        message = "FROM_DEC_STR_ERROR".to_string();
    } else if let Some(NoURLForChainIdError) = err.find() {
        code = StatusCode::BAD_REQUEST;
        message = "CHAIN_ID_NOT_SUPPORTED".to_string();
    } else if let Some(_e) = err.find::<MultipleChainIdsError>() {
        code = StatusCode::BAD_REQUEST;
        message = "MULTIPLE_CHAIN_IDS".to_string();
    } else if let Some(_e) = err.find::<MultipleBlockNumbersError>() {
        code = StatusCode::BAD_REQUEST;
        message = "MULTIPLE_BLOCK_NUMBERS".to_string();
    } else if let Some(_e) = err.find::<EvmError>() {
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "EVM_ERROR".to_string();
    } else if let Some(e) = err.find::<BodyDeserializeError>() {
        // This error happens if the body could not be deserialized correctly
        // We can use the cause to analyze the error and customize the error message
        dbg!(e);
        message = match e.source() {
            Some(cause) => format!("BAD REQUEST: {cause}"),
            None => "BAD_REQUEST".to_string(),
        };
        code = StatusCode::BAD_REQUEST;
    } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
        // We can handle a specific error, here METHOD_NOT_ALLOWED,
        // and render it however we want
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "METHOD_NOT_ALLOWED".to_string();
    } else if err.find::<warp::reject::MissingHeader>().is_some() {
        code = StatusCode::UNAUTHORIZED;
        message = "UNAUTHORIZED".to_string();
    } else {
        // We should have expected this... Just log and say its a 500
        eprintln!("unhandled rejection: {err:?}");
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "UNHANDLED_REJECTION".to_string();
    }

    let json = warp::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message,
    });

    Ok(warp::reply::with_status(json, code))
}
