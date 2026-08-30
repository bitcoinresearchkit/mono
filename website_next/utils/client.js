import { BitviewClient } from "../modules/bitview-client/index.js";
import { BITVIEW_BASE_URL } from "./api.js";

export { BITVIEW_BASE_URL } from "./api.js";
export const bitview = new BitviewClient(BITVIEW_BASE_URL);
