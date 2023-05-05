#!/bin/sh
set -e
export DOCKER_BUILDKIT=1
IMG_NAME=${1:-function}
docker build . -f Dockerfile --tag "$IMG_NAME"
id=$(docker run -it -d --rm --entrypoint bash "$IMG_NAME")
mkdir -p out/
docker cp "$id":/measurement.txt out/measurement.txt
docker kill "$id"
MEASUREMENT="$(cat out/measurement.txt)"
echo "Measurement: ${MEASUREMENT}"
docker images -f "reference=${IMG_NAME}" | grep -v "<none>"
