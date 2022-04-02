# Todo

- [ ] EASY: Help user setup their database & automatically populate .env with the DATABASE_URL (warn them that the password will be written to a file in plaintext)
  - potentially show steps to boot up DB using docker
- [ ] Make sure users can switch databases! 
- [ ] EASY: Look over unwrap()s in the auth plugin to make sure errors are handled
- [ ] The devbox plugin should not require the auth plugin (make an attempt at this)
- [ ] EASY: use web::block|| for services that access the db, see https://actix.rs/docs/databases/
- [ ] Add create-rust-app build step which removes admin plugin's /.cargo/admin/node_modules
- [ ] Frontend: Add eslint, add prettier
- [ ] find cra project root based on Cargo.toml "[create-rust-app]" key
- [ ] EASY: (this should be part of a larger effort to make it seem like there aren't many different projects that you need to learn about) Change `diesel_manage_updated_at` to `manage_updated_at` (it might confuse devs who aren't familiar with diesel)
- [ ] EASY: Change title from "React App" to "Create Rust App" with cra logo and update manifest for mobile installations
- [ ] Option to not have the default template (instead, just show a blank page like create-react-app)
- [ ] Implement a CSRF mitigation technique
- [ ] EASY: Don't let users create packages named 'test' or names that start with a digit (cargo init fails): 
  - ```error: the name `test` cannot be used as a package name, it conflicts with Rust's built-in test library```,
  - ```error: the name `123` cannot be used as a package name, the name cannot start with a digit```
- [ ] EASY: split frontend cra logic (like the useAuth hook) into a separate package (create-rust-app on npm) -- this may not be favourable if we decide that we're not just focusing on react
- [ ] Social logins (google, github, oauth 2 for mobile authentication workflows)

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
