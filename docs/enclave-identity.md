# Enclave identity
In this module, an enclave persistence maintains an identity for itself.

## State
* An immutable identity for an enclave persistence. Lives as sealed data. Consists of:
  * an asymmetric key pair used to bootstrap secure communications
  * if we need monotonic counters, the IDs of those counters

## Interfaces
* ECALL `createId() -> sealedId`: Generates an identity for a new enclave persistence and exports it in a sealed form.
* ECALL `restoreId(sealedId) -> void`: Populates an enclave launch with an identity. The enclave launch caches the identity in ephemeral enclave memory so that we don't have to pass the sealed ID and unseal it for every entry.
* ECALL `createReport(targetInfo) -> report`: Create a report with a body as specified below.

## Public identity string
The public identity of an enclave persistence established this way is a string that canonically encodes the public parts of the identity.

Protocol buffers would not be a suitable serialization format because [the specification does not define a canonical form](https://gist.github.com/kchristidis/39c8b310fd9da43d515c4394c3cd9510).

## Report body
* Vanity/disambiguation prefix
* Identity version
* Digest of public identity string

This allows us to fit a potentially large public identity string in the report body. It may help allow changes to the format of the identity and the public identity string.

## Enclave identity proof
It's the **public identity string** and an **attestation verification report** (AVR) (includes quote; quote includes report).

A compute node creates this proof by calling `createReport`, getting a revocation list, getting a quote, and verifying that quote.

To validate:
* Verify the signature (chain) on the AVR.
* Check that the AVR is recent enough.
* Check that the AVR says that the quote is okay.
* (we don't care about the quote outside of the report; that's IAS's problem)
* Check that the report body was derived from the public identity string.

This tells you *only* that all this identity came from **some** enclave persistence running **some** enclave program on **some** platform that IAS trusts (recently trusted). It's only the *authentication*. Next, for *authorization*, you would have to apply some policy to the information (e.g., the MRENCLAVE and flags in the report).

These proofs are intended to be valid for a period of time, so that the system can use keys in the enclave identity to sign and verify messages without contacting IAS. Currently we have it so that AVRs expire after a while. This would be much better if IAS would include a timesta
