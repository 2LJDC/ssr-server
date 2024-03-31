FROM almalinux:minimal

WORKDIR /app

COPY target/release/ssr-server /app/ssr-server

COPY configuration.yaml /app/configuration.yaml

//COPY update-www.sh /app/update-www.sh

COPY www/ /app/www

COPY key.pem /app/key.pem

COPY cert.pem /app/cert.pem

RUN chmod 755 server

EXPOSE 8000

CMD ["/app/ssr-server"]
