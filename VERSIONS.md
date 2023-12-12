# vade-didcomm

## Next Version

### Features

- allow setting a `label` in the invitation and response message
- add `state_storage` feature to be able to disable rocksdb storage
- remove rocksdb dependency from vade-didcomm
- update didcomm-rs dependency
- update presentation proposal data format
- update proof proposal message format to match specification
  - attachment name changed from `proposals_attach` to `proposals~attach`
  - content updated from `PresentationPreview` to an array of `PresentationAttach` values
- add `create_pairwise_did` custom function to create `CommunicationDidDocument` for pairwise did

### Fixes

- remove warnings when building/testing with and/or without `state_storage` feature
- update dependency `didcomm-rs` to a fork without `resolve` feature
- update dependencies for critical vulnerabilities
- fix `pthid` of message to be None, if not supplied
- add `comment` field to `did-exchange` body

### Deprecations

## v0.3.0

### Features

- added support for "skipMessagePackaging" in didcomm_receive
- updated didcomm-rs version

### Fixes

- enabled multithread support for rocksdb
- added problem report and state management support for did_exchange protocol

## v0.2.0

### Features

- refactor Present Proof protocol handling - refactor data structure according to spec
- refactor Issue Credential protocol handling - refactor data structure according to spec
- add create_keys function to generate X25519 secret/public keys for encryption
- remove messageRaw property from the output of didcomm_receive
- store raw messages in rocks db for didcomm send/receive
- retrieve stored didcomm messages by custom function

### Fixes

- fix smaller things in Issue Credential flow
  - move ack message out of DIDComm nesting in body and use a single DIDComm message for this
  - allow issuer to enter the flow upon receiving a request
  - fix mime type in test
  - add search_db_keys function for local_storage and debug storage.

## v0.1.0

### Features

- encrypt DID exchange message by default
- allow to pass private/public keys for encryption
- allow to pass options (and with it additional metadata) to protocol handlers
- Update logic to follow transition to DIDComm to follow specification to follow the DIDComm v2 standard
  - update format of keys passed to send/receive
  - make signing and encryption optional (can be both, one of both or none)
  - allow to import keys to db
  - add options to protocol arguments to allow passing data that is not included in message to protocol handlers
- allow to pass private keys to DID exchange logic to import them if pre-generated externally,
  can be done with `didExchangeMySecret` in options
- allows to skip message packaging to work with unencrypted messages,
  can be done with `skipMessagePackaging` in options
- allows to skip protocol handling to only en- or decrypt DIDComm messages,
  cna be cone with `skipProtocolHandling` in options

### Fixes

- fix Linux and WASM build
- `body` is now mandatory in for DIDComm messages according to spec
- fixes DID exchange protocol handling:
  - fix `thid` preservation between DID exchange messages
  - fix `from` and `to` in DID exchange response
  - keys may be saved twice if exchanging a DID document with a DID that differs from documents `id`
  - DID exchange result metadata is now properly camel cased
  - fix missing `thid` in DID exchange complete message

## v0.0.2

### Fixes

- added git urls as dependencies

## Initial Version

- add initial project setup
