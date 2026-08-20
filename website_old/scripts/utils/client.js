import { BitviewClient } from "../modules/bitview-client/index.js";

// const brk = new BitviewClient("https://bitview.space");
const brk = new BitviewClient("/");

console.log(`VERSION = ${brk.VERSION}`);

export { brk };
