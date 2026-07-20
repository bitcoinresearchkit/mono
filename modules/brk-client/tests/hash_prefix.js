import assert from "node:assert/strict";
import { BrkClient, addressPayloadHashPrefix } from "../index.js";

const vectors = [
  [Uint8Array.of(0x4e, 0x73), "58101afa51a1ecfd"],
  [Uint8Array.from({ length: 20 }, (_, i) => i), "c3327ecb8ae1ff23"],
  [Uint8Array.from({ length: 32 }, (_, i) => i), "c0186990f026b180"],
  [Uint8Array.from({ length: 65 }, (_, i) => i), "0d4b77027ae7d700"],
];

for (const [payload, hash] of vectors) {
  assert.equal(addressPayloadHashPrefix(payload, 16), hash);
  assert.equal(BrkClient.addressPayloadHashPrefix(payload, 8), hash.slice(0, 8));
}

assert.throws(() => addressPayloadHashPrefix(Uint8Array.of(), 16), /non-empty/);
assert.throws(() => addressPayloadHashPrefix(new Uint8Array(66), 16), /at most 65/);
assert.throws(() => addressPayloadHashPrefix(Uint8Array.of(1, 2), 0), /1 to 16/);

const client = new BrkClient("http://127.0.0.1:0");
client.getAddressHashPrefixMatches = (addrType, prefix) => ({ addrType, prefix, truncated: false, addresses: [] });

assert.deepEqual(
  client.getAddressPayloadHashPrefixMatches("p2pkh", Uint8Array.from({ length: 20 }, (_, i) => i), 8),
  { addrType: "p2pkh", prefix: "c3327ecb", truncated: false, addresses: [] },
);
for (const [addrType, length] of [["p2pk33", 33], ["p2pk65", 65]]) {
  const payload = Uint8Array.from({ length }, (_, i) => i);
  assert.deepEqual(
    client.getAddressPayloadHashPrefixMatches(addrType, payload, 8),
    {
      addrType,
      prefix: addressPayloadHashPrefix(payload, 8),
      truncated: false,
      addresses: [],
    },
  );
}
assert.throws(
  () => client.getAddressPayloadHashPrefixMatches("p2pkh", Uint8Array.of(1, 2), 8),
  /p2pkh address payload length 20 bytes/,
);
assert.throws(
  () => client.getAddressPayloadHashPrefixMatches("p2pk33", new Uint8Array(65), 8),
  /p2pk33 address payload length 33 bytes/,
);
assert.throws(
  () => client.getAddressPayloadHashPrefixMatches("p2pk65", new Uint8Array(33), 8),
  /p2pk65 address payload length 65 bytes/,
);
assert.throws(
  () => client.getAddressPayloadHashPrefixMatches("p2pk", new Uint8Array(33), 8),
  /Unsupported address type/,
);
assert.throws(
  () => client.getAddressPayloadHashPrefixMatches("opreturn", Uint8Array.of(1, 2), 8),
  /Unsupported address type/,
);
