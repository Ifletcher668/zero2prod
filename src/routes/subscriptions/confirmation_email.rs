use crate::{domain::NewSubscriber, email_client::EmailClient};

#[tracing::instrument(
  name = "Sending a confirmation email to a new subscriber",
  skip(email_client, new_subscriber, base_url, subscription_token)
)]
pub async fn send_confirmation_email(
  email_client: &EmailClient,
  new_subscriber: NewSubscriber,
  base_url: &str,
  subscription_token: &str,
) -> Result<(), reqwest::Error> {
  let confirmation_link = format!(
    "{}/subscriptions/confirm?subscription_token={}",
    base_url, subscription_token
  );
  let html_body = format!(
    "Welcome to our newsletter!<br />\
    Click <a href=\"{}\">here</a> to confirm your subscription.",
    confirmation_link
  );
  let plain_body = format!(
    "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
    confirmation_link
  );

  email_client
    .send_email(new_subscriber.email, "Welcome!", &html_body, &plain_body)
    .await
}
