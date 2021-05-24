# Create Rust App

<a href="https://crates.io/crates/create-rust-app"><img src="https://img.shields.io/crates/v/create-rust-app.svg?style=for-the-badge" height="20" alt="License: MIT OR Apache-2.0" /></a>

 Set up a modern rust+react web app by running one command. 

# Features

* Project creation
  * Rust backend (using actix_web, diesel, r2d2)
  * Typescript frontend (using react)!
* Code-gen for easy CRUD scaffolding!

# Requirements

* [tsync](https://github.com/Wulf/tsync) (see install section)

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

# Scaffold CRUD for Todo model
create-rust-app --add resource todo
```