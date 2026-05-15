use axum::http::StatusCode;

use crate::{error::CompactionHttpError, AppResult};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct CompactionLimits {
    pub max_carrier_items: usize,
    pub max_encrypted_content_bytes: usize,
    pub max_decrypted_pack_bytes: usize,
    pub max_source_items: usize,
    pub max_rendered_tokens: usize,
    pub max_compactor_input_bytes: usize,
}

impl Default for CompactionLimits {
    fn default() -> Self {
        Self {
            max_carrier_items: 1,
            max_encrypted_content_bytes: 1_048_576,
            max_decrypted_pack_bytes: 2_097_152,
            max_source_items: 2_000,
            max_rendered_tokens: 16_384,
            max_compactor_input_bytes: 4_194_304,
        }
    }
}

impl CompactionLimits {
    pub fn check_carrier_count(self, count: usize) -> AppResult<()> {
        if count > self.max_carrier_items {
            return Err(CompactionHttpError::new(
                StatusCode::BAD_REQUEST,
                "too_many_compaction_items",
                "invalid_request",
                format!("request contains {count} compaction items; at most one is allowed"),
            )
            .into());
        }
        Ok(())
    }

    pub fn check_encrypted_content(self, encrypted_content: &str) -> AppResult<()> {
        if encrypted_content.len() > self.max_encrypted_content_bytes {
            return Err(CompactionHttpError::new(
                StatusCode::PAYLOAD_TOO_LARGE,
                "compaction_pack_too_large",
                "invalid_request",
                "compaction encrypted_content exceeds the configured byte limit",
            )
            .into());
        }
        Ok(())
    }

    pub fn check_decrypted_pack(self, decrypted_pack: &[u8]) -> AppResult<()> {
        if decrypted_pack.len() > self.max_decrypted_pack_bytes {
            return Err(CompactionHttpError::new(
                StatusCode::PAYLOAD_TOO_LARGE,
                "compaction_pack_too_large",
                "invalid_request",
                "decrypted UMP compaction pack exceeds the configured byte limit",
            )
            .into());
        }
        Ok(())
    }
}
