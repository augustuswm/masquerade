FROM alpine

COPY target/x86_64-unknown-linux-musl/release/masquerade /masquerade
COPY www /www

EXPOSE 8088

CMD ["./masquerade"]