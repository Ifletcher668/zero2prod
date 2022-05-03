use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};
use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
  email: String,
  name: String,
}

#[tracing::instrument(
  name = "Adding a new subscriber",
  skip(form, pool),
  fields(
    subscriber_email = %form.email,
    subscriber_name = %form.name
  )
)]
pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
  let new_subscriber = match form.0.try_into() {
    Ok(form) => form,
    Err(_) => return HttpResponse::BadRequest().finish(),
  };

  match insert_subscriber(&new_subscriber, &pool).await {
    Ok(_) => HttpResponse::Ok().finish(),
    Err(_) => HttpResponse::InternalServerError().finish(),
  }
}

impl TryFrom<FormData> for NewSubscriber {
  type Error = String;
  fn try_from(value: FormData) -> Result<Self, Self::Error> {
    let name = SubscriberName::parse(value.name)?;
    let email = SubscriberEmail::parse(value.email)?;
    Ok(Self { email, name })
  }
}

#[tracing::instrument(
  name = "Saving new subscriber details in the database",
  skip(new_subscriber, pool)
)]
pub async fn insert_subscriber(
  new_subscriber: &NewSubscriber,
  pool: &PgPool,
) -> Result<(), sqlx::Error> {
  let NewSubscriber { email, name } = new_subscriber;

  sqlx::query!(
    r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at, status)
    VALUES ($1, $2, $3, $4, 'CONFIRMED')
    "#,
    Uuid::new_v4(),
    email.as_ref(),
    name.as_ref(),
    Utc::now()
  )
  .execute(pool)
  .await
  .map_err(|e| {
    tracing::error!("Failed to execute query: {:?}", e);
    e
  })?;

  Ok(())
}
