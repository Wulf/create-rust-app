# Create Rust App

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