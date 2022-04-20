# zero2prod Learning Environment
This repository houses my code as I follow the [zero2production](https://www.zero2prod.com/) book.

## Tools used:

- rust-analyzer
- tarpaulin (code coverage)
- clippy (the official Rust linter)
- rustfmt (the official Rust formatter)
- bunyan (log formatter)
## Database
PostgreSQL

sqlx

## CI 
This is a placeholder to test CI/CD on my first branch

## Logging
For Full logging, use the RUST_LOG environment variable

To run a prettified, full logging of the test suite, run:
```
  TEST_LOG=true cargo test health_check_works | bunyan
```

To run a prettified log of the runtime environment, run:
```
  RUST_LOG=trace cargo run | bunyan
```