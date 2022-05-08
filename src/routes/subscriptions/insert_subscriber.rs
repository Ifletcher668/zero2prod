use super::FormData;
use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};
use chrono::Utc;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

#[tracing::instrument(
  name = "Saving new subscriber details in the database",
  skip(new_subscriber, transaction)
)]
pub async fn insert_subscriber(
  new_subscriber: &NewSubscriber,
  transaction: &mut Transaction<'_, Postgres>,
) -> Result<Uuid, sqlx::Error> {
  let NewSubscriber { email, name } = new_subscriber;
  let subscriber_id = Uuid::new_v4();

  sqlx::query!(
    r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at, status)
    VALUES ($1, $2, $3, $4, 'PENDING')
    "#,
    subscriber_id,
    email.as_ref(),
    name.as_ref(),
    Utc::now()
  )
  .execute(transaction)
  .await?;

  Ok(subscriber_id)
}

impl TryFrom<FormData> for NewSubscriber {
  type Error = String;
  fn try_from(value: FormData) -> Result<Self, Self::Error> {
    let name = SubscriberName::parse(value.name)?;
    let email = SubscriberEmail::parse(value.email)?;
    Ok(Self { email, name })
  }
}
