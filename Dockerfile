FROM rust

COPY target/release/masquerade /masquerade
COPY www /www

EXPOSE 8088

CMD ["./masquerade"]