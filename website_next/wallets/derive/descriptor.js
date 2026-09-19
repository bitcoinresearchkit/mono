import { derivePublicKeys, parseXpub } from "./bip32.js";
import { encodeP2wshAddressData } from "./address.js";
import { parseOutputDescriptor } from "./descriptor-parser.js";
import { encodeSortedMultisigScript } from "./multisig.js";

export {
  getOutputDescriptorBranchIds,
  isOutputDescriptor,
  parseOutputDescriptor,
  selectOutputDescriptor,
} from "./descriptor-parser.js";

/**
 * @typedef {import("./index.js").GeneratedAddress} GeneratedAddress
 */

/**
 * @param {string} descriptorText
 * @param {Object} options
 * @param {number} options.start
 * @param {number} options.count
 * @returns {Promise<GeneratedAddress[]>}
 */
export async function generateAddressesFromDescriptor(descriptorText, options) {
  const descriptor = parseOutputDescriptor(descriptorText);
  const parsedKeys = await Promise.all(
    descriptor.keys.map((key) => parseXpub(key.xpub)),
  );
  const network = parsedKeys[0].version.network;
  const childSets = await Promise.all(
    parsedKeys.map((key, index) => {
      if (key.version.network !== network) {
        throw new Error("Descriptor xpub networks must match");
      }

      return derivePublicKeys(
        key,
        options.start,
        options.count,
        descriptor.keys[index].path,
      );
    }),
  );
  const addresses = /** @type {GeneratedAddress[]} */ ([]);

  for (let offset = 0; offset < options.count; offset += 1) {
    const publicKeys = childSets.map((children) => children[offset].publicKey);
    const witnessScript = encodeSortedMultisigScript(
      publicKeys,
      descriptor.threshold,
    );
    const addressData = await encodeP2wshAddressData(witnessScript, network);

    addresses.push({
      index: options.start + offset,
      address: addressData.address,
      payload: addressData.payload,
      script: descriptor.script,
      network,
      addrType: "p2wsh",
    });
  }

  return addresses;
}
