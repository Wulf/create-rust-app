FROM rust:1.54

RUN apt-get update -y
RUN apt-get install -y nodejs npm
RUN npm i -g yarn
# You can remove the next RUN step if you're not using the Create Rust App 'auth' plugin
# For the argonautica crate, we need to install LLVM/Clang v3.9 or higher
RUN apt-get install -y clang llvm-dev libclang-dev

WORKDIR /app
COPY . .

RUN cargo build --release

EXPOSE 8080

CMD ["cargo", "run", "--release"]