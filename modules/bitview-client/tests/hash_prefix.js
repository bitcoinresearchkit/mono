import assert from "node:assert/strict";
import { BitviewClient, addressPayloadHashPrefix } from "../index.js";

const vectors = [
  [Uint8Array.of(0x4e, 0x73), "58101afa51a1ecfd"],
  [Uint8Array.from({ length: 20 }, (_, i) => i), "c3327ecb8ae1ff23"],
  [Uint8Array.from({ length: 32 }, (_, i) => i), "c0186990f026b180"],
  [Uint8Array.from({ length: 65 }, (_, i) => i), "0d4b77027ae7d700"],
];

for (const [payload, hash] of vectors) {
  assert.equal(addressPayloadHashPrefix(payload, 16), hash);
  assert.equal(BitviewClient.addressPayloadHashPrefix(payload, 8), hash.slice(0, 8));
}
