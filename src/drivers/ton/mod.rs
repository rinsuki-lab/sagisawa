use axum::{
    body::{BodyDataStream, Bytes},
    response::{IntoResponse, Response},
};
use md5::Digest;
use sqlx::PgTransaction;
use tokio_stream::StreamExt;

use crate::s3serv::error::S3Error;

#[derive(serde::Deserialize)]
struct SessionStartResponse {
    token: String,
    chunk_size: usize,
}

#[derive(serde::Serialize)]
struct UploadFinalizeRequest {
    name: String,
    md5: String,
}

#[derive(serde::Deserialize)]
struct UploadFinalizeResponse {
    r#ref: String,
}

pub struct UploadResult {
    pub r#ref: String,
    pub md5: [u8; 16],
    pub size: u64,
}

#[tracing::instrument(skip(client, session, bytes), fields())]
async fn upload_chunk(
    client: &reqwest::Client,
    session: &SessionStartResponse,
    offset: u64,
    bytes: &[u8],
) -> Result<(), Response> {
    assert!(bytes.len() <= session.chunk_size);
    let upload_chunk_res = client
        .post("http://localhost:4000/v1/upload/chunk")
        .query(&[("token", &session.token), ("offset", &offset.to_string())])
        .body(bytes.to_vec()) // TODO: why I need to copy it?
        .send()
        .await;

    let upload_chunk_res = match upload_chunk_res {
        Ok(v) => {
            if !v.status().is_success() {
                let status = v.status();
                let v = v.text().await.unwrap_or_default();
                tracing::error!("Failed to upload chunk: {}, {}", status, v);
                return Err(S3Error::InternalError.into_response());
            }
            v
        }
        Err(e) => {
            tracing::error!("Failed to upload chunk: {:?}", e);
            return Err(S3Error::InternalError.into_response());
        }
    };

    tracing::debug!("chunk uploaded, {}", upload_chunk_res.status());

    Ok(())
}

pub async fn upload_from_stream(
    pool: &PgTransaction<'_>,
    body: &mut BodyDataStream,
) -> Result<Option<UploadResult>, Response> {
    let first = loop {
        let first = body.next().await;
        match first {
            None => return Ok(None),
            Some(Ok(v)) => {
                if v.is_empty() {
                    continue;
                }
                break v;
            }
            Some(Err(e)) => {
                tracing::error!("Failed to read first frame: {:?}", e);
                return Err(S3Error::InternalError.into_response());
            }
        }
    };

    let client = reqwest::Client::new();

    let session_result = client
        .post("http://localhost:4000/v1/upload/start")
        .send()
        .await;

    let session_result = match session_result {
        Ok(v) => {
            if !v.status().is_success() {
                let status = v.status();
                let v = v.text().await.unwrap_or_default();
                tracing::error!("Failed to start upload: {}, {}", status, v);
                return Err(S3Error::InternalError.into_response());
            }
            v
        }
        Err(e) => {
            tracing::error!("Failed to start upload: {:?}", e);
            return Err(S3Error::InternalError.into_response());
        }
    };

    let session_result = session_result.json::<SessionStartResponse>().await;

    let session = match session_result {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Failed to parse session start response: {:?}", e);
            return Err(S3Error::InternalError.into_response());
        }
    };

    let mut buf = Vec::<u8>::with_capacity(session.chunk_size);

    if session.chunk_size < 1 {
        tracing::error!("Chunk size is too small");
        return Err(S3Error::InternalError.into_response());
    }

    let mut current = first;
    let mut offset = 0;
    let mut hasher = md5::Md5::new();
    loop {
        if (buf.len() + current.len()) > session.chunk_size {
            let available = session.chunk_size - buf.len();
            buf.extend_from_slice(&current[0..available]);
            tracing::debug!(
                "uploading chunk len={} + {}, offset={}",
                buf.len(),
                current.len(),
                offset
            );
            hasher.update(&buf);
            upload_chunk(&client, &session, offset, &buf).await?;
            offset += buf.len() as u64;
            current = Bytes::from(current[available..].to_vec());
            tracing::debug!("new current len={}", current.len());
            buf.clear();
            continue;
        } else {
            tracing::debug!(
                "adding current thing to buffer {} + {}",
                buf.len(),
                current.len()
            );
            buf.extend_from_slice(&current);
        }

        let next = body.next().await;
        match next {
            None => {
                tracing::debug!("End of stream");
                break;
            }
            Some(Ok(v)) => {
                tracing::debug!("Read frame {}", v.len());
                current = v;
            }
            Some(Err(e)) => {
                tracing::error!("Failed to read frame: {:?}", e);
                return Err(S3Error::InternalError.into_response());
            }
        }
    }

    if !buf.is_empty() {
        hasher.update(&buf);
        upload_chunk(&client, &session, offset, &buf).await?;
        offset += buf.len() as u64;
    }
    drop(buf);

    let hasher = hasher.finalize();

    let finalize_chunk_res = client
        .post("http://localhost:4000/v1/upload/finalize")
        .query(&[("token", &session.token)])
        .json(&UploadFinalizeRequest {
            name: "sagisawa.bin".to_string(),
            md5: hex::encode(&hasher),
        })
        .send()
        .await;

    let finalize_chunk_res = match finalize_chunk_res {
        Ok(v) => {
            if !v.status().is_success() {
                let status = v.status();
                let v = v.text().await.unwrap_or_default();
                tracing::error!("Failed to finish upload: {}, {}", status, v);
                return Err(S3Error::InternalError.into_response());
            }
            v
        }
        Err(e) => {
            tracing::error!("Failed to finish upload: {:?}", e);
            return Err(S3Error::InternalError.into_response());
        }
    };

    let finalize_chunk_res = finalize_chunk_res.json::<UploadFinalizeResponse>().await;

    match finalize_chunk_res {
        Ok(v) => Ok(Some(UploadResult {
            r#ref: v.r#ref,
            md5: hasher.into(),
            size: offset,
        })),
        Err(e) => {
            tracing::error!("Failed to parse session finish response: {:?}", e);
            return Err(S3Error::InternalError.into_response());
        }
    }
}
