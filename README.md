# Switchboard attestation example

In this example we will create a verifiable, confidential binary through the
Switchboard attestation service.

The steps to create the verifiable binary have been done for you, your only job
is to write the code!

In this setup we have created you simply need to run `bash build.sh` to build
your verifiable image.

You will see a `measurement.txt` generated in your directory which acts as the
signed identifier for your sgx secured program.

On startup of this program, we will generate a secure key using entropy from
within Intel SGX.

Then, we will generate an SGX Quote (https://is.gd/TYLu5k) to associate this key
with this enclave's measurement so you can prove that this generated key was
generated securely and confidentially within the enclave.

After the initialization transactions are completed, you can then use this
signer for privilidged actions in your application!

See https://crates.io/crates/solana_switchboard_attestation_program_sdk for
asserting this chain inside your program.
