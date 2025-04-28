#!/bin/sh
podman run -d --user 1000:1000

-p 8080:8080

-v ./src:/frontend/src

rust-trunk-frontend \
  /bin/sh -c "./docker-start.sh"
