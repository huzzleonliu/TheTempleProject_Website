#!/bin/sh
podman run -d \
  -p 3000:3000 \
  -v .:/frontend \
  frontend
