# Todo

- [ ] EASY: write documentation (in another repo? create-rust-app-docs), maybe take `create-rust-app.dev`? (just like `create-react-app.dev`)
- [ ] FIX tsync script to work on all rust files (not inluding .build) -- currently, `backend/models/permissions/*` are ignored
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
- [ ] IMPORTANT: Write tests
  - [x] Frontend
  - [ ] Backend
- [ ] find cra project root based on Cargo.toml "[create-rust-app]" key
- [ ] EASY: auth plugin: change del_cookie to a removal cookie in actix and check poem's cookie jar del calls as well
- [ ] EASY: (this should be part of a larger effort to make it seem like there aren't many different projects that you need to learn about) Change `diesel_manage_updated_at` to `manage_updated_at` (it might confuse devs who aren't familiar with diesel)
- [ ] EASY: Change title from "React App" to "Create Rust App" with cra logo and update manifest for mobile installations
- [ ] Support multiple backend frameworks; for example,
      * actix_web
      * rocket
      * poem
      * axum
      * warp
- [ ] Implement a CSRF mitigation technique
- [ ] Fix created project's README.md
- [ ] Move package.json to root of project (instead of the `frontend/` directory)

# Done

- [x] EASY: run cargo fmt on all code (including `template/*` files)
- [x] Remove sentry crate
- [x] Dockerfile
- [x] Validate project name
- [x] Move `migrations` folder to `backend/migrations`

  **Result**: I attempted this but the diesel_cli doesn't respect what is written in `diesel.toml` (so something like `diesel database reset` doesn't work...):

  ```
  [migrations_directory]
  file="backend/migrations"
  dir="backend/migrations"
  ```
  
  From what I read, Diesel 2.x will respect this property and it hasn't landed yet in 1.4.8.

- [x] Admin REPL: evcxr (see https://depth-first.com/articles/2020/09/21/interactive-rust-in-a-repl-and-jupyter-notebook-with-evcxr/)

  **Result**: rust is too slow to be interpretted via a REPL. Just loading the project crate takes too long. I tried setting EVCXR_HOME (or whatever it was called) to the backend/.build directory since the project is likely compiled when developing but it didn't work out :'(

- [x] Single binary build output for created projects (bundle assets in binary)

  **Result**: Single binary build output isn't a good idea as it will make the template project much harder to go about and modify for some users
