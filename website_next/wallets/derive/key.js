import { decodeBase58Check } from "./base58.js";
import { readUint32 } from "./bytes.js";

const EXTENDED_PUBLIC_KEY_LENGTH = 78;
const PUBLIC_KEY_LENGTH = 33;
const CHAIN_CODE_LENGTH = 32;

const extendedPublicKeyVersions = /** @type {const} */ ([
  {
    version: 0x0488b21e,
    prefix: "xpub",
    network: "mainnet",
    script: "p2pkh",
    addrType: "p2pkh",
  },
  {
    version: 0x049d7cb2,
    prefix: "ypub",
    network: "mainnet",
    script: "p2sh_p2wpkh",
    addrType: "p2sh",
  },
  {
    version: 0x04b24746,
    prefix: "zpub",
    network: "mainnet",
    script: "v0_p2wpkh",
    addrType: "p2wpkh",
  },
  {
    version: 0x043587cf,
    prefix: "tpub",
    network: "testnet",
    script: "p2pkh",
    addrType: "p2pkh",
  },
  {
    version: 0x044a5262,
    prefix: "upub",
    network: "testnet",
    script: "p2sh_p2wpkh",
    addrType: "p2sh",
  },
  {
    version: 0x045f1cf6,
    prefix: "vpub",
    network: "testnet",
    script: "v0_p2wpkh",
    addrType: "p2wpkh",
  },
]);

/**
 * @typedef {typeof extendedPublicKeyVersions[number]} ExtendedPublicKeyVersion
 */

/**
 * @typedef {Object} ExtendedPublicKey
 * @property {string} text
 * @property {number} depth
 * @property {number} childNumber
 * @property {Uint8Array} parentFingerprint
 * @property {Uint8Array} chainCode
 * @property {Uint8Array} publicKey
 * @property {ExtendedPublicKeyVersion} version
 */

/**
 * @param {number} version
 * @returns {ExtendedPublicKeyVersion}
 */
function findExtendedPublicKeyVersion(version) {
  const metadata = extendedPublicKeyVersions.find((item) => {
    return item.version === version;
  });

  if (!metadata) {
    throw new Error(`Unsupported extended public key version: ${version}`);
  }

  return metadata;
}

/**
 * @param {Uint8Array} publicKey
 */
function validateCompressedPublicKey(publicKey) {
  if (
    publicKey.length !== PUBLIC_KEY_LENGTH ||
    (publicKey[0] !== 0x02 && publicKey[0] !== 0x03)
  ) {
    throw new Error("Expected a compressed public key");
  }
}

/**
 * @param {string} text
 * @returns {Promise<ExtendedPublicKey>}
 */
export async function parseExtendedPublicKey(text) {
  const value = text.trim();
  const bytes = await decodeBase58Check(value);

  if (bytes.length !== EXTENDED_PUBLIC_KEY_LENGTH) {
    throw new Error("Invalid extended public key length");
  }

  const version = findExtendedPublicKeyVersion(readUint32(bytes, 0));
  const parentFingerprint = bytes.slice(5, 9);
  const chainCode = bytes.slice(13, 13 + CHAIN_CODE_LENGTH);
  const publicKey = bytes.slice(45);

  validateCompressedPublicKey(publicKey);

  return {
    text: value,
    depth: bytes[4],
    childNumber: readUint32(bytes, 9),
    parentFingerprint,
    chainCode,
    publicKey,
    version,
  };
}
