#!/usr/bin/env bash
#
# One-time setup: create a self-signed "macro_paste-dev" code-signing identity
# in the login keychain. This lets local dev builds be signed with a stable
# identity so the macOS Accessibility (TCC) permission survives rebuilds.
#
# Safe to re-run: it skips creation if the identity already exists.

set -euo pipefail

IDENTITY="macro_paste-dev"

if security find-identity -p codesigning | grep -q "$IDENTITY"; then
    echo "Identity '$IDENTITY' already exists – nothing to do."
    exit 0
fi

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

cat > "$TMP/cert.conf" <<'EOF'
[ req ]
distinguished_name = req_dn
x509_extensions = v3_codesign
prompt = no
[ req_dn ]
CN = macro_paste-dev
[ v3_codesign ]
basicConstraints = critical,CA:false
keyUsage = critical,digitalSignature
extendedKeyUsage = critical,codeSigning
EOF

echo "==> Generating key + self-signed code-signing certificate"
openssl req -x509 -newkey rsa:2048 \
    -keyout "$TMP/key.pem" -out "$TMP/cert.pem" \
    -days 3650 -nodes -config "$TMP/cert.conf" 2>/dev/null

echo "==> Packaging as PKCS#12 (legacy algorithms for macOS compatibility)"
openssl pkcs12 -export -legacy \
    -inkey "$TMP/key.pem" -in "$TMP/cert.pem" \
    -out "$TMP/id.p12" -passout pass:temp -name "$IDENTITY"

echo "==> Importing into login keychain (allowing codesign to use the key)"
security import "$TMP/id.p12" \
    -k ~/Library/Keychains/login.keychain-db \
    -P temp -T /usr/bin/codesign

echo "==> Done. Identity '$IDENTITY' created."
echo "    The certificate is self-signed and not trusted as a root; that is fine —"
echo "    codesign uses it for signing and TCC keys on the Designated Requirement."
