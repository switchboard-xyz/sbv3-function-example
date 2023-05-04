FROM switchboardlabs/sgx-function AS builder

# Install cargo-strip and use cache if available
RUN cargo install cargo-strip

WORKDIR /app
COPY . .

# Build the release binary
RUN --mount=type=cache,target=/usr/local/cargo/registry --mount=type=cache,target=/app/target \
    cargo build --release && \
    cargo strip && \
    mv /app/target/release/app /app


FROM gramineproject/gramine:v1.4
RUN curl -fsSLo /usr/share/keyrings/microsoft.asc https://packages.microsoft.com/keys/microsoft.asc
RUN echo "deb [arch=amd64 signed-by=/usr/share/keyrings/microsoft.asc] https://packages.microsoft.com/ubuntu/20.04/prod focal main" | \
    tee /etc/apt/sources.list.d/msprod.list

# install Azure DCAP library
RUN apt-get update && apt-get install -y az-dcap-client

WORKDIR /sgx
COPY --from=builder /app/app .
COPY configs/app.manifest.template .
RUN gramine-manifest app.manifest.template > app.manifest
RUN gramine-sgx-gen-private-key
RUN gramine-sgx-sign --manifest app.manifest --output app.manifest.sgx | tee /out.txt
RUN apt-get install xxd
RUN cat /out.txt | tail -1 | sed -e "s/^[[:space:]]*//" | xxd -r -p | base64 | tee /measurement.txt
RUN ["./app"]

