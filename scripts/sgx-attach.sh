#!/bin/bash

which docker >/dev/null || {
  echo "ERROR: Please install Docker first."
  exit 1
}

# Attach to the existing `ekiden` container
docker exec -i -t \
  ekiden bash
