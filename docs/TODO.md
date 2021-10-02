# Todo

- [ ] EASY: write documentation (in another repo? create-rust-app-docs), maybe take `create-rust-app.dev`? (just like `create-react-app.dev`)
- [ ] FIX tsync script to work on all rust files (not inluding .build) -- currently, `backend/models/permissions/*` are ignored
- [ ] EASY: run cargo fmt on all code (including `template/*` files)
- [ ] EASY: Help user setup their database & automatically populate .env with the DATABASE_URL (warn them that the password will be written to a file in plaintext)
  - [ ] Create 'superadmin' role+user as a step in project creation
- [ ] EASY: Look over unwrap()s in the auth plugin to make sure errors are handled
- [ ] The devbox plugin should not require the auth plugin (make an attempt at this)
- [ ] EASY: use web::block|| for service files, see https://actix.rs/docs/databases/
- [ ] Add create-rust-app build step which removes admin plugin's /.cargo/admin/node_modules
- [ ] Frontend: Add eslint, add prettier
- [ ] EASY: Admin plugin: expose roles, make frontend useAuth() hook expose methods like hasPermission() hasRole()
- [ ] Remove all plugins, just have a single template which builds the project
- [ ] Move `bin/*` into `.cargo/*`
- [ ] Move `target-dir="backend/.build"` into `.cargo`

# Done

- [x] Remove sentry crate
- [x] Dockerfile
- [x] Validate project name

# Needs thought

- [x] Move `migrations` folder to `backend/migrations`

  NOTE: I attempted this but the diesel_cli doesn't respect what is written in `diesel.toml` (so something like `diesel database reset` doesn't work...):

  ```
  [migrations_directory]
  file="backend/migrations"
  dir="backend/migrations"
  ```

- [x] Admin REPL: evcxr (see https://depth-first.com/articles/2020/09/21/interactive-rust-in-a-repl-and-jupyter-notebook-with-evcxr/)

  NOTE: rust is too slow to be interpretted via a REPL. Just loading the project crate takes too long. I tried setting EVCXR_HOME (or whatever it was called) to the backend/.build directory since the project is likely compiled when developing but it didn't work out :'(

- [ ] Single binary build output for created projects (bundle assets in binary)

  NOTE: Single binary build output isn't a good idea as it will make the template project much harder to go about and modify for some users
