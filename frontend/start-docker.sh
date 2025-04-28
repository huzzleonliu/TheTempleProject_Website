#!/bin/sh
podman run -d --user 1000:1000 \
  -p 3000:3000 \
  -v .:/frontend \
  frontend \
  /bin/sh -c "./docker-start.sh"
