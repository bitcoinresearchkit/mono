// @ts-check
// The rest's first module: the typed tree of every series, which a first chart, reading by id, doesn't need.
import { BitviewClient } from "/modules/bitview-client/index.js";
import { CLIENT, whole } from "/studio/app.js";

whole(new BitviewClient(CLIENT));
