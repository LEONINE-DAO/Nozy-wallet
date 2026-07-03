# Address Endpoints

## `POST /api/address/generate`

Generate new Orchard receiving address.

Body may include `password` if wallet locked.

Response: unified address string (`u1…`).

CLI equivalent: `nozy receive`.

Tauri: `generate_address`.

## Validation

Recipients for send must be unified addresses with Orchard receiver. Transparent addresses rejected.

See [Address Management](../user-guide/address-management.md).
