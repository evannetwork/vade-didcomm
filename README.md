# vade-didcomm

## About

This crate is a [DIDComm V2](https://identity.foundation/didcomm-messaging/spec/) vade plugin that currently offers:

- message encryption & decryption using [DIDComm rs](https://github.com/decentralized-identity/didcomm-rs)
- protocol support using DIDComm message `type`

It implements the following [`VadePlugin`] functions:

- [`didcomm_send`]
- [`didcomm_receive`]

Currently supported protocols:

- [`did_exchange`]
- [`trust_ping`]

## Usage

`didcomm_send` prepares a message for being sent to the recipient and `didcomm_receive` is used for decryption and analyzing an incoming message. Per default each sent message will be encrypted, either with the saved encryption key from an existing DID exchange with the communication partner, or with the provided one. Specific protocol types can override the encryption setting of a message to just send a plain message (like DID exchange).

*NOTE*: When you send any message that will be encrypted, you need to have a finished DID exchange or correct encryption keys, that are passed to vade_didcomm.

The two functions [`didcomm_send`] and [`didcomm_receive`] can be called with two parameters, `options` and `message`:

1. Options: Contains specific information for passing special configuration to the vade_didcomm. Currently its just used to inject specific encryption configuration, to overwrite the default DIDComm DID exchange key encryption.

```json
{
  "sharedSecret": "..."
}
```

2. Message: The plain message object, containing at least the type, to DID and from DID.

The result of both functions will have the same structure and will always return a stringified json, with the following pattern:

```json
{
  "message": {},
  "metadata": {}
}
```

The data that is represented in `message` and `metadata` is protocol specific.

### trust_ping

This protocol implementation has only 2 steps and is used more like a testing protocol.
To send a `trust_ping` message, pass the following message to the `didcomm_send`:

```json
{
    "type": "https://didcomm.org/trust_ping/1.0/ping",
    "from": "did::xyz:34r3cu403hnth03r49g01",
    "to": [ "did::xyz:34r3cu403hnth03r49g03" ],
}
```

This will return an encrypted stringified message and will also add a body to the message, that contains:

```json
{
  "body": {
    "response_requested": true
  }
}
```

### did_exchange

The [`DID exchange protocol`] is a bit more complex and consist of 3 steps. The whole flow is implemented in the [`did-exchange test`]. Like with the trust_ping, you can pass a simple message, that will be enhanced with the actual data that will be sent. Here an example for the first request (ensure, to set the service_endpoint to the url, where your DID agent is available):

```json
{
  "type": "https://didcomm.org/didexchange/1.0/request",
  "service_endpoint": "https://evan.network",
  "from": "did:uknow:d34db33d",
  "to": ["did:uknow:d34db33f"]
}
```

This will return the following result:

```json
{
  "message": {
    "body": {
      "@context": "https://w3id.org/did/v1",
      "authentication": [
        "{0}#key-1"
      ],
      "id": "did:uknow:d34db33d",
      "publicKey": [
        {
          "id": "did:uknow:d34db33d#key-1",
          "publicKeyBase58": "b1f88eebc9576fcb923837d9455ffd24a2c634d95e4e7c9fdf0ab362fd092a7c",
          "type": [
            "Ed25519VerificationKey2018"
          ]
        }
      ],
      "service": [
        {
          "id": "did:uknow:d34db33d#didcomm",
          "priority": 0,
          "recipientKeys": [
            "b1f88eebc9576fcb923837d9455ffd24a2c634d95e4e7c9fdf0ab362fd092a7c"
          ],
          "serviceEndpoint": "",
          "type": "did-communication"
        }
      ]
    },
    "from": "did:uknow:d34db33d",
    "id": "de9358ae810341e6b02bc08f7fd061ec",
    "pthid": "de9358ae810341e6b02bc08f7fd061ec#key-1",
    "thid": "did:uknow:d34db33d#key-1",
    "to": [
      "did:uknow:d34db33f"
    ],
    "type": "https://didcomm.org/didexchange/1.0/request"
  },
  "metadata": {
    "pub_key": "b1f88eebc9576fcb923837d9455ffd24a2c634d95e4e7c9fdf0ab362fd092a7c",
    "secret_key": "487db1e4be6f0ec0cb4fa07a64a5aea9bd5e77ba8f639e8595563535c5784166",
    "target_pub_key": "",
    "target_service_endpoint": ""
  }
}
```

As you can see, the whole message was enriched with the data that is necessary for the DID exchange. The metadata contains the generated communication hex encoded public key and secret key. The receiver can just pass the whole json to the `didcomm_receive` function, that will analyse the message, will save the communication keys and generate new ones for himself as well. The receiver can then use the logic for sending the response, by just replacing the type of the message `https://didcomm.org/didexchange/1.0/response.`

## Registering a new protocol

Each protocol is represented by a set of steps. To register a new protocol, just follow the following steps:

1. add new file into `src/protocols` with the following sample content:

```rs
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomBody {
    custom: Option<bool>,
}

pub fn generate_my_custom_protocol() -> Protocol {
    let mut protocol = Protocol {
        name: String::from("my_custom_protocol"),
        steps: Vec::new(),
    };

    protocol.steps.push(send_step("step1", send_step1));
    protocol.steps.push(receive_step("step1", receive_step1));

    return protocol;
}

pub fn send_step1(message: &str) -> StepResult {
    let mut parsed_message: MessageWithBody<CustomBody> = serde_json::from_str(message)?;
    parsed_message.body = Some(CustomBody {
        response_requested: Some(true),
    });
    return generate_step_output(&serde_json::to_string(&parsed_message)?, "{}");
}

pub fn receive_step1(message: &str) -> StepResult {
    return generate_step_output(message, "{}");
}
```

2. Import it into the protocols `mod.rs` file:

```rs
pub(crate) mod my_custom_protocol;
```

3. Register it within the `protocol_handler.rs`:

```rs
let protocols: [&Protocol; 3] = [
  &generate_did_exchange_protocol(),
  &generate_ping_pong_protocol(),
  &my_custom_protocol(),
];
```

Afterwards, you can just test your protocol by passing the following message to the DIDComm functions:

```json
{
    "type": "my_custom_protocol/step1",
    "from": "did::xyz:34r3cu403hnth03r49g01",
    "to": [ "did::xyz:34r3cu403hnth03r49g03" ],
}
```

[`didcomm_send`]: https://git.slock.it/equs/interop/vade/vade-didcomm/-/blob/feature/SL-6-key-exchange/src/vade_didcomm.rs#L37
[`didcomm_receive`]: https://git.slock.it/equs/interop/vade/vade-didcomm/-/blob/feature/SL-6-key-exchange/src/vade_didcomm.rs#L104
[`did_exchange`]: https://git.slock.it/equs/interop/vade/vade-didcomm/-/blob/feature/SL-6-key-exchange/src/protocols/did_exchange/did_exchange.rs
[`trust_ping`]: https://git.slock.it/equs/interop/vade/vade-didcomm/-/blob/feature/SL-6-key-exchange/src/protocols/pingpong.rs
[`DID exchange protocol`]: https://github.com/hyperledger/aries-rfcs/tree/master/features/0023-did-exchange
[`did-exchange test`]: https://git.slock.it/equs/interop/vade/vade-didcomm/-/blob/feature/SL-6-key-exchange/tests/did-exchange.rs
