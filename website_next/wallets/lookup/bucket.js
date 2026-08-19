const MIN_PREFIX_NIBBLES = 4;
const MAX_PREFIX_NIBBLES = 16;

/**
 * @typedef {import("../../modules/bitview-client/index.js").AddrHashPrefixMatches} AddrHashPrefixMatches
 * @typedef {import("../derive/index.js").AddressType} AddressType
 * @typedef {import("../derive/index.js").GeneratedAddress} GeneratedAddress
 */

/**
 * @typedef {Object} AddressClient
 * @property {(addrType: AddressType, payload: Uint8Array, nibbles: number, options?: { cache?: boolean }) => Promise<AddrHashPrefixMatches>} getAddressPayloadHashPrefixMatches
 */

/**
 * @param {AddressClient} client
 * @param {GeneratedAddress} generated
 * @param {number} nibbles
 * @returns {Promise<AddrHashPrefixMatches>}
 */
async function fetchPrefixMatches(client, generated, nibbles) {
  return /** @type {AddrHashPrefixMatches} */ (
    await client.getAddressPayloadHashPrefixMatches(generated.addrType, generated.payload, nibbles, {
      cache: false,
    })
  );
}

/**
 * @param {AddressClient} client
 * @param {GeneratedAddress} generated
 * @returns {Promise<AddrHashPrefixMatches>}
 */
export async function findUsablePrefixBucket(client, generated) {
  for (
    let nibbles = MIN_PREFIX_NIBBLES;
    nibbles <= MAX_PREFIX_NIBBLES;
    nibbles += 1
  ) {
    const matches = await fetchPrefixMatches(client, generated, nibbles);

    if (matches.truncated) continue;

    return matches;
  }

  throw new Error("Address prefix bucket is too large");
}
