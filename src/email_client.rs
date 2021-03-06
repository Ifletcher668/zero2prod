use reqwest::Client;
use secrecy::{ExposeSecret, Secret};

use crate::domain::SubscriberEmail;

pub struct EmailClient {
  sender: SubscriberEmail,
  http_client: Client,
  base_url: String,
  mailgun_api_key: Secret<String>,
}

#[derive(serde::Serialize)]
struct SendEmailRequest<'a> {
  from: &'a str,
  to: &'a str,
  subject: &'a str,
  html: &'a str,
  text: &'a str,
}

pub const EMAIL_ENDPOINT: &str = "messages";

impl EmailClient {
  pub fn new(
    base_url: String,
    sender: SubscriberEmail,
    mailgun_api_key: Secret<String>,
    timeout: std::time::Duration,
  ) -> Self {
    let http_client = Client::builder().timeout(timeout).build().unwrap();

    Self {
      http_client,
      base_url,
      sender,
      mailgun_api_key,
    }
  }

  pub async fn send_email(
    &self,
    recipient: SubscriberEmail,
    subject: &str,
    html: &str,
    text: &str,
  ) -> Result<(), reqwest::Error> {
    let url = format!("{}/{}", self.base_url, EMAIL_ENDPOINT);
    let request_body = SendEmailRequest {
      from: self.sender.as_ref(),
      to: recipient.as_ref(),
      subject,
      html,
      text,
    };

    self
      .http_client
      .post(&url)
      .basic_auth("api", Some(self.mailgun_api_key.expose_secret()))
      .json(&request_body)
      .send()
      .await?
      .error_for_status()?;

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::{EmailClient, EMAIL_ENDPOINT};
  use crate::domain::SubscriberEmail;
  use claim::{assert_err, assert_ok};
  use fake::faker::lorem::en::{Paragraph, Sentence};
  use fake::{faker::internet::en::SafeEmail, Fake, Faker};
  use secrecy::Secret;
  use wiremock::matchers::{any, header, method, path};
  use wiremock::{Mock, MockServer, Request, ResponseTemplate};

  struct SendEmailBodyMatcher;

  impl wiremock::Match for SendEmailBodyMatcher {
    fn matches(&self, request: &Request) -> bool {
      let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);

      if let Ok(body) = result {
        body.get("from").is_some()
          && body.get("to").is_some()
          && body.get("subject").is_some()
          && body.get("html").is_some()
          && body.get("text").is_some()
      } else {
        false
      }
    }
  }

  /// Generate a random email subject
  fn subject() -> String {
    Sentence(1..2).fake()
  }
  /// Generate a random email content
  fn content() -> String {
    Paragraph(1..10).fake()
  }
  /// Generate a random subscriber email
  fn email() -> SubscriberEmail {
    SubscriberEmail::parse(SafeEmail().fake()).unwrap()
  }
  /// Get a test instance of `EmailClient`.
  fn email_client(base_url: String) -> EmailClient {
    EmailClient::new(
      base_url,
      email(),
      Secret::new(Faker.fake()),
      std::time::Duration::from_millis(200),
    )
  }

  #[tokio::test]
  async fn send_email_fires_a_request_to_base_url() {
    let mock_server = MockServer::start().await;
    let email_client = email_client(mock_server.uri());

    Mock::given(header("Content-Type", "application/json"))
      .and(path(format!("/{}", EMAIL_ENDPOINT)))
      .and(method("POST"))
      .and(SendEmailBodyMatcher)
      .respond_with(ResponseTemplate::new(200))
      .expect(1)
      .mount(&mock_server)
      .await;

    let _ = email_client
      .send_email(email(), &subject(), &content(), &content())
      .await;
  }

  #[tokio::test]
  async fn send_email_succeeds_if_the_server_returns_200() {
    let mock_server = MockServer::start().await;
    let email_client = email_client(mock_server.uri());

    Mock::given(any())
      .respond_with(ResponseTemplate::new(200))
      .expect(1)
      .mount(&mock_server)
      .await;

    let outcome = email_client
      .send_email(email(), &subject(), &content(), &content())
      .await;

    assert_ok!(outcome);
  }

  #[tokio::test]
  async fn send_email_times_out_if_the_server_takes_too_long() {
    let mock_server = MockServer::start().await;
    let email_client = email_client(mock_server.uri());

    Mock::given(any())
      .respond_with(ResponseTemplate::new(500))
      .expect(1)
      .mount(&mock_server)
      .await;

    let outcome = email_client
      .send_email(email(), &subject(), &content(), &content())
      .await;

    assert_err!(outcome);
  }
}
