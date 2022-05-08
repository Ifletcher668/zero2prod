use crate::helpers::error_chain_fmt;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

#[tracing::instrument(
  name = "Store subscription token in the database",
  skip(subscription_token, transaction)
)]
pub async fn store_token(
  transaction: &mut Transaction<'_, Postgres>,
  subscriber_id: Uuid,
  subscription_token: &str,
) -> Result<(), StoreTokenError> {
  sqlx::query!(
    r#"INSERT INTO subscription_tokens (subscription_token, subscriber_id) VALUES ($1, $2)"#,
    subscription_token,
    subscriber_id,
  )
  .execute(transaction)
  .await
  .map_err(|e| StoreTokenError(e))?;

  Ok(())
}

pub fn generate_subscription_token() -> String {
  let mut rng = thread_rng();

  std::iter::repeat_with(|| rng.sample(Alphanumeric))
    .map(char::from)
    .take(25)
    .collect()
}

pub struct StoreTokenError(sqlx::Error);

impl std::error::Error for StoreTokenError {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    Some(&self.0)
  }
}

impl std::fmt::Display for StoreTokenError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "A database failure was encountered while trying to store a subscription token."
    )
  }
}

impl std::fmt::Debug for StoreTokenError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    error_chain_fmt(self, f)
  }
}
