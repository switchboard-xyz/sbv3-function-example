FROM gramineproject/gramine:v1.4 as builder
RUN curl -fsSLo /usr/share/keyrings/microsoft.asc https://packages.microsoft.com/keys/microsoft.asc
RUN echo "deb [arch=amd64 signed-by=/usr/share/keyrings/microsoft.asc] https://packages.microsoft.com/ubuntu/20.04/prod focal main" | \
    tee /etc/apt/sources.list.d/msprod.list
# install Azure DCAP library
RUN apt-get update && apt-get install -y az-dcap-client xxd build-essential gcc clang
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH=$PATH:/root/.cargo/bin/
RUN cargo install cargo-strip
WORKDIR /app
RUN  apt-get install -y wget
ENV DEBIAN_FRONTEND="noninteractive"
ENV DISTRO="ubuntu20.04-server"
ENV SGX_DRIVER_VERSION="2.11.54c9c4c"
ENV SGX_SDK_VERSION="2.19.100.3"
ENV SGX_SDK_VERSION_SHORT="2.19"
ENV UNAME="5.15.0-1035-azure"
COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry --mount=type=cache,target=/app/target \
    cargo build --release && \
    cargo strip && \
    mv /app/target/release/app /app
#
# FROM gramineproject/gramine:v1.4
# COPY --from=builder /app/app .
# COPY configs/app.manifest.template .
# RUN gramine-manifest app.manifest.template > app.manifest
# RUN gramine-sgx-gen-private-key
# RUN gramine-sgx-sign --manifest app.manifest --output app.manifest.sgx | tee /out.txt
# RUN cat /out.txt | tail -1 | sed -e "s/^[[:space:]]*//" | xxd -r -p | base64 | tee /measurement.txt
# ENTRYPOINT ["gramine-sgx", "app"]
#
