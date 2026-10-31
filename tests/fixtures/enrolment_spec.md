# Device enrolment client: design

Date: 2026-10-20
Status: Approved, ready for implementation planning

This client answers to the server contract in `enrolment_contract.md`. Where
this document and the contract disagree, the contract wins and this document
is wrong.

## Intent

A desktop agent today ships with a hand-provisioned device key and a device id
pasted into its config. We want the agent to enrol itself: generate a key,
ask the server for a pending enrolment, send the user to approve it in a
browser, and pick up its device id once an operator approves. After that it
calls `products` like any other enrolled device.

The work splits into a library crate, `enrol`, that owns the protocol and the
on-disk record, and the agent app, which owns the UI and the platform glue.

## Goals

- A fresh install reaches an enrolled state with one click and one browser
  approval, and no config editing.
- A restart at any point resumes where it left off: a pending enrolment is
  not abandoned because the app quit, and an enrolled device never enrols
  twice.
- The library has no UI and no platform dependency beyond the keychain and
  the filesystem, so it can be tested without either.

## The client

`Client` is the protocol surface. It holds a base URL and an HTTP client, and
signs every request with the device key and the next counter value.

```rust
pub struct Client { /* base url, http client, counter store */ }

impl Client {
    pub fn new(base: &str) -> Client;
    pub fn enrol(&self, key: &DeviceKey, name: &str) -> Result<Pending, EnrolError>;
    pub fn finalize(&self, key: &DeviceKey, pending_id: &str)
        -> Result<Enrolled, FinalizeError>;
    pub fn products(&self, key: &DeviceKey, device_id: &str)
        -> Result<Vec<Product>, ProductsError>;
}

pub struct Pending { pub pending_id: String, pub approve_url: String, pub expires_at: SystemTime }
pub struct Enrolled { pub device_id: String, pub enrolled_at: SystemTime }
```

All three calls are blocking; the agent calls them from a worker thread.
`name` is the machine's hostname by default and the user can change it in
settings before enrolling.

`finalize` maps the contract's outcomes onto an error so the caller can match
on them:

```rust
#[non_exhaustive]
pub enum FinalizeError {
    Pending,
    Denied,
    Expired,
    Rejected,
    Unreachable,
}
```

It is non-exhaustive so the server can add outcomes without breaking us.
`Rejected` covers a `401`, including a replay; `Unreachable` covers transport
failures and any status the contract does not list.

## The device record

The library persists one record per install, as TOML beside the app's other
state. It is the only thing that survives a restart.

```rust
pub struct Record {
    pub pending: Option<PendingRecord>,
    pub device: Option<DeviceRecord>,
}

pub struct PendingRecord { pub pending_id: String, pub expires_at: SystemTime }
pub struct DeviceRecord { pub device_id: String, pub enrolled_at: SystemTime }
```

Exactly one of pending or device is present, or neither; a file with both
prefers device. A missing file is the same as a record with neither.

The private key lives in the platform keychain, not in the record. A keychain
item that is not 32 bytes is `StoreError::Corrupt` and is treated as no key.
The app then generates a fresh key and starts a new enrolment.

```rust
pub enum StoreError {
    NotFound,
    Corrupt,
    Io(io::Error),
}
```

The record and the counter store sit behind one `Mutex` owned by the
`Enrolment` handle the app holds. Every write re-checks that the record still
names the id being updated, and the mutex is never held across a network or
keychain call.

## The enrolment machine

The app drives enrolment through a small state machine in the library, so
the UI only renders states and sends events.

```rust
pub enum State {
    Idle,
    Waiting { pending: Pending, next_poll: Instant },
    Enrolled(Enrolled),
    Failed(FinalizeError),
}

pub enum Event { Begin, Cancel, Poll }
```

`Begin` generates a key if there is none, calls `enrol`, writes the pending
record, and opens the approval page. `Poll` calls `finalize` once; on
`Pending` it schedules the next poll five seconds out, on success it writes
the device record and clears the pending one. `Cancel` returns to `Idle` and
leaves the pending record in place so a later `Begin` can decide what to do
with it.

The machine remembers `cancelled: Option<PendingId>`; `Begin` clears it, so a
new attempt is never hijacked by the old one. A `Poll` that returns for a
cancelled id is dropped.

## Opening the browser

The approval page is opened through a trait, so the library does not depend
on a platform:

```rust
pub trait BrowserOpener {
    fn open(&self, url: &str) -> io::Result<()>;
}
```

The app's implementation shells out to `open` with the approval URL. The
library takes a `&dyn BrowserOpener` in `Begin`'s handler, so a future
platform can supply its own.

## Decisions

The counter is persisted through a `CounterStore` trait
(`fn reserve(&mut self) -> io::Result<u64>`), with a file store in the app
and a memory store in the library's tests, rather than a typestate over the
request. Both stores are real; the trait has two implementors from the first
commit.

`reserve` increments and persists before returning, so a crash between
reserving and sending wastes a counter value rather than repeating one. On a
`401 replay`, the client reseeds the store from `last_counter + 1` and retries
once.

## Left to the builder

- The TOML field names and whether timestamps are RFC 3339 strings or
  seconds since the epoch.
- The keychain service name.
- The exact shape of the tick timer that wakes the machine at its next
  deadline.

## Open with the user

- Whether the settings screen shows the pending enrolment's expiry, or only
  "waiting for approval".
- Whether a denied enrolment should offer to start again immediately, or ask
  the user to contact an operator first.
