FROM rust:latest

WORKDIR /app

COPY . .

RUN cargo install cargo-watch

RUN cargo build

CMD [ "cargo", "watch", "-x", "run" ]