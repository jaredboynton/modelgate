use std::io::{Cursor, Read};

use axum::{
    body::Bytes,
    http::{header, HeaderMap},
};

use crate::{AppError, AppResult};

pub const MAX_ENCODED_REQUEST_BODY_BYTES: usize = 64 * 1024 * 1024;
const MAX_DECODED_REQUEST_BODY_BYTES: u64 = 64 * 1024 * 1024;

pub fn has_content_encoding(headers: &HeaderMap) -> bool {
    headers
        .get_all(header::CONTENT_ENCODING)
        .iter()
        .any(|value| value.to_str().is_ok_and(|value| !value.trim().is_empty()))
}

pub fn decode_content_encoded_body(headers: &mut HeaderMap, body: Bytes) -> AppResult<Bytes> {
    let encodings = content_encodings(headers)?;
    if encodings.is_empty() {
        return Ok(body);
    }

    let mut decoded = body;
    let mut decoded_any = false;
    for encoding in encodings.iter().rev() {
        match encoding.as_str() {
            "identity" => {}
            "gzip" | "x-gzip" => {
                decoded =
                    read_limited(flate2::read::GzDecoder::new(Cursor::new(decoded)), encoding)?;
                decoded_any = true;
            }
            "deflate" => {
                decoded = read_limited(
                    flate2::read::ZlibDecoder::new(Cursor::new(decoded)),
                    encoding,
                )?;
                decoded_any = true;
            }
            "zstd" => {
                decoded = read_limited(
                    zstd::stream::read::Decoder::new(Cursor::new(decoded))
                        .map_err(|error| decode_error(encoding, error))?,
                    encoding,
                )?;
                decoded_any = true;
            }
            other => {
                return Err(AppError::BadRequest(format!(
                    "unsupported request content-encoding: {other}"
                )));
            }
        }
    }

    if decoded_any {
        headers.remove(header::CONTENT_ENCODING);
        headers.remove(header::CONTENT_LENGTH);
    }

    Ok(decoded)
}

fn content_encodings(headers: &HeaderMap) -> AppResult<Vec<String>> {
    let mut encodings = Vec::new();
    for value in headers.get_all(header::CONTENT_ENCODING) {
        let value = value
            .to_str()
            .map_err(|_| AppError::BadRequest("invalid request content-encoding header".into()))?;
        encodings.extend(
            value
                .split(',')
                .map(|encoding| encoding.trim().to_ascii_lowercase())
                .filter(|encoding| !encoding.is_empty()),
        );
    }
    Ok(encodings)
}

fn read_limited<R: Read>(reader: R, encoding: &str) -> AppResult<Bytes> {
    let mut reader = reader.take(MAX_DECODED_REQUEST_BODY_BYTES + 1);
    let mut out = Vec::new();
    reader
        .read_to_end(&mut out)
        .map_err(|error| decode_error(encoding, error))?;
    if out.len() as u64 > MAX_DECODED_REQUEST_BODY_BYTES {
        return Err(AppError::BadRequest(format!(
            "decoded request body exceeds {} bytes",
            MAX_DECODED_REQUEST_BODY_BYTES
        )));
    }
    Ok(Bytes::from(out))
}

fn decode_error(encoding: &str, error: std::io::Error) -> AppError {
    AppError::BadRequest(format!("invalid {encoding} request body: {error}"))
}
