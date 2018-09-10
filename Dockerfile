FROM alpine

RUN apk add --update supervisor && rm  -rf /tmp/* /var/cache/apk/*
COPY supervisord/supervisord.conf /etc/
COPY supervisord/masquerade.conf /etc/supervisor/conf.d/masquerade.conf

COPY target/x86_64-unknown-linux-musl/release/masquerade /usr/bin/masquerade
COPY www /www

RUN touch /var/log/masquerade.log

EXPOSE 8088

ENTRYPOINT ["supervisord", "--nodaemon", "--configuration", "/etc/supervisord.conf"]