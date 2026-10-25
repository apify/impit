#!/bin/sh
# Regenerates the test PKI used by the certificate-verification cache tests and benchmark:
# an RSA root and intermediate and ECDSA P-256 leaves, like the chains large sites serve.
# The tests hardcode the leaves' notAfter, so update them after regenerating.
set -eu
cd "$(dirname "$0")"
ext=$(mktemp)
trap 'rm -f "$ext" ./*.csr ./*.srl' EXIT

openssl req -x509 -newkey rsa:2048 -nodes -keyout root.key -out root.pem -days 36500 -subj "/CN=impit test root" \
  -addext "basicConstraints=critical,CA:TRUE" -addext "keyUsage=critical,keyCertSign,cRLSign"

openssl req -newkey rsa:2048 -nodes -keyout intermediate.key -out intermediate.csr -subj "/CN=impit test intermediate"
printf 'basicConstraints=critical,CA:TRUE,pathlen:0\nkeyUsage=critical,keyCertSign,cRLSign\n' >"$ext"
openssl x509 -req -in intermediate.csr -CA root.pem -CAkey root.key -CAcreateserial -days 30000 -sha256 \
  -extfile "$ext" -out intermediate.pem

printf 'basicConstraints=critical,CA:FALSE\nkeyUsage=critical,digitalSignature\nextendedKeyUsage=serverAuth\nsubjectAltName=DNS:localhost,DNS:a.test\n' >"$ext"
for leaf in leaf-a:20000 leaf-b:20000 leaf-short:1000; do
  name=${leaf%%:*}; days=${leaf##*:}
  openssl req -newkey ec -pkeyopt ec_paramgen_curve:P-256 -nodes -keyout "$name.key" -out "$name.csr" -subj "/CN=$name"
  openssl x509 -req -in "$name.csr" -CA intermediate.pem -CAkey intermediate.key -CAcreateserial -days "$days" \
    -sha256 -extfile "$ext" -out "$name.pem"
done
for f in *.pem; do openssl x509 -in "$f" -outform DER -out "${f%.pem}.der"; done
# The benchmark's server uses leaf-a.
openssl pkcs8 -topk8 -nocrypt -in leaf-a.key -outform DER -out leaf-a.key.der
rm -f ./*.pem ./*.key
