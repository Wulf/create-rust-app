# Create Rust App

<a href="https://crates.io/crates/create-rust-app"><img src="https://img.shields.io/crates/v/create-rust-app.svg?style=for-the-badge" height="20" alt="License: MIT OR Apache-2.0" /></a>

 Set up a modern rust+react web app by running one command. 

# Requirements

* [`tsync`](https://github.com/Wulf/tsync) (see install section)
* `yarn` (or `npm`)
* Stable rust

# Install

```sh
cargo install tsync
cargo install create-rust-app
```

# Quick start

```sh
# Creates a new rust+react project
create-rust-app --project ./workspace/my-todo-app

cd my-todo-app

# Add authentication to your app
create-rust-app --add plugin auth

# Scaffold CRUD for a Note model
create-rust-app --add resource note
```

# Features

* Project creation (`create-rust-app --project my_project`)
  * Rust backend
    * Fastest backend server (via actix_web)
    * Database migrations (via diesel.rs)
    * Sending mail (via lettre)
    * PostgreSQL (via r2d2)
  * React frontend
    * Typescript, with backend type definition generation (via `tsync`)
    * Routing (via `react-router-dom`)
    * Update to latest create-react-app (generated frontend is not ejected from `create-react-app`)
* Resource creation (`create-rust-app --add resource UserRatings`)
  * CRUD code-gen to reduce boileplate
* Auth plugin (`create-rust-app --add plugin auth`)
  * Add JWT token-based auth with a simple command
  * Session management: restoration of previous session, revoking of refresh tokens
  * Credentials management/recovery
  * Email validation / activation flow
  * Adds frontend UI + react hooks
  * Adds auth service, and user / session models
  * Block your endpoints via `Auth` guard
  * Follows OWASP security best practices

# Walkthrough

[![Gif](docs/create-rust-app_fast.gif)](https://github.com/Wulf/create-rust-app/blob/main/docs/create-rust-app.mp4)
