FROM rust:1.54

RUN apt-get update -y
RUN apt-get install -y nodejs npm
RUN npm i -g yarn
# You can remove the next RUN step if you're not using the Create Rust App 'auth' plugin
# For the argonautica crate, we need to install LLVM/Clang v3.9 or higher
RUN apt-get install -y clang llvm-dev libclang-dev

# Since the official rust image doesn't have nightly variants, we're going to install it ourselves
# This will make the image bigger than it needs to be -- you can try using another image if this is important to you
RUN rustup update nightly;
RUN rustup default nightly;

WORKDIR /app
COPY . .

RUN cargo build --release

EXPOSE 3000

CMD ["cargo", "run", "--release"]