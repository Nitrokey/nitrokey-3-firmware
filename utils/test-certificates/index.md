# Nitrokey 3 Test Certificates

- [`root`](./root): Nitrokey 3 root CA 
- [`fido`](./fido): intermdiate CA and EE certificate for the fido-authenticator application
  - [`nk-fido-ca-cert.der`](./fido/nk-fido-ca-cert.der), [`nk-fido-ca-cert.pem`](./fido/nk-fido-ca-cert.pem), [`nk-fido-ca-key.pem`](./fido/nk-fido-ca-key.pem): FIDO intermediate CA 
  - [`nk-fido-ee-cert.der`](./fido/nk-fido-ee-cert.der), [`nk-fido-ee-cert.pem`](./fido/nk-fido-ee-cert.pem), [`nk-fido-ee-key.pem`](./fido/nk-fido-ee-key.pem): FIDO EE key and certificate 
  - [`nk-fido-ca-key.trussed`](./fido/nk-fido-ca-key.trussed), [`nk-fido-ee-key.trussed`](./fido/nk-fido-ee-key.trussed): keys in format required by Trussed, converted using 
- [`firmware-lpc55`](./firmware-lpc55): firmware signature keys for LPC55: one EE certificate signed by a root certificate and three standalone EE certificates
- [`firmware-nrf52`](./firmware-nrf52): firmware signature key for NRF52
