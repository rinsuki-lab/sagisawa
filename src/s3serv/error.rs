use axum::{http::StatusCode, response::IntoResponse};
use serde::Serialize;

#[derive(strum::IntoStaticStr)]
pub enum S3Error {
    AccessDenied,
    InternalError,
    NotImplemented,
    // ---
    NoSuchBucket,
    NoSuchKey,
    // create bucket
    BucketAlreadyExists,
    BucketAlreadyOwnedByYou,
}

impl IntoResponse for S3Error {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            S3Error::AccessDenied => StatusCode::FORBIDDEN,
            S3Error::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            S3Error::NotImplemented => StatusCode::NOT_IMPLEMENTED,
            S3Error::NoSuchBucket => StatusCode::NOT_FOUND,
            S3Error::NoSuchKey => StatusCode::NOT_FOUND,
            S3Error::BucketAlreadyExists => StatusCode::CONFLICT,
            S3Error::BucketAlreadyOwnedByYou => StatusCode::CONFLICT,
        };
        let description = match self {
            S3Error::AccessDenied => "Access Denied",
            S3Error::InternalError => "Server encounted an internal error",
            S3Error::NotImplemented => "Currently this feature is not implemented",
            S3Error::BucketAlreadyExists => "Bucket already exists",
            S3Error::BucketAlreadyOwnedByYou => "Bucket already owned by you",
            S3Error::NoSuchBucket => "The specified bucket does not exist",
            S3Error::NoSuchKey => "The specified key does not exist",
        };

        let mut buffer = String::new();
        let serializer = quick_xml::se::Serializer::new(&mut buffer);

        #[derive(serde::Serialize)]
        struct Error {
            #[serde(rename = "Code")]
            code: &'static str,
            #[serde(rename = "Message")]
            message: String,
        }

        let error = Error {
            code: self.into(),
            message: description.to_string(),
        };

        error.serialize(serializer).expect("Failed to serialize.");

        (status, [("Content-Type", "application/xml")], buffer).into_response()
    }
}
