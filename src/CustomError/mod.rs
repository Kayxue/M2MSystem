use std::{convert::Infallible, error, fmt};

use xitca_web::{
    error::Error,
    http::{StatusCode, WebResponse},
    service::Service,
    WebContext,
};

#[derive(Debug)]
pub struct BadRequest {
    pub message: String,
}

impl BadRequest {
    pub fn new(message: &str) -> Self {
        BadRequest {
            message: message.to_owned(),
        }
    }
}

impl fmt::Display for BadRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BadRequest: {}", self.message)
    }
}

impl error::Error for BadRequest {}

impl From<BadRequest> for Error {
    fn from(err: BadRequest) -> Self {
        Self::from_service(err)
    }
}

impl<'r, C> Service<WebContext<'r, C>> for BadRequest {
    type Response = WebResponse;
    type Error = Infallible;

    async fn call(&self, ctx: WebContext<'r, C>) -> Result<Self::Response, Self::Error> {
        Service::call(&StatusCode::BAD_REQUEST, ctx).await
    }
}

#[derive(Debug)]
pub struct InternalServerError {
    pub message: String,
}

impl InternalServerError {
    pub fn new(message: &str) -> Self {
        InternalServerError {
            message: message.to_owned(),
        }
    }
}

impl fmt::Display for InternalServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "InternalServerError: {}", self.message)
    }
}

impl error::Error for InternalServerError {}

impl From<InternalServerError> for Error {
    fn from(err: InternalServerError) -> Self {
        Self::from_service(err)
    }
}

impl<'r, C> Service<WebContext<'r, C>> for InternalServerError {
    type Response = WebResponse;
    type Error = Infallible;

    async fn call(&self, ctx: WebContext<'r, C>) -> Result<Self::Response, Self::Error> {
        Service::call(&StatusCode::INTERNAL_SERVER_ERROR, ctx).await
    }
}