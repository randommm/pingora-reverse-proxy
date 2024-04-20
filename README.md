# Pingora reverse proxy example

Create a reverse proxy that connects backend HTTP or HTTPs services to a single HTTPs frontend (similar to what is generally done with Nginx) serving distinct SSL certificates (using SNI and the host header) for each backend service.

In this example, if you connect to <https://127.0.0.1:4430>

* with an SNI and Host header of somedomain.com, you will be served an SSL certificate to somedomain.com and a proxied connection to <http://127.0.0.1:4000>.

* with an SNI and Host header of one.one.one.one, you will be served an SSL certificate to one.one.one.one and a proxied connection to <https://one.one.one.one>.

# Usage

Create some self-signed certificates:

```bash
mkdir -p keys &&
openssl req -x509 -sha256 -days 356 -nodes -newkey rsa:2048 -subj "/CN=somedomain.com/C=UK/L=London" -keyout keys/some_domain_key.pem -out keys/some_domain_cert.crt &&
openssl req -x509 -sha256 -days 356 -nodes -newkey rsa:2048 -subj "/CN=one.one.one.one/C=UK/L=London" -keyout keys/one_key.pem -out keys/one_cert.crt
```

Start the service.

```bash
HTTPS_PORT=4430 HTTP_PORT=8080 cargo run
```

Start some HTTP server on port 4000, e.g.:

```bash
cd $(mktemp -d) && touch somefile && python -m http.server 4000
```

Play:

* with HTTPs apps that support SNI, e.g:

```bash
curl --connect-to somedomain.com:443:127.0.0.1:4430 https://somedomain.com -vk
```

```bash
curl --connect-to one.one.one.one:443:127.0.0.1:4430 https://one.one.one.one -vk
```

* with an HTTP only app:

```bash
curl --connect-to someotherdomain.com:80:127.0.0.1:8080 http://someotherdomain.com -vk
```
