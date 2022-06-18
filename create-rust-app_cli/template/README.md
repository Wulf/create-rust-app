# Getting Started with Create Rust App

This project was bootstrapped with [Create Rust App](https://github.com/wulf/create-rust-app).

## Requirements

- rustup, (Archlinux: `pacman -S rustup`)
- Rust stable, (bash: `rustup toolchain install stable`)
- Diesel CLI (after rust is installed: `cargo install diesel_cli`)

- Other helpful tools

  - `cargo install cargo-edit` (makes it easy to add deps with `cargo add`)
  - `cargo install cargo-watch` (allows running `cargo watch -x run -i frontend/` for continuous compilation)

- `.env` file (use `.env.example` for reference)

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
