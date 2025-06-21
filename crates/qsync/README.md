Generated reat-query hooks for your services in actix-web, poem, axum, you-name-it.

Makes many assumptions!

- Works with a particular version of react-query (last tested on: "react-query": "^3.39.3")
- Endpoints return JSON (i.e. `fetch(..).json()` should work)
- Directory structure determines API paths!
- Just use `#[qsync]` above your method names.
- You can also specify a typescript return type like `#[qsync(return_type="string[]")]` or whether it's a mutation or not (`#[qsync(mutate)]`).
- When using a web framework like `actix_web`, make sure to declare the qsync attribute above above `#[get("..")]` or similar attributes which denote the endpoint's method and path. Rust evaluates macros from outer-most to the inner-most which means order is important!

See [https://github.com/Wulf/create-rust-app](https://github.com/Wulf/create-rust-app).
