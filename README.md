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

## Non root deploy

Without root, you cannot bind to ports 80 and 443 on Linux, so you need to use custom ports and some work arounds on curl.

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

* Connect to HTTP first and then be redirected to HTTPS:

```
curl --connect-to somedomain.com:443:127.0.0.1:4430 --connect-to somedomain.com:80:127.0.0.1:8080 http://somedomain.com -vkL
```

```
curl --connect-to one.one.one.one:443:127.0.0.1:4430 --connect-to one.one.one.one:80:127.0.0.1:8080 http://one.one.one.one -vkL
```

* with an HTTP only app (does not redirect to HTTPS):

```bash
curl --connect-to someotherdomain.com:80:127.0.0.1:8082 http://someotherdomain.com -vk
```

## Privileged deploy

If you have root access, you can allow the binary to bind to ports 80 and 443.

First, build the binary:

```cargo build```

then allow the binary to bind to ports lower than 1024:

```
sudo setcap 'cap_net_bind_service=+ep' target/debug/pingora-reverse-proxy
```

Start the service.

```bash
cargo run
```

Start some HTTP server on port 4000, e.g.:

```bash
cd $(mktemp -d) && touch somefile && python -m http.server 4000
```

Play:

* with HTTPs apps that support SNI, e.g:

```bash
curl --resolve somedomain.com:443:127.0.0.1 https://somedomain.com -vk
```

```bash
curl --resolve one.one.one.one:443:127.0.0.1 https://one.one.one.one -vk
```

* Connect to HTTP first and then be redirected to HTTPS:

```
curl --resolve somedomain.com:443:127.0.0.1 --resolve somedomain.com:80:127.0.0.1 http://somedomain.com -vkL
```

```
curl --resolve one.one.one.one:443:127.0.0.1 --resolve one.one.one.one:80:127.0.0.1 http://one.one.one.one -vkL
```

* with an HTTP only app (does not redirect to HTTPS):

```bash
curl --connect-to someotherdomain.com:80:127.0.0.1:8082 http://someotherdomain.com -vk
```
