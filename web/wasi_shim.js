// wasi_shim.js - the wasi_snapshot_preview1 imports camcraft-ui.wasm asks a
// browser for (#31). flecs' C is linked against the pinned wasi libc
// (web/build-app-wasm.sh), so the module imports a handful of WASI calls
// for its clock, its log output and its exit path. A browser has no WASI,
// so this module answers them: the clocks from the page's own clocks,
// stdout/stderr into the console, and everything a tab cannot do (files,
// blocking waits) with the WASI errno that says so. Nothing here grants
// the module anything a page script does not already have.
// Provenance: our own code, written from the public WASI preview1
// specification (the witx function signatures and errno table).

const ERRNO_SUCCESS = 0;
const ERRNO_BADF = 8;
const ERRNO_NOSYS = 52;
const ERRNO_SPIPE = 70;
const FILETYPE_CHARACTER_DEVICE = 2;
const CLOCK_REALTIME = 0;

let memory = null;
const pending = { 1: "", 2: "" };
const decoder = new TextDecoder("utf-8");

/** Hand the shim the module's linear memory. Call once, right after the
 *  module is instantiated and before any export that can reach flecs. */
export function bind_memory(wasm_memory) {
  memory = wasm_memory;
}

/** A view over the current memory (it is re-read every call because the
 *  buffer is replaced whenever the module grows its memory). */
function view() {
  return memory ? new DataView(memory.buffer) : null;
}

export function clock_time_get(id, _precision, result_ptr) {
  const v = view();
  if (!v) return ERRNO_NOSYS;
  const ms = id === CLOCK_REALTIME ? Date.now() : performance.now();
  v.setBigUint64(result_ptr, BigInt(Math.round(ms * 1e6)), true);
  return ERRNO_SUCCESS;
}

/** stdout (1) and stderr (2) go to the console, one line at a time. */
export function fd_write(fd, iovs_ptr, iovs_len, nwritten_ptr) {
  const v = view();
  if (!v) return ERRNO_NOSYS;
  if (fd !== 1 && fd !== 2) return ERRNO_BADF;
  let written = 0;
  for (let i = 0; i < iovs_len; i++) {
    const ptr = v.getUint32(iovs_ptr + i * 8, true);
    const len = v.getUint32(iovs_ptr + i * 8 + 4, true);
    // Copy out of the module's memory before decoding.
    pending[fd] += decoder.decode(new Uint8Array(memory.buffer, ptr, len).slice());
    written += len;
  }
  let nl = pending[fd].indexOf("\n");
  while (nl >= 0) {
    const line = pending[fd].slice(0, nl);
    if (fd === 2) console.warn(line);
    else console.log(line);
    pending[fd] = pending[fd].slice(nl + 1);
    nl = pending[fd].indexOf("\n");
  }
  v.setUint32(nwritten_ptr, written, true);
  return ERRNO_SUCCESS;
}

export function fd_read(_fd, _iovs_ptr, _iovs_len, nread_ptr) {
  const v = view();
  if (!v) return ERRNO_NOSYS;
  v.setUint32(nread_ptr, 0, true); // end of file: a tab has no stdin
  return ERRNO_SUCCESS;
}

export function fd_close(_fd) {
  return ERRNO_SUCCESS;
}

export function fd_seek(_fd, _offset, _whence, _result_ptr) {
  return ERRNO_SPIPE;
}

export function fd_fdstat_get(fd, stat_ptr) {
  const v = view();
  if (!v) return ERRNO_NOSYS;
  if (fd > 2) return ERRNO_BADF;
  for (let i = 0; i < 24; i += 4) v.setUint32(stat_ptr + i, 0, true);
  v.setUint8(stat_ptr, FILETYPE_CHARACTER_DEVICE);
  return ERRNO_SUCCESS;
}

export function fd_fdstat_set_flags(_fd, _flags) {
  return ERRNO_NOSYS;
}

export function fd_prestat_get(_fd, _result_ptr) {
  return ERRNO_BADF; // no preopened directories: there is no file system
}

export function fd_prestat_dir_name(_fd, _path_ptr, _path_len) {
  return ERRNO_BADF;
}

export function environ_sizes_get(count_ptr, size_ptr) {
  const v = view();
  if (!v) return ERRNO_NOSYS;
  v.setUint32(count_ptr, 0, true);
  v.setUint32(size_ptr, 0, true);
  return ERRNO_SUCCESS;
}

export function environ_get(_environ_ptr, _buf_ptr) {
  return ERRNO_SUCCESS;
}

/** A tab cannot block its only thread; the caller's sleep returns at once. */
export function poll_oneoff(_in_ptr, _out_ptr, _n, _nevents_ptr) {
  return ERRNO_NOSYS;
}

export function sched_yield() {
  return ERRNO_SUCCESS;
}

/** The C side asked to end the process (flecs' abort path). A tab has no
 *  process to end: stop the module with an error the page can show. */
export function proc_exit(code) {
  throw new Error(`CamCraft's engine stopped (exit code ${code}); reload the page.`);
}
