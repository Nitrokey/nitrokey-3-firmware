# SSH Tests

Here is a test script for:
- running preconfigured OpenSSH server based on Debian,
- generating a FIDO2 device based OpenSSH keys for multiple algorithms and storage types,
- connection test using a FIDO2 device.

Tested storage types: resident and non-resident.
Tested algorithms: ed25519-sk, ecdsa-sk.

## Usage

```
# to set everything up
$ make build

# to run the actual SSH auth test
$ make test

# to reset - remove keys, remove built image
$ make clean
```

## Further Improvements
1. Self-report used versions of the software: OpenSSH server, Debian's Docker image ID, etc.
2. Make the cow-message personalized by key type
3. Make a final report summary
4. Rewrite in pytest
5. Hide redundant messages, when not needed (or direct to log, except for the confirmation requests)