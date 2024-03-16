# Pingora reverse proxy example

Usage:

```bash
mkdir -p keys &&
openssl req -x509 -sha256 -days 356 -nodes -newkey rsa:2048 -subj "/CN=somedomain.com/C=UK/L=London" -keyout keys/some_domain_key.pem -out keys/some_domain_cert.crt &&
openssl req -x509 -sha256 -days 356 -nodes -newkey rsa:2048 -subj "/CN=one.one.one.one/C=UK/L=London" -keyout keys/one_key.pem -out keys/one_cert.crt
```

```bash
cargo run
```

```bash
cd $(mktemp -d) && touch somefile && python -m http.server 4000
```

```bash
curl --connect-to somedomain.com:443:127.0.0.1:4430 https://somedomain.com -vk
```

```bash
curl --connect-to one.one.one.one:443:127.0.0.1:4430 https://one.one.one.one -vk
```
