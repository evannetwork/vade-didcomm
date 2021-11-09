# vade-didcomm

## Next Version

### Features

- encrypt DID exchange message by default
- allow to pass private/public keys for encryption
- allow to pass options (and with it additional metadata) to protocol handlers
- Update logic to follow transition to DIDComm to follow specification to follow the DIDComm v2 standard
  - update format of keys passed to send/receive
  - make signing and encryption optional (can be both, one of both or none)
  - allow to import keys to db
  - add options to protocol arguments to allow passing data that is not included in message to protocol handlers

### Fixes

- fix Linux and WASM build

### Deprecations

## v0.0.2

### Fixes

- added git urls as dependencies

## Initial Version

- add initial project setup
