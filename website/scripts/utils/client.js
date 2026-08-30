import { BitviewClient } from "../modules/bitview-client/index.js";

// const bitview = new BitviewClient("https://bitview.space");
const bitview = new BitviewClient("/");

console.log(`VERSION = ${bitview.VERSION}`);

export { bitview };
