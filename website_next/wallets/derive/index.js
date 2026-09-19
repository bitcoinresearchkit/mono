import { encodePublicKeyAddressData } from "./address.js";
import { derivePublicKeys, parseXpub } from "./bip32.js";
import {
  generateAddressesFromDescriptor,
  getOutputDescriptorBranchIds,
  isOutputDescriptor,
  selectOutputDescriptor,
} from "./descriptor.js";

const DEFAULT_START_INDEX = 0;
const DEFAULT_ADDRESS_COUNT = 20;
const MAX_ADDRESS_COUNT = 100;

const addrTypeByScript = /** @type {const} */ ({
  p2pkh: "p2pkh",
  p2sh_p2wpkh: "p2sh",
  v0_p2wpkh: "p2wpkh",
  v1_p2tr: "p2tr",
  v0_p2wsh_sortedmulti: "p2wsh",
});

/**
 * @typedef {import("./address.js").AddressScript} AddressScript
 * @typedef {import("./address.js").BitcoinNetwork} BitcoinNetwork
 * @typedef {(typeof addrTypeByScript)[keyof typeof addrTypeByScript]} AddressType
 */

/**
 * @typedef {Object} GeneratedAddress
 * @property {number} index
 * @property {string} address
 * @property {Uint8Array} payload
 * @property {Uint8Array} [publicKey]
 * @property {AddressScript} script
 * @property {BitcoinNetwork} network
 * @property {AddressType} addrType
 */

/**
 * @typedef {Object} GenerateAddressesOptions
 * @property {number} [start]
 * @property {number} [count]
 * @property {AddressScript} [script]
 * @property {readonly number[]} [path]
 * @property {string} [branchId]
 */

/**
 * @param {number | undefined} value
 */
function readStart(value) {
  if (value === undefined) return DEFAULT_START_INDEX;
  if (!Number.isSafeInteger(value) || value < 0) {
    throw new Error("Expected a non-negative start index");
  }

  return value;
}

/**
 * @param {number | undefined} value
 */
function readCount(value) {
  const count = value ?? DEFAULT_ADDRESS_COUNT;

  if (!Number.isSafeInteger(count) || count < 1 || count > MAX_ADDRESS_COUNT) {
    throw new Error(`Expected an address count from 1 to ${MAX_ADDRESS_COUNT}`);
  }

  return count;
}

/**
 * @param {string} source
 * @param {GenerateAddressesOptions} [options]
 * @returns {Promise<GeneratedAddress[]>}
 */
export async function generateAddressesFromKey(source, options = {}) {
  const key = await parseXpub(source);
  const start = readStart(options.start);
  const count = readCount(options.count);
  const script = options.script ?? key.version.script;
  const addrType = addrTypeByScript[script];
  const children = await derivePublicKeys(key, start, count, options.path);
  const addresses = /** @type {GeneratedAddress[]} */ ([]);

  for (const child of children) {
    const addressData = await encodePublicKeyAddressData(
      child.publicKey,
      script,
      key.version.network,
    );

    addresses.push({
      index: child.index,
      address: addressData.address,
      payload: addressData.payload,
      publicKey: child.publicKey,
      script,
      network: key.version.network,
      addrType,
    });
  }

  return addresses;
}

/**
 * @param {string} source
 * @param {GenerateAddressesOptions} [options]
 * @returns {Promise<GeneratedAddress[]>}
 */
export async function generateAddressesFromWalletSource(source, options = {}) {
  const start = readStart(options.start);
  const count = readCount(options.count);

  if (isOutputDescriptor(source)) {
    return generateAddressesFromDescriptor(
      selectOutputDescriptor(source, options.branchId),
      { start, count },
    );
  }

  return generateAddressesFromKey(source, {
    ...options,
    start,
    count,
  });
}

export {
  getOutputDescriptorBranchIds,
  isOutputDescriptor,
};
