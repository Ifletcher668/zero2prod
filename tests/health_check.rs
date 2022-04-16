use sqlx::{Connection, PgConnection};
use std::net::TcpListener;
use zero2prod::{configuration::get_configuration, startup::run};

/// Spin up instance of app and return its address
fn spawn_app() -> String {
  let listener =
    TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
  let port = listener.local_addr().unwrap().port();
  let server = run(listener).expect("Failed to bind address");
  let _ = tokio::spawn(server);

  format!("http://127.0.0.1:{}", port)
}

#[tokio::test]
async fn health_check_works() {
  let address = spawn_app();
  let client = reqwest::Client::new();
  let response = client
    .get(&format!("{}/health_check", &address))
    .send()
    .await
    .expect("Failed to execute request.");

  assert!(response.status().is_success());
  assert_eq!(Some(0), response.content_length())
}

#[tokio::test]
async fn subscribe_returns_200_when_data_is_present() {
  let app_address = spawn_app();
  let config = get_configuration().expect("Failed to read configuration");
  let connection_string = config.database.connection_string();
  let mut connection = PgConnection::connect(&connection_string)
    .await
    .expect("Failed to connect to Postgres.");
  let client = reqwest::Client::new();

  let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
  let response = client
    .post(&format!("{}/subscriptions", &app_address))
    .header("Content-Type", "application/x-www-form-urlencoded")
    .body(body)
    .send()
    .await
    .expect("failed to execute request");

  assert_eq!(200, response.status().as_u16());
  
  let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
    .fetch_one(&mut connection)
    .await
    .expect("Failed to fetch saved subscription.");
  
  assert_eq!(saved.email, "ursela_le_guin@gmail.com");
  assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_400_when_data_is_missing() {
  let app_address = spawn_app();
  let client = reqwest::Client::new();
  let test_cases = vec![
    ("name=le%20guin", "missing the email"),
    ("email=ursula_le_guin%40gmail.com", "missing the name"),
    ("", "missing both name and email"),
  ];

  for (invalid_body, error_message) in test_cases {
    let response = client
      .post(&format!("{}/subscriptions", &app_address))
      .header("Content-Type", "application/x-www-form-urlencoded")
      .body(invalid_body)
      .send()
      .await
      .expect("failed to execute request");

    assert_eq!(
      400,
      response.status().as_u16(),
      "The API did not fail with 400 Bad Request when the payload was {}.",
      error_message
    )
  }
}
