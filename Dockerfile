FROM alpine

RUN apk add --update supervisor
RUN apk add --update curl
RUN rm  -rf /tmp/* /var/cache/apk/*
COPY supervisord/supervisord.conf /etc/
COPY supervisord/masquerade.conf /etc/supervisor/conf.d/

COPY target/x86_64-unknown-linux-musl/release/masquerade /usr/bin/masquerade
COPY www /www

RUN touch /var/log/masquerade.log

EXPOSE 443

HEALTHCHECK --interval=1m --timeout=5s \
  CMD /usr/bin/curl --silent -f -k https://localhost || exit 1

ENTRYPOINT ["supervisord", "--nodaemon", "--configuration", "/etc/supervisord.conf"]