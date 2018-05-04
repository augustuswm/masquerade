FROM rust

COPY target/release/masquerade /masquerade
COPY www /www

CMD ["./masquerade"]