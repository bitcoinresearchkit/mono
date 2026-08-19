import { BitviewClient } from "../modules/bitview-client/index.js";
import { BRK_BASE_URL } from "./api.js";

export { BRK_BASE_URL } from "./api.js";
export const brk = new BitviewClient(BRK_BASE_URL);
