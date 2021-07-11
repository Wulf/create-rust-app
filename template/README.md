# Getting Started with Create Rust App

This project was bootstrapped with [Create Rust App](https://github.com/wulf/create-rust-app).

## Requirements

- rustup, (Archlinux: `pacman -S rustup`)
- Rust stable, (bash: `rustup toolchain install stable`)
- Diesel CLI (after rust is installed: `cargo install diesel_cli`)

- Other helpful tools
  - `cargo install cargo-edit` (makes it easy to add deps with `cargo add`)
  - `cargo install cargo-watch` (allows running `cargo watch -x run -i app/` for continuous compilation)

- `.env` file (use `.env.example` for reference)

## Available Scripts

In the project directory, you can run:

### `cd app && yarn && yarn start`

Runs the frontend in the development mode.\
Visit [http://localhost:3000](http://localhost:3000) to view it.

The page will reload if you make edits to the frontend.\
You will also see any lint errors in the console.

### `cargo watch -x run -i app/`

Runs the backend in the development mode.\
Endpoints are hosted at [http://localhost:8080](http://localhost:8080).

The backend will recompile and restart when files change.
Needs `cargo-watch` installed, see requirements.

Note: you may need to run `cd app && yarn && yarn build` first in order for this command to work.

## Developer's note

When writing migrations, make sure the final schema structure is in the same order as Queryable model structs. Otherwise, diesel will fail to properly populate the struct.

### Deployment

TODO

## Database Migrations

* `diesel migration generate <migration_name>`
* `diesel migration run`
* `diesel migration revert`

* `diesel database setup`
* `diesel database reset`

## Development

2 processes are required; one for frontend continuous compilation, the other for backend.

```sh
cd app && yarn && yarn start
```

```sh
# you may need to run `cd app && yarn && yarn build` first!
cargo watch -x run -i app/
```

## Todo

* Implement a CSRF mitigation technique