<!-- function start_cropper() {
  cropper_loop(cropper_worker_bundle.value.result);
}
async function cropper_loop(scanner_state: ScannerState) {
  if (
    cropper_worker_bundle.value === null ||
    cropper_worker_bundle.value.worker === null ||
    cropper_worker_bundle.value.result === null
  ) {
    console.log("no more cropper");
    cropper_running.value = false;
    return;
  }
  cropper_running.value = true;
  cropper_worker_bundle.value.debounced_start(
    WasmOp.Cropper,
    scanner_state,
    (scanner_state) => cropper_loop(scanner_state),
    0,
    false,
  );
} -->
