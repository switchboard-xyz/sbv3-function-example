FROM gramineproject/gramine:v1.4
RUN curl -fsSLo /usr/share/keyrings/microsoft.asc https://packages.microsoft.com/keys/microsoft.asc
RUN echo "deb [arch=amd64 signed-by=/usr/share/keyrings/microsoft.asc] https://packages.microsoft.com/ubuntu/20.04/prod focal main" | \
    tee /etc/apt/sources.list.d/msprod.list

# install Azure DCAP library
RUN apt-get update && apt-get install -y az-dcap-client

# Copy the binary
WORKDIR /sgx
# COPY --from=builder /app /sgx/app
# COPY ./configs/app.manifest.template /sgx/app.manifest.template
#
# # Get the measurement from the enclave
# RUN gramine-manifest /sgx/app.manifest.template > /sgx/app.manifest
# RUN gramine-sgx-gen-private-key
# RUN gramine-sgx-sign --manifest /sgx/app.manifest --output /sgx/app.manifest.sgx | tee /out.txt
# CMD ["gramine-sgx", "/sgx/app"]
