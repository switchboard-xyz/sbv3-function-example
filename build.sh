#!/bin/sh
export DOCKER_BUILDKIT=1
IMG_NAME=${function-base}
docker build . -f Dockerfile.builder --tag "$IMG_NAME"
id=$(docker run -it -d --rm "$IMG_NAME" bash)
rm -rf bin/
mkdir -p bin
docker cp "$id":/app/app bin/app
docker kill "$id"
IMG_NAME=${1:-function}
docker build . -f Dockerfile.function --tag "$IMG_NAME"
