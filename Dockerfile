FROM rust:latest

WORKDIR /usr/src/app

COPY . .

RUN cargo install --path .

COPY ./test-config.toml /etc/config.toml

CMD [ "zookoo", "--config", "/etc/zookoo/config.toml" ]