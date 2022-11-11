# Getting Started with Create Rust App

This project was bootstrapped with [Create Rust App](https://github.com/wulf/create-rust-app).

## Requirements

- [stable Rust](https://www.rust-lang.org/tools/install)
- Diesel CLI 
  - if using postgres, `cargo install diesel_cli --no-default-features --features postgres`
  - if using sqlite, `cargo install diesel_cli --no-default-features --features sqlite-bundled`
- cargo-watch to recompile on change:
  - `cargo install cargo-watch` (allows running `cargo watch -x run -i frontend/` for continuous compilation; see "available scripts")

## Notes

- In development, the `.env` file is read (use `.env.example` for reference)
- In production, environment variables are sourced directly

## Available Scripts

In the project directory, you can run:

### `cargo fullstack`

Runs the app in development mode and watches for changes. Visit [http://localhost:3000](http://localhost:3000) to view it.

Any frontend changes should instantly appear. Backend changes will need to recompile.
Needs `cargo-watch` installed, see requirements.

To test/debug issues with the production build, set the `debug-assertions` to `true` for `[profile.dev]` in `Cargo.toml`. This way, development-only code paths are discarded and instead, production-only code paths are included.

Alternatively, use `cargo run` to run the app in development mode without watching for file changes.

### `cargo build`

Builds a production-ready build.

### `cargo tsync`

Generates the typescript types from rust code marked with [`tsync`](https://github.com/Wulf/tsync).
Outputs to `frontend/src/types/rust.d.ts`.

### Running frontend and backend individually

```sh
# frontend
cd frontend && yarn && yarn start
```

```sh
# backend
cargo watch -x run -i frontend/
```

## Database Migrations

- `diesel migration generate <migration_name>`
- `diesel migration run`
- `diesel migration revert`

- `diesel database setup`
- `diesel database reset`
