#!/bin/bash

which docker >/dev/null || {
  echo "ERROR: Please install Docker first."
  exit 1
}

# Attach to the existing storage container
docker exec -i -t \
  storage bash
