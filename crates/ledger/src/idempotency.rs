use sha2::{Digest, Sha512};
use sqlx::*;
use std::time::SystemTime;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IdempotencyKey {
    pub uuid: Uuid,
}

#[derive(Debug, sqlx::Type, Clone)]
#[sqlx(type_name = "idempotency_status", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IdempotencyStatus {
    InFlight,
    Failed,
    Completed,
}

#[derive(Debug, Clone)]
pub struct IdempotencyRecord {
    pub key: IdempotencyKey,
    pub payload_hash: String,
    pub status: IdempotencyStatus,
    pub created_at: SystemTime,
}

#[derive(Debug)]
pub enum IdempotencyCheckOutcome {
    Proceed,
    CachedResponse { status_code: u16, body: String },
    InFlight,
    PayloadMismatch,
}

pub struct IdempotencyEngine {
    pool: PgPool,
}

impl IdempotencyEngine {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub fn hash_payload(payload_bytes: &[u8]) -> String {
        let mut hasher = Sha512::new();
        hasher.update(payload_bytes);

        let hash = hex::encode(hasher.finalize());

        hash
    }
    pub async fn check_and_lock_idempotency(
        &self,
        key: IdempotencyKey,
        payload_hash: &str,
    ) -> Result<IdempotencyCheckOutcome, String> {
        let inserted = sqlx::query(
            r#"INSERT INTO idempotency_keys (key, payload_hash, status, created_at)
       VALUES ($1, $2, 'IN_FLIGHT', NOW()) ON CONFLICT (key) DO NOTHING
       RETURNING key"#,
        )
        .bind(key.uuid)
        .bind(payload_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        if inserted.is_some() {
            return Ok(IdempotencyCheckOutcome::Proceed);
        }

        let existing = sqlx::query(
            r#"
SELECT payload_hash, status AS "status: IdempotencyStatus", status_code, response_body,
       FROM idempotency_keys
       WHERE key = $1"#,
        )
        .bind(key.uuid)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        use sqlx::Row;
        let db_payload_hash: String = existing.try_get("payload_hash").unwrap_or_default();

        if db_payload_hash != payload_hash {
            return Ok(IdempotencyCheckOutcome::PayloadMismatch);
        }

        let status_str: String = existing.try_get("status").unwrap_or_default();
        let status_code: Option<i32> = existing.try_get("status_code").ok();
        let response_body: Option<String> = existing.try_get("response_body").ok();

        match status_str.as_str() {
            "IN_FLIGHT" => Ok(IdempotencyCheckOutcome::InFlight),
            "COMPLETED" | "FAILED" => Ok(IdempotencyCheckOutcome::CachedResponse {
                status_code: status_code.unwrap_or(200) as u16,
                body: response_body.unwrap_or_default(),
            }),
            _ => Err("Unknown status in database".to_string()),
        }
    }

    pub async fn complete_within_tx(
        tx: &mut Transaction<'_, Postgres>,
        key: Uuid,
        status_code: u16,
        response_body: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE idempotency_keys
            SET status = 'COMPLETED',
                response_code = $2,
                response_body = $3
            WHERE idempotency_key = $1
            "#,
        )
        .bind(key)
        .bind(status_code as i16)
        .bind(response_body)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }
}
