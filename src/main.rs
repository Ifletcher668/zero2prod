use secrecy::ExposeSecret;
use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> std::io::Result<()> {
  let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
  init_subscriber(subscriber);

  let config = get_configuration().expect("Failed to read configuration.");
  let connection_pool = PgPoolOptions::new()
    .connect_timeout(std::time::Duration::from_secs(2))
    .connect_lazy(config.database.with_db());
  let address = format!("{}:{}", config.application.host, config.application.port);
  let listener = TcpListener::bind(address)?;

  run(listener, connection_pool)?.await
}
