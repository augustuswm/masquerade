FROM rust

COPY target/release/banner /banner

RUN mkdir /www
RUN mkdir /www/dist
COPY src/frontend/static/index.html /www/index.html
COPY src/frontend/static/dist/bundle.js /www/dist/bundle.js

CMD ["./banner"]