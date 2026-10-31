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

`Client` is the protocol surface. It holds a server URL and an HTTP client,
and signs every request with the device key and the next counter value.

```rust
pub struct ServerUrl { /* parsed once, by ServerUrl::parse */ }
pub struct PendingId(String);
pub struct DeviceId(String);

pub struct Client { /* server url, http client, counter store */ }

impl Client {
    pub fn new(server: ServerUrl) -> Client;
    pub fn enrol(&self, key: &DeviceKey, name: &str) -> Result<Pending, EnrolError>;
    pub fn finalize(&self, key: &DeviceKey, pending_id: &PendingId)
        -> Result<Enrolled, FinalizeError>;
    pub fn products(&self, key: &DeviceKey, device_id: &DeviceId)
        -> Result<Vec<Product>, ProductsError>;
}

pub struct Pending { pub pending_id: PendingId, pub approve_url: String, pub expires_at: SystemTime }
pub struct Enrolled { pub device_id: DeviceId, pub enrolled_at: SystemTime }
```

`ServerUrl::parse` takes the app's configured base URL and fails on anything
that is not an absolute `https` URL. `PendingId` and `DeviceId` wrap the
contract's opaque id strings; the client never parses either.

All three calls are blocking; the agent calls them from a worker thread.
`name` is the machine's hostname by default and the user can change it in
settings before enrolling.

`finalize` maps the contract's outcomes onto an error so the caller can match
on them:

```rust
pub enum FinalizeError {
    Pending,
    Denied,
    Expired,
    Rejected,
    Unreachable,
}
```

`Rejected` covers a `401`, including a replay; `Unreachable` covers transport
failures and any status the contract does not list.

## The device record

The library persists one record per install, as TOML beside the app's other
state. It is the only thing that survives a restart.

```rust
pub enum Record {
    Pending(PendingRecord),
    Device(DeviceRecord),
}

pub struct PendingRecord { pub pending_id: PendingId, pub expires_at: SystemTime }
pub struct DeviceRecord { pub device_id: DeviceId, pub enrolled_at: SystemTime }

pub fn load(path: &Path) -> Result<Option<Record>, LoadError>;

pub enum LoadError {
    BothRecorded,
    Io(io::Error),
    Malformed,
}
```

A missing file loads as `Ok(None)`. A file that names both a pending and a
device record is `LoadError::BothRecorded`.

The private key lives in the platform keychain, not in the record. Loading it
returns `Result<Option<DeviceKey>, StoreError>`: a missing item is `Ok(None)`,
and the app then generates a fresh key and starts a new enrolment. A keychain item
that is not 32 bytes is `StoreError::Corrupt`.

```rust
pub enum StoreError {
    Corrupt,
    Io(io::Error),
}
```

The record and the counter store sit behind one `Mutex` owned by the
`Enrolment` handle the app holds. The record changes only through
`Record::update_if(id, next)`, which takes the lock, replaces the record with
`next` only when it still names the pending id `id`, and reports whether it
did; `next` is computed before the call, so nothing else runs under the lock.

## The enrolment machine

The app drives enrolment through a small state machine in the library, so
the UI only renders states and sends events.

```rust
pub enum State {
    Idle,
    Waiting { attempt: AttemptId, pending: Pending, next_poll: Instant },
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

Each `Begin` starts a new attempt with a fresh `AttemptId`, which
`State::Waiting` carries and every poll result names. A poll result for any
other attempt is dropped.

## Opening the browser

`Begin`'s handler takes the opener as a plain function,
`open_browser: fn(&str) -> io::Result<()>`, and calls it with the approval
URL. The app passes one that shells out to `open`; the library's tests pass
one that records the URL.

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

## Design decisions

- **Adopted with a cost (round 1).** Ids are the newtypes `PendingId` and
  `DeviceId` in every signature and record that carries one, and
  `Client::new` takes a `ServerUrl` parsed once. Cost: every call site that
  forwards an id converts at the boundary, and the app parses its configured
  base URL before it can build a `Client`. Retires: passing a pending id where
  a device id belongs, or the reverse, and a malformed base URL surfacing only
  on the first request.
- **Adopted (round 1).** `Record` is an enum of `Pending(PendingRecord)` or
  `Device(DeviceRecord)`, loaded as `Option<Record>`, and a file that names
  both is `LoadError::BothRecorded`. Retires: the "exactly one of pending or
  device is present, or neither" rule and the tie-break that a file with both
  prefers device.
- **Adopted (round 1).** Loading the key returns
  `Result<Option<DeviceKey>, StoreError>`, in which a missing item is
  `Ok(None)` and an item that is not 32 bytes is `StoreError::Corrupt`, never
  no key. Retires:
  the "treated as no key" rule, under which a corrupt keychain item silently
  started a new enrolment.
- **Adopted (round 1).** The record changes only through
  `Record::update_if(id, next)`, which takes the lock and replaces the record
  only when it still names `id`. Retires: the rule that every write re-checks
  that the record still names the id being updated, and the rule that the
  mutex is never held across a network or keychain call.
- **Adopted (round 1).** Each `Begin` starts an attempt with a fresh
  `AttemptId` that `State::Waiting` carries and every poll result names.
  Retires: the remembered `cancelled: Option<PendingId>` and the rule that
  `Begin` clears it so a new attempt is never hijacked by the old one.
- **Adopted (round 1).** The approval page is opened through a plain
  `fn(&str) -> io::Result<()>` passed to `Begin`'s handler, not a trait.
  Retires: a `BrowserOpener` trait with one implementor that every caller had
  to implement and pass as `&dyn`.
- **Adopted (round 1).** `FinalizeError` is exhaustive, as the contract's
  closed set of finalize outcomes is. Retires: a wildcard arm in every
  caller's match, which would hide a new contract version's outcome.
- **Declined (round 2).** `Client` becomes a typestate over the enrolment
  stage, so `products` cannot be called before `finalize` has succeeded.
  Reason: the three calls are made from different owners; needs the user.

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
