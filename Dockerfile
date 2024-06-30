FROM ubuntu

RUN apt-get update

RUN apt-get upgrade -y

RUN apt-get install -y \
    curl \
    clang \
    gcc \
    g++ \
    zlib1g-dev \
    libmpc-dev \
    libmpfr-dev \
    libgmp-dev \
    git \
    cmake \
    pkg-config \
    libssl-dev \
    build-essential

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s - -y

ENV PATH=/root/.cargo/bin:${PATH}

WORKDIR /app

COPY Cargo.toml Cargo.toml

COPY Cargo.lock Cargo.lock

RUN mkdir src && echo "fn main() {}" > src/main.rs

RUN cargo build --release --locked

RUN rm -rf src

COPY src src

RUN cargo build --release --locked

RUN mkdir -p keys && \
    openssl req -x509 -sha256 -days 356 -nodes -newkey rsa:2048 -subj "/CN=somedomain.com/C=UK/L=London" -keyout keys/some_domain_key.pem -out keys/some_domain_cert.crt && \
    openssl req -x509 -sha256 -days 356 -nodes -newkey rsa:2048 -subj "/CN=one.one.one.one/C=UK/L=London" -keyout keys/one_key.pem -out keys/one_cert.crt

CMD cargo run --release --locked
