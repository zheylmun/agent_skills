# Device enrolment client Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let the desktop agent enrol itself against the device enrolment API: generate a key, request a pending enrolment, send the user to approve it, and pick up its device id, then call `products` as an enrolled device.

**Architecture:** A library crate, `enrol`, owns the protocol client, the on-disk record, the keychain access, and the enrolment state machine; the agent app owns the UI and passes in the platform glue. The client signs every request with the device key and a counter reserved from a `CounterStore`. The library has no UI and no platform dependency beyond the keychain and the filesystem.

**Tech Stack:** Rust 2021, `ureq` for blocking HTTP, `ed25519-dalek` for signing, `serde` and `toml` for the record, `keyring` for the keychain.

**Spec:** `enrolment_spec_reviewed.md`. Read it before starting, including its `## Design decisions` section.

---

### Task 1: Ids, the server URL, and the signed client

**Files:**
- Create: `enrol/src/ids.rs`
- Create: `enrol/src/counter.rs`
- Create: `enrol/src/client.rs`
- Test: `enrol/tests/client.rs`

- [ ] **Step 1: Write the id and URL types**

`ServerUrl::parse(&str) -> Result<ServerUrl, UrlError>` accepts only an absolute `https` URL. `PendingId(String)` and `DeviceId(String)` wrap the contract's opaque ids; neither is parsed, and each has `as_str` for the `Device-Id` header.

- [ ] **Step 2: Write the counter store**

`trait CounterStore { fn reserve(&mut self) -> io::Result<u64>; }` with `FileCounterStore` for the app and `MemoryCounterStore` for tests. `reserve` increments and persists before returning.

- [ ] **Step 3: Write `Client::new`, `enrol`, and `finalize`**

`Client::new(server: ServerUrl) -> Client`. `enrol(&self, key: &DeviceKey, name: &str) -> Result<Pending, EnrolError>` and `finalize(&self, key: &DeviceKey, pending_id: &PendingId) -> Result<Enrolled, FinalizeError>`, each signed with a reserved counter. `FinalizeError` is the exhaustive enum `Pending`, `Denied`, `Expired`, `Rejected`, `Unreachable`, mapped from `409`, `410`, `404`, `401`, and transport failures or unlisted statuses. On `401 replay`, reseed the store from `last_counter + 1` and retry once.

- [ ] **Step 4: Test against a mock server**

One test per finalize outcome, one for the replay reseed, one for a `ServerUrl` that is `http`.

- [ ] **Step 5: Commit**

```bash
git add enrol/src/ids.rs enrol/src/counter.rs enrol/src/client.rs enrol/tests/client.rs
git commit -m "enrol: ids, counter store, and the signed client"
```

---

### Task 2: The device record and the key store

**Files:**
- Create: `enrol/src/record.rs`
- Create: `enrol/src/keystore.rs`
- Create: `enrol/src/enrolment.rs`
- Test: `enrol/tests/record.rs`

- [ ] **Step 1: Write the record file**

`DeviceFile` holds `pending: Option<PendingRecord>` and `device: Option<DeviceRecord>`; `load` prefers `device` when both are present. A missing file loads as a `DeviceFile` with neither. `PendingRecord { pending_id: PendingId, expires_at: SystemTime }` and `DeviceRecord { device_id: DeviceId, enrolled_at: SystemTime }`. Timestamps are RFC 3339 strings in the TOML.

- [ ] **Step 2: Write the key store**

`load_key() -> Result<Option<DeviceKey>, StoreError>`: a missing keychain item is `Ok(None)`, and an item that is not 32 bytes is `StoreError::Corrupt`. `StoreError` is `Corrupt` or `Io(io::Error)`.

- [ ] **Step 3: Write the `Enrolment` handle**

`Enrolment` owns the record and the counter store behind one `Mutex`, and is the handle the app holds.

- [ ] **Step 4: Test load and the key store**

A missing file, a pending-only file, a device-only file; a missing key, a 31-byte key.

- [ ] **Step 5: Commit**

```bash
git add enrol/src/record.rs enrol/src/keystore.rs enrol/src/enrolment.rs enrol/tests/record.rs
git commit -m "enrol: device record, key store, and the enrolment handle"
```

---

### Task 3: Products

**Files:**
- Modify: `enrol/src/client.rs`
- Test: `enrol/tests/products.rs`

- [ ] **Step 1: Write `products`**

`Client::products(&self, key: &DeviceKey, device_id: String) -> Result<Vec<Product>, ProductsError>`, signed with a reserved counter and sending `device_id` as the `Device-Id` header.

- [ ] **Step 2: Test against a mock server**

An entitled device, an empty list, a `401`.

- [ ] **Step 3: Commit**

```bash
git add enrol/src/client.rs enrol/tests/products.rs
git commit -m "enrol: products"
```

---

### Task 4: The enrolment machine

**Files:**
- Create: `enrol/src/machine.rs`
- Test: `enrol/tests/machine.rs`

- [ ] **Step 1: Write the states and events**

`State` is `Idle`, `Waiting { attempt: AttemptId, pending: Pending, next_poll: Instant }`, `Enrolled(Enrolled)`, or `Failed(FinalizeError)`. `Event` is `Begin`, `Cancel`, or `Poll`.

- [ ] **Step 2: Write the handlers**

`Begin` takes `open_browser: fn(&str) -> io::Result<()>`, generates a key if there is none, calls `enrol`, writes the pending record, mints a fresh `AttemptId`, and calls `open_browser` with the approval URL. `Poll` calls `finalize` once; on `Pending` it schedules the next poll five seconds out, and on success it writes the device record and clears the pending one. A poll result whose `AttemptId` is not the current attempt's is dropped. `Cancel` returns to `Idle` and leaves the pending record in place.

- [ ] **Step 3: Test the machine**

With `MemoryCounterStore`, a mock server, and an `open_browser` that records the URL: a full enrolment, a cancel then a new `Begin` whose old poll result is dropped, and a denial.

- [ ] **Step 4: Commit**

```bash
git add enrol/src/machine.rs enrol/tests/machine.rs
git commit -m "enrol: the enrolment machine"
```
