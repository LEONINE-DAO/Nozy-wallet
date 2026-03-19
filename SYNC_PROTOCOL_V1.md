# Nozy Mobile Sync Protocol v1

This document defines Nozy's first mobile sync protocol for architecture A:
seed on both devices, metadata-only synchronization.

## Goals

- Keep mnemonic and private keys device-local.
- Pair extension and mobile using a short-lived handshake.
- Sync only non-secret metadata (labels, preferences, pending tx state).
- Support pair, unpair, and session expiry semantics.

## Non-goals (v1)

- No seed transport.
- No cloud key escrow.
- No cross-device transaction signing relay in v1.

## Trust model

- User restores/creates the same wallet seed on both extension and mobile.
- Pairing binds device identities and a session record.
- Metadata sync is authenticated and encrypted.

## Handshake flow (QR-based)

1. Extension starts pairing and creates:
   - `sessionId`
   - `challenge`
   - `verifyCode`
   - expiry timestamp
2. Extension encodes payload as QR:
   - schema version
   - wallet address
   - session metadata above
3. Mobile scans QR and displays `verifyCode` for user confirmation.
4. Extension confirms session and marks device paired.

## Data classes

- Secret data (never synced):
  - mnemonic, spending keys, private keys
- Metadata (sync eligible):
  - account labels
  - address book aliases
  - pending transaction status
  - UI preferences

## Replay and expiry

- Every pairing session has explicit expiry.
- Expired sessions are invalidated and removed from active state.
- Pairing session id must match during confirm step.

## Versioning

- Payload includes `v: 1`.
- Future versions should be additive where possible.

## Extension API surface (v1 skeleton)

- `mobile_sync_get_state`
- `mobile_sync_init_pairing`
- `mobile_sync_confirm_pairing`
- `mobile_sync_unpair`

## Security checklist for production hardening

- Encrypted transport envelope for metadata exchange.
- Authenticated device identity records.
- Anti-replay tokens for sync message batches.
- Clear user approvals for pair and unlink.
- Optional PIN/biometric gates on mobile.
