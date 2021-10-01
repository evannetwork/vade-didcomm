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

### present_proof protocol

The [`Present Proof Protocol`] consists of 4 steps. The whole flow is implemented in the [`present-proof test`]. The general flow starts with a verifier sending a `request-presentation` message to a prover. The prover has the option to answer with the requested presentation or propose a new presentation to the verifier. The format for `request-presentation` is the following:

```json
{
    "@type": "https://didcomm.org/present-proof/1.0/request-presentation",
    "@id": "<uuid-request>",
    "comment": "some comment",
    "request_presentations~attach": [
        {
            "@id": "libindy-request-presentation-0",
            "mime-type": "application/json",
            "data":  {
                "base64": "<bytes for base64>"
            }
        }
    ]
}
```

The prover then responds with either a `presentation` or `propose-presentation` message. The following are the formats for the messages:

Presentation response format:

```json
{
    "@type": "https://didcomm.org/present-proof/1.0/presentation",
    "@id": "<uuid-presentation>",
    "comment": "some comment",
    "presentations~attach": [
        {
            "@id": "libindy-presentation-0",
            "mime-type": "application/json",
            "data": {
                "base64": "<bytes for base64>"
            }
        }
    ]
}
```

Presentation proposal format:


```json
{
    "@type": "https://didcomm.org/present-proof/1.0/presentation-preview",
    "attributes": [
        {
            "name": "<attribute_name>",
            "cred_def_id": "<cred_def_id>",
            "mime-type": "<type>",
            "value": "<value>",
            "referent": "<referent>"
        },
        // more attributes
    ],
    "predicates": [
        {
            "name": "<attribute_name>",
            "cred_def_id": "<cred_def_id>",
            "predicate": "<predicate>",
            "threshold": <threshold>
        },
        // more predicates
    ]
}
```
Once the presentation exchange is complete, the verifier sends an ack message to the prover to confirm the receival and validity of the received Presentation data. 

### issue_credential protocol

The [`Issue Credential Protocol`] consists of 5 steps. The whole flow is implemented in the [`issue-credential test`]. The general flow starts with a holder sending a `propose-credential` message to a issuer. The issuer has the option to answer with the `offer-credential` or terminate request with `problem-report` message. Holder receives `offer-credential` and decides to send `request-credential` message , Once issuer receives `request-credential`, he/she would respond with `issue-credential` and Holder will receive and send `ack` message to acknowledge the receipt of credential. 

Propose Credential message:

```json
{
    "@type": "https://didcomm.org/issue-credential/1.1/propose-credential",
    "@id": "<uuid-of-propose-message>",
    "comment": "some comment",
    "credential_proposal": <json-ld object>,
    "schema_issuer_did": "DID of the proposed schema issuer",
    "schema_id": "Schema ID string",
    "schema_name": "Schema name string",
    "schema_version": "Schema version string",
    "cred_def_id": "Credential Definition ID string"
    "issuer_did": "DID of the proposed issuer"
}
```

Offer Credential message :

```json
{
    "@type": "https://didcomm.org/issue-credential/1.0/offer-credential",
    "@id": "<uuid-of-offer-message>",
    "comment": "some comment",
    "credential_preview": <json-ld object>,
    "offers~attach": [
        {
            "@id": "libindy-cred-offer-0",
            "mime-type": "application/json",
            "data": {
                "base64": "<bytes for base64>"
            }
        }
    ]
}
```

Request Credential message:

```json
{
    "@type": "https://didcomm.org/issue-credential/1.0/request-credential",
    "@id": "<uuid-of-request-message>",
    "comment": "some comment",
    "requests~attach": [
        {
            "@id": "attachment id",
            "mime-type": "application/json",
            "data": {
                "base64": "<bytes for base64>"
            }
        },
    ]
}
```

Issue Credential message:

```json
{
    "@type": "https://didcomm.org/issue-credential/1.0/issue-credential",
    "@id": "<uuid-of-issue-message>",
    "comment": "some comment",
    "credentials~attach": [
        {
            "@id": "libindy-cred-0",
            "mime-type": "application/json",
            "data": {
                "base64": "<bytes for base64>"
            }
        }
    ]
}
```
### presentation exchange protocol

The [`Presentation Exchange Protocol`] consists of 3 steps. The whole flow is implemented in the [`presentation-exchange test`]. The general flow starts with a verifier sending a `request-presentation` message to a holder. The holder has an option to answer with the `propose-presentation` or send `presentation` message. Once Verifier receives `presentation` message, he/she will match the received credential claims against `presentation-definition` request and validate the claims values with the contraints present in the `input-descriptors` array in `presentation-definition` 

In the current implementation of presentation exchange protocol, the json schema constraints have to be verified by the client application which is using vade because the constraints are quite diverse and specific to application requirements, for details regarding constraints, please visit [`Presentation Exchange Protocol`].

request-presentation message:

```json
{
    "options": {
        "challenge": "...",
        "domain": "...",
    },
    "presentation_definition": {
        // presentation definition object
    }
}
```

presentation-definition example:

```json
{
  "comment": "Note: VP, OIDC, DIDComm, or CHAPI outer wrapper would be here.",
  "presentation_definition": {
    "id": "32f54163-7166-48f1-93d8-ff217bdb0653",
    "input_descriptors": [
      {
        "id": "wa_driver_license",
        "name": "Washington State Business License",
        "purpose": "We can only allow licensed Washington State business representatives into the WA Business Conference",
        "schema": [{
            "uri": "https://licenses.example.com/business-license.json"
        }]
      }
    ]
  }
}
```

presentation message example:

```json
{
    "@type": "https://didcomm.org/present-proof/%VER/presentation",
    "@id": "f1ca8245-ab2d-4d9c-8d7d-94bf310314ef",
    "comment": "some comment",
    "formats" : [{
        "attach_id" : "2a3f1c4c-623c-44e6-b159-179048c51260",
        "format" : "dif/presentation-exchange/submission@v1.0"
    }],
    "presentations~attach": [{
        "@id": "2a3f1c4c-623c-44e6-b159-179048c51260",
        "mime-type": "application/ld+json",
        "data": {
            "json": {
                "@context": [
                    "https://www.w3.org/2018/credentials/v1",
                    "https://identity.foundation/presentation-exchange/submission/v1"
                ],
                "type": [
                    "VerifiablePresentation",
                    "PresentationSubmission"
                ],
                "presentation_submission": {
                    "descriptor_map": [{
                        "id": "citizenship_input",
                        "path": "$.verifiableCredential.[0]"
                    }]
                },
                "verifiableCredential": [{
                    "@context": "https://www.w3.org/2018/credentials/v1",
                    "id": "https://eu.com/claims/DriversLicense",
                    "type": ["EUDriversLicense"],
                    "issuer": "did:foo:123",
                    "issuanceDate": "2010-01-01T19:73:24Z",
                    "credentialSubject": {
                        "id": "did:example:ebfeb1f712ebc6f1c276e12ec21",
                        "license": {
                            "number": "34DGE352",
                            "dob": "07/13/80"
                        }
                    },
                    "proof": {
                        "type": "RsaSignature2018",
                        "created": "2017-06-18T21:19:10Z",
                        "proofPurpose": "assertionMethod",
                        "verificationMethod": "https://example.edu/issuers/keys/1",
                        "jws": "..."
                    }
                }],
                "proof": {
                    "type": "RsaSignature2018",
                    "created": "2018-09-14T21:19:10Z",
                    "proofPurpose": "authentication",
                    "verificationMethod": "did:example:ebfeb1f712ebc6f1c276e12ec21#keys-1",
                    "challenge": "1f44d55f-f161-4938-a659-f8026467f126",
                    "domain": "4jt78h47fh47",
                    "jws": "..."
                }
            }
        }
    }]
}
```

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
[`Present Proof Protocol`]: https://github.com/hyperledger/aries-rfcs/tree/master/features/0037-present-proof
[`present-proof test`]:https://git.slock.it/equs/interop/vade/vade-didcomm/-/blob/DID-46-implement-present-proof-protocol-in-vade/tests/present-proof.rs
[`Issue Credential Protocol`]: https://github.com/hyperledger/aries-rfcs/tree/main/features/0036-issue-credential#preview-credential
[`issue-credential test`]:https://git.slock.it/equs/interop/vade/vade-didcomm/-/blob/feature/DID-54-implement-issue-credential-protocol-in-vade/tests/issue-credential.rs
[`Presentation Exchange Protocol`]:https://identity.foundation/presentation-exchange/
[`presentation-exchange test`]:https://git.slock.it/equs/interop/vade/vade-didcomm/-/blob/feature/DID-517-implement-presentation-exchange-protocol-in-vade/tests/presentation-exchange.rs