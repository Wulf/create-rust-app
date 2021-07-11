# Create Rust App

<a href="https://crates.io/crates/create-rust-app"><img src="https://img.shields.io/crates/v/create-rust-app.svg?style=for-the-badge" height="20" alt="License: MIT OR Apache-2.0" /></a>

 Set up a modern rust+react web app by running one command. 

# Features

* Project creation (`create-rust-app --project my_project`)
  * Rust backend (using actix_web, diesel, r2d2)
  * Typescript frontend (using react)!
  * 1:1 synchronization of frontend <=> backend types (`rust <=> typescript`)
* Resource creation (`create-rust-app --add resource UserRatings`)
  * CRUD code-gen to reduce boileplate
* Auth plugin (`create-rust-app --add plugin auth`)
  * Add JWT token-based auth in a simple command

# Walkthrough

Video: 
https://github.com/Wulf/create-rust-app/tree/main/src/docs/create-rust-app.mp4

![Gif](docs/create-rust-app_fast.gif)

# Requirements

* [tsync](https://github.com/Wulf/tsync) (see install section)
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

# Todo

- [ ] Ensure current directory belongs to a project when adding plugins
- [ ] Single binary build output for created projects