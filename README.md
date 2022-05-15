# <img src="https://user-images.githubusercontent.com/4259838/150465966-7ac954d1-9f0c-48d4-a37a-10543b3bbfe1.png" height="40px"> Create Rust App

<a href="https://crates.io/crates/create-rust-app"><img src="https://img.shields.io/crates/v/create-rust-app.svg?style=for-the-badge" height="20" alt="License: MIT OR Apache-2.0" /></a>

Set up a modern rust+react web app by running one command.

[create-rust-app.dev](https://create-rust-app.dev)

# Requirements

- [`tsync`](https://github.com/Wulf/tsync)
  - ```cargo install tsync```
- `yarn`
  - ```npm i -g yarn```
- Stable rust
  - ```rustup install stable``` (nightly is fine too)

# Install

```sh
cargo install create-rust-app_cli
```

# Quick start

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

# Features

### 1. Project creation
```
$ create-rust-app <project_name>
```

  - Run frontend & backend with a single command: `cargo fullstack`
  - Rust backend
    - One of the following frameworks: `actix-web`, `poem` or let us know which one you want to use!
    - Database migrations (using diesel.rs)
    - Sending mail
    - PostgreSQL (but you can easily switch to another one!)
    - ViteJS (blazing fast frontend compile speeds)
    - SSR templating with an option to include bundles that are automatically code-split
    - Automatically route to your single page application(s)
  - React frontend (or install your own framework!)
    - Typescript, with backend type definition generation (via `tsync`)
    - Routing (via `react-router-dom`)
    - Typed `react-query` hooks generation (`$ cd my_project && create-rust-app`, then select "Generate react-query hooks")
    - Update to latest create-react-app (generated frontend is not ejected from `create-react-app`)

**Available Plugins**

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
  - View your database via the admin portal at `localhost:8080/admin` (editing functionality coming soon™)
  - A "devbox" on the frontend indicates when the backend is compiling or when the database is not reachable
  - Moreover, the devbox displays when migrations are pending + includes a "run migrations" button
    

- **Storage plugin**
  - Adds `Storage` extractor which allows you to upload/download files from an S3-compatible object store 
  - Seamlessly add single or multiple attachments to your models using `Attachment::*`!
  - Here are some examples:
    - Adding an avatar to a user in your users table: 
    ```rs
    let s3_key = Attachment::attach("avatar", "users", user_id, AttachmentData {
        file_name: "image.png",
        data: bytes
    })?;
    ```
    - Getting the url for the attachment
    ```rs
    let storage: Storage // retreive this via the appropriate extractor in your frameowrk of choice
    let url = storage.download_uri(s3_key)?;
    ```
    (note: see `Attachment::*` and `Storage::*` for more functionality!)
    

- **GraphQL plugin**
  - Adds all the boilerplate necessary to expose GraphQL
  - Requires the auth plugin: authentication and authorization setup out-of-the-box
  - Find a graphql playground at `localhost:8080/graphql`


### 2. Code-gen to reduce boilerplate
```
$ cd my_project && create-rust-app
```
  - CRUD code-gen to reduce boilerplate
    - Scaffolds the db model, endpoints service file, and hooks it up in your `/api`! 
  - `react-query` hooks generation for frontend
    - Generates a hook for each handler function defined in the `services/` folder 
    - Edit generated hooks afterwards -- they won't be regenerated unless you delete (or rename) the hook! 

# Walkthrough

[![Gif](docs/create-rust-app-v2.gif)](https://github.com/Wulf/create-rust-app/blob/main/docs/create-rust-app-v2.mp4)

# Contributing

Questions and comments are welcome in the issues section! 

If you're experiencing slow compilation time, make sure there isn't any bloat in the template files (look for `node_modules` or typescript / parcel caches and delete them).