#
# Step 1: use the rust image to compile the app
#
FROM rust:alpine as build

# update cargo index as a separate step to cache it
RUN cargo search create-rust-app

# install dependencies
RUN apk --no-cache add --update nodejs npm libc-dev openssl-dev libpq-dev

# build the app
WORKDIR /app
COPY . .
# note: this also builds the frontend (see `build.rs`)
RUN cargo build --release --bin app-binary-name --target x86_64-unknown-linux-musl

#
# Step 2: create an image just containing the compiled app and static assets
#
FROM alpine:latest
WORKDIR /app
COPY --from=build /app /appPtr
RUN ls -lha /appPtr
COPY --from=build /app/.cargo/.build/release/app-binary-name /app/app-binary-name
COPY --from=build /app/frontend/dist /app/frontend/dist

EXPOSE 3000

CMD ["app-binary-name"]