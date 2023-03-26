# <img src="https://user-images.githubusercontent.com/4259838/150465966-7ac954d1-9f0c-48d4-a37a-10543b3bbfe1.png" height="40px"> Create Rust App

<!-- markdownlint-disable-file MD033 -->

<a href="https://crates.io/crates/create-rust-app"><img src="https://img.shields.io/crates/v/create-rust-app.svg?style=for-the-badge" height="20" alt="License: MIT OR Apache-2.0" /></a>

Set up a modern rust+react web app by running one command. [Join us on discord](https://discord.gg/tm6Ey33ZPN).

[create-rust-app.dev](https://create-rust-app.dev)

## Requirements

- [Rust](https://www.rust-lang.org/tools/install)
- [`diesel_cli`](http://diesel.rs/guides/getting-started#installing-diesel-cli)
  - For SQLite, if you don't wish to dynamically link `diesel_cli` with your system's `libsqlite3`, you may run `cargo install diesel_cli --no-default-features --features sqlite-bundled`.

## Install

```sh
cargo install create-rust-app_cli
```

## Quick start

```sh
create-rust-app my-todo-app
# .. select backend framework, plugins, etc.
```

```sh
# Code-gen resources for your project
cd ./my-todo-app
create-rust-app
# .. select resource type / properties
```

## Features

### 1. Project creation

```rust
create-rust-app create <project_name>
```

- Run frontend & backend with a single command: `cargo fullstack`
- Rust backend
  - One of the following frameworks: `actix-web`, `poem` or let us know which one you want to use!
  - Database migrations (using diesel.rs)
    - Generate diesel structs and types by running `cargo dsync` in your project (see codegen section below).
  - Sending mail
  - PostgreSQL, SQLite 3.35+ support
  - ViteJS (blazing fast frontend compile speeds)
  - SSR templating with an option to include bundles that are automatically code-split
    - The `/views` folder contains all templates
    - The `/frontend/bundles` folder contains all the bundles which can be included in your views via `{{bundle(name="MyBundle.tsx")}}`
  - Automatically route to your single page application(s)
    - Use `create_rust_app::render_single_page_application("/app","your_spa.html")`
- React frontend (or install your own framework!)
  - Typescript, with backend type definition generation (run `cargo tsync` in your project folder; see codegen section below)
  - Routing (via `react-router-dom`)
  - Typed `react-query` hooks generation (`$ cd my_project && create-rust-app`, then select "Generate react-query hooks")

#### Available Plugins

- **Authentication (+ Authorization) plugin**
  - Add JWT token-based auth with a simple command
  - Session management: restoration of previous session, revoking of refresh tokens
  - Credentials management/recovery
  - Email validation / activation flow
  - Adds frontend UI + react hooks
  - Adds auth service, and user / session models
  - Block your endpoints via `Auth` guard
  - Follows OWASP security best practices
  - RBAC permissions out of the box (assign roles and permissions to users)

- **Container plugin**
  - Dockerfile to containerize your rust app into a single image

- **Development plugin**
  - View your database via the admin portal at `localhost:3000/admin` (still in development)
  - A "devbox" on the frontend indicates when the backend is compiling or when the database is not reachable
  - Moreover, the devbox displays when migrations are pending + includes a "run migrations" button
  - In-browser compilation errors and migration checking:
    <a href="https://user-images.githubusercontent.com/4259838/218256539-b94ecba1-abe6-4e42-b4f4-4d80b6d4079b.png"><img src="https://user-images.githubusercontent.com/4259838/218256539-b94ecba1-abe6-4e42-b4f4-4d80b6d4079b.png" width="650px" /></a>
    <a href="https://user-images.githubusercontent.com/4259838/218256539-b94ecba1-abe6-4e42-b4f4-4d80b6d4079b.png"><img src="https://user-images.githubusercontent.com/4259838/218256528-4b6ca2a4-ffae-4c9e-bc20-c4a483355b01.png" width="650px" /></a>

- **Storage plugin**
  - Adds `Storage` extractor which allows you to upload/download files from an S3-compatible object store
  - Seamlessly add single or multiple attachments to your models using `Attachment::*`!
  - Here are some examples:
    - Adding an avatar to a user in your users table:

    ```rust
    let s3_key = Attachment::attach("avatar", "users", user_id, AttachmentData {
        file_name: "image.png",
        data: bytes
    })?;
    ```

    - Getting the url for the attachment

    ```rust
    let storage: Storage // retreive this via the appropriate extractor in your frameowrk of choice
    let url = storage.download_uri(s3_key)?;
    ```

    (note: see `Attachment::*` and `Storage::*` for more functionality!)

- **GraphQL plugin**
  - Adds all the boilerplate necessary to expose GraphQL
  - Requires the auth plugin: authentication and authorization setup out-of-the-box
  - Find a graphql playground at `localhost:3000/graphql`

- **Utoipa plugin**
  - Uses the [utoipa](https://github.com/juhaku/utoipa) crate to add OpenAPI documentation and serve it in a SwaggerUI playground.
  - Find the playground at `localhost:3000/swagger-ui`
  - Requires the backend be Actix (for now ;) )
  - check out [this page](https://github.com/juhaku/utoipa/tree/master/examples) to see how to document your own API endpoints with a variety of backends

- **Tasks Plugin**
  - For running background jobs, currently only supports actix-web and postgresql
  - Uses [`fang`](https://github.com/ayrat555/fang) under the hood and all it's features are exposed. 
  - Add a task to the queue with `create_rust_app::tasks::queue()`
  - Run the queue with `cargo run --bin tasks`

### 2. Code-gen to reduce boilerplate

````sh
cargo dsync
````

- Run this commmand to generate diesel model structs and queries in your `backend/models` folder!
- See dsync docs [here](https://github.com/Wulf/dsync)

```sh
cargo tsync
```

- Run this command to generate typescript types for your rust code marked with `#[tsync::tsync]`. You'll find the output for this command here: `frontend/src/types/rust.d.ts`.
- See tsync docs [here](https://github.com/Wulf/tsync)

```sh
cd my_project && create-rust-app
```

- CRUD code-gen to reduce boilerplate
  - Scaffolds the db model, endpoints service file, and hooks it up in your `/api`!
- `react-query` hooks generation for frontend
  - Generates a hook for each handler function defined in the `services/` folder
  - Edit generated hooks afterwards -- they won't be regenerated unless you delete (or rename) the hook!

## Walkthrough (old)

[![Gif](docs/create-rust-app-v2.gif)](https://github.com/Wulf/create-rust-app/blob/main/docs/create-rust-app-v2.mp4)

## Contributing

Questions and comments are welcome in the issues section!

If you're experiencing slow compilation time, make sure there isn't any bloat in the template files (look for `node_modules` or typescript / parcel caches and delete them).
Moreover, you can try using the [mold](https://github.com/rui314/mold) linker which may also improve compilation times.
