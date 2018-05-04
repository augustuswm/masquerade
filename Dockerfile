FROM rust

COPY target/release/banner /banner
COPY www /www

CMD ["./banner"]