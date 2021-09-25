# Create Rust App

<a href="https://crates.io/crates/create-rust-app"><img src="https://img.shields.io/crates/v/create-rust-app.svg?style=for-the-badge" height="20" alt="License: MIT OR Apache-2.0" /></a>

 Set up a modern rust+react web app by running one command. 

# Requirements

* [`tsync`](https://github.com/Wulf/tsync) (see install section)
* `yarn` (or `npm`)
* Stable rust

# Install

```sh
cargo install create-rust-app
```

# Quick start

```sh
# Creates a new rust+react project
create-rust-app my-todo-app
# .. select plugins, etc.

# Code-gen resources for your project
cd ./my-todo-app
create-rust-app 
# .. select resource type / properties
```

# Features

* Project creation (`$ create-rust-app <project_name>`)
  * Run frontend & backend with a single command: `cargo fullstack`
  * Rust backend
    * Fastest backend server (via actix_web)
    * Database migrations (via diesel.rs)
    * Sending mail (via lettre)
    * PostgreSQL (via r2d2)
  * React frontend
    * Typescript, with backend type definition generation (via `tsync`)
    * Routing (via `react-router-dom`)
    * Update to latest create-react-app (generated frontend is not ejected from `create-react-app`)
* Resource creation (`$ cd my_project && create-rust-app`)
  * CRUD code-gen to reduce boileplate
* Auth plugin
  * Add JWT token-based auth with a simple command
  * Session management: restoration of previous session, revoking of refresh tokens
  * Credentials management/recovery
  * Email validation / activation flow
  * Adds frontend UI + react hooks
  * Adds auth service, and user / session models
  * Block your endpoints via `Auth` guard
  * Follows OWASP security best practices
* Container plugin
  * Dockerfile to containerize your rust app into a single image
* Admin Portal plugin
  * View your database via the admin portal (editing functionality coming soon™)
  * A "devbox" on the frontend indicates when the backend is compiling or when the database is not reachable
  * Moreover, the devbox displays when migrations are pending + includes a "run migrations" button

# Walkthrough

(the full video can be found in the repo at this path: [`docs/create-rust-app-v2.mp4`](https://github.com/Wulf/create-rust-app/blob/main/docs/create-rust-app-v2.mp4)
)

[![Gif](docs/create-rust-app-v2.gif)](https://github.com/Wulf/create-rust-app/blob/main/docs/create-rust-app-v2.mp4)
