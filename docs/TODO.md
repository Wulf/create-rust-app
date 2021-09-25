# Todo

- [ ] Validate project name
- [ ] Single binary build output for created projects (bundle assets in binary)
- [ ] Dockerfile
- [ ] Admin REPL: evcxr (see https://depth-first.com/articles/2020/09/21/interactive-rust-in-a-repl-and-jupyter-notebook-with-evcxr/)
- [ ] Help user setup their database & automatically populate .env with the DATABASE_URL (warn them that the password will be written to a file in plaintext)
- [ ] Look over unwrap()s in the auth plugin to make sure errors are handled
- [ ] The devbox plugin should not require the auth plugin (make an attempt at this)
- [ ] use web::block|| for service files, see https://actix.rs/docs/databases/ 
- [ ] Add create-rust-app build step which removes admin plugin's /.cargo/admin/node_modules
