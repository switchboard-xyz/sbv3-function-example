#!/bin/sh
set -e
export DOCKER_BUILDKIT=1
IMG_NAME=${1:-function}
if [[ "$(uname -s)" == "Linux" ]]; then
  docker build . -f Dockerfile --tag "${IMG_NAME}"
elif [[ "$(uname -s)" == "Darwin" ]]; then
  docker buildx create --name mybuilder > /dev/null 2>&1 || true;
  docker buildx use mybuilder
  docker buildx build --platform linux/amd64 --tag ${IMG_NAME} -f Dockerfile . --load
fi
id=$(docker run -it -d --rm --entrypoint bash "$IMG_NAME")
mkdir -p out/
docker cp "$id":/measurement.txt out/measurement.txt
docker kill "$id"
MEASUREMENT="$(cat out/measurement.txt)"
echo "${MEASUREMENT}"
docker images -f "reference=${IMG_NAME}" | grep -v "<none>"
