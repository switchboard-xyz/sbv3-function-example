killall -q aesm_service || true
export AZDCAP_DEBUG_LOG_LEVEL=WARNING
(
  AESM_PATH=/opt/intel/sgx-aesm-service/aesm LD_LIBRARY_PATH=/opt/intel/sgx-aesm-service/aesm exec /opt/intel/sgx-aesm-service/aesm/aesm_service --no-syslog
)
cd /app
echo "Starting enclave.."
gramine-sgx app

