# Todo

- [ ] EASY: Help user setup their database & automatically populate .env with the DATABASE_URL (warn them that the password will be written to a file in plaintext)
  - [ ] Create 'superadmin' role+user as a step in project creation
- [ ] EASY: Look over unwrap()s in the auth plugin to make sure errors are handled
- [ ] The devbox plugin should not require the auth plugin (make an attempt at this)
- [ ] EASY: use web::block|| for service files, see https://actix.rs/docs/databases/
- [ ] Add create-rust-app build step which removes admin plugin's /.cargo/admin/node_modules
- [ ] Frontend: Add eslint, add prettier
- [ ] Move `bin/*` into `.cargo/*`?
- [ ] Move `target-dir="backend/.build"` into `.cargo`
- [ ] IMPORTANT: Write backend tests (poem + actix_web)
- [ ] EASY: set device to 'web' for login demo
- [ ] find cra project root based on Cargo.toml "[create-rust-app]" key
- [ ] EASY: (this should be part of a larger effort to make it seem like there aren't many different projects that you need to learn about) Change `diesel_manage_updated_at` to `manage_updated_at` (it might confuse devs who aren't familiar with diesel)
- [ ] EASY: Change title from "React App" to "Create Rust App" with cra logo and update manifest for mobile installations
- [ ] Support multiple backend frameworks; for example,
  - [x] actix_web
  - [ ] rocket
  - [x] poem
  - [ ] axum
  - [ ] warp
- [ ] Implement a CSRF mitigation technique
- [ ] Fix created project's README.md
- [ ] EASY: fix activation page on frontend (fetch token from url params, introduce a useQueryParam() hook)
- [ ] EASY: Don't let users create packages named 'test' or names that start with a digit (cargo init fails): 
  - ```error: the name `test` cannot be used as a package name, it conflicts with Rust's built-in test library```,
  - ```error: the name `123` cannot be used as a package name, the name cannot start with a digit```
- [ ] EASY: Add cargo build step before cargo fullstack so the concurrent process isn't started until we're sure that the backend builds.
- [ ] EASY: split frontend cra logic (like the useAuth hook) into a separate package (create-rust-app on npm)
- [ ] EASY: remove redundant `useHistory()` in `App.tsx`
- [ ] EASY: check CLI version and inform user to update / upgrade accordingly (use: https://crates.io/crates/update-informer) 
- [ ] EASY: add `cargo tsync` command which calls `yarn tsync` in frontend/

# To be revisited

- [x] Move package.json to root of project (instead of the `frontend/` directory)

  **Result**: react-scripts, including yarn start/build/eject would not run, even with something like `cd frontend && yarn start` because `react-scripts` expected a `package.json` in the `frontend/` folder.

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
