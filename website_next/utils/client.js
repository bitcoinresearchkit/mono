import { BrkClient } from "../modules/brk-client/index.js";
import { BRK_BASE_URL } from "./api.js";

export { BRK_BASE_URL } from "./api.js";
export const brk = new BrkClient(BRK_BASE_URL);
