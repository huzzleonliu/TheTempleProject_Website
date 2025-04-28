#!/bin/sh
podman run -d --user 1000:1000

-p 3000:3000

-v .:/frontend

rust-trunk-frontend \
  /bin/sh -c "./docker-start.sh"
