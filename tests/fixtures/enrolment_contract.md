# Device enrolment API: contract

Version 1. Owned by the server team. Clients answer to this document; where a
client spec and this contract disagree, this contract wins.

## Endpoints

| Method | Path | Signed |
|---|---|---|
| `POST` | `/api/devices/enrol` | Yes |
| `POST` | `/api/devices/finalize` | Yes |
| `GET` | `/api/devices/products` | Yes |

`enrol` takes `{ "public_key": <base64>, "name": <string> }` and returns
`{ "pending_id": <string>, "approve_url": <string>, "expires_at": <RFC 3339> }`.
An operator approves or denies the pending enrolment at `approve_url`.

`finalize` takes `{ "pending_id": <string> }`. `products` takes no body and
returns the products the enrolled device is entitled to.

## Signed requests

Every signed request carries three headers:

- `Device-Id`: the pending id before finalization, the device id after.
- `Device-Counter`: a decimal `u64`.
- `Device-Signature`: base64 Ed25519 over the method, path, counter, and body.

The counter is a `u64` the device increments before each signed request. The
server records the highest counter it has accepted per key and rejects any
request whose counter is not greater, with `401` and the body
`{ "error": "replay", "last_counter": <u64> }`. A client that loses its counter
may resume from `last_counter + 1`.

## Finalize outcomes

`finalize` answers exactly four ways:

| Status | Meaning | Body |
|---|---|---|
| `200` | Finalized | `{ "device_id": <string> }` |
| `409` | Still pending; the operator has not acted | none |
| `410` | Denied by the operator | none |
| `404` | Expired; the pending enrolment is gone | none |

This set is closed; a new outcome is a new contract version.

## Validation

The server does not validate device names beyond length 1 to 64. Names are
stored as given and shown to the operator verbatim.

Pending ids and device ids are opaque strings. Clients must not parse them.

## Not in this version

There is no user code. Approval happens only through `approve_url`; no field
or endpoint accepts a code typed by a user.
