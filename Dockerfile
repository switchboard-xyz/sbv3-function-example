FROM gramineproject/gramine:v1.4 as builder
RUN curl -fsSLo /usr/share/keyrings/microsoft.asc https://packages.microsoft.com/keys/microsoft.asc
RUN echo "deb [arch=amd64 signed-by=/usr/share/keyrings/microsoft.asc] https://packages.microsoft.com/ubuntu/20.04/prod focal main" | \
    tee /etc/apt/sources.list.d/msprod.list
RUN apt-get update && apt-get install -y az-dcap-client xxd build-essential gcc clang wget
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH=$PATH:/root/.cargo/bin/
RUN cargo install cargo-strip
WORKDIR /app
COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry --mount=type=cache,target=/app/target \
    cargo build --release && \
    cargo strip && \
    mv /app/target/release/app /app

FROM gramineproject/gramine:v1.4
WORKDIR /app
COPY --from=builder /app/app .
COPY configs/app.manifest.template .
COPY configs/boot.sh /boot.sh
RUN gramine-manifest app.manifest.template > app.manifest
RUN gramine-sgx-gen-private-key
RUN gramine-sgx-sign --manifest app.manifest --output app.manifest.sgx | tail -2 | tee /measurement.txt
RUN mkdir -p /data/protected_files
ENTRYPOINT ["bash", "/boot.sh"]

