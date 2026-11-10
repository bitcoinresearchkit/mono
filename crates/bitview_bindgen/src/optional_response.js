import assert from 'node:assert/strict';
let passed = 0;
async function check(name, run) { await run(); passed++; console.log(`PASS ${name}`); }
const client = new FixtureClient({baseUrl:'http://offline.invalid', browserCache:false});
let replies = [], calls = 0;
globalThis.fetch = async () => { calls++; const response = replies.shift(); if (!response) throw new Error('unexpected request'); return response; };
const reply = (status, body, headers = {}) => new Response(body, {status, headers:{'Content-Type':'application/json', 'Cache-Control':'no-store', ...headers}});
const get = async (res, options = {cache:false}) => { replies.push(res); return client.getOptional(options); };
const post = async (res) => { replies.push(res); return client.postOptional('body'); };
await check('204 bypasses JSON decoder', async () => {
  const res = reply(204, null); res.json = () => { throw new Error('204 parsed'); };
  assert.equal(await get(res), null);
});
await check('200 object uses existing key conversion and map preservation', async () => {
  assert.deepEqual(await get(reply(200, '{"tx_id":"x","rates":{"keep_key":1}}')), {txId:'x', rates:{keep_key:1}});
});
await check('empty 200 fails', async () => { await assert.rejects(get(reply(200, '')), SyntaxError); });
await check('malformed 200 fails', async () => { await assert.rejects(get(reply(200, '{')), SyntaxError); });
await check('JSON null 200 remains valid JSON', async () => { assert.equal(await get(reply(200, 'null')), null); });
for (const status of [400,404,429,503]) await check(`${status} error details preserved`, async () => {
  await assert.rejects(get(reply(status, '{"error":{"code":"invalid_parameter","message":"detail"}}')), e =>
    e instanceof BitviewError && e.status === status && e.code === 'invalid_parameter' && e.message === 'detail');
});
await check('ordinary JSON 200 unchanged', async () => {
  replies.push(reply(200, '{"tx_id":"normal","rates":{"keep_key":1}}'));
  assert.deepEqual(await client.getOrdinary({cache:false}), {txId:'normal', rates:{keep_key:1}});
});
await check('ordinary empty 200 fails', async () => {
  replies.push(reply(200, '')); await assert.rejects(client.getOrdinary({cache:false}), SyntaxError);
});
await check('ordinary undeclared 204 fails', async () => {
  replies.push(reply(204, null)); await assert.rejects(client.getOrdinary({cache:false}), SyntaxError);
});
await check('POST 204 bypasses JSON decoder', async () => {
  const res=reply(204,null); res.json=()=>{throw new Error('204 parsed');}; assert.equal(await post(res),null);
});
await check('POST 200 preserves conversion', async () => {
  assert.deepEqual(await post(reply(200, '{"tx_id":"post"}')), {txId:'post'});
});
await check('POST empty 200 fails', async () => { await assert.rejects(post(reply(200, '')), SyntaxError); });
await check('POST error details preserved', async () => {
  await assert.rejects(post(reply(503, '{"error":{"code":"state_updating","message":"wait"}}')),e=>e.status===503&&e.code==='state_updating'&&e.message==='wait');
});
await check('cache 200 -> no-store 204 -> 200 never retains absence', async () => {
  const seen=[];
  assert.equal((await get(reply(200,'{"tx_id":"old"}',{'Cache-Control':'public','ETag':'old'}),{onValue:v=>seen.push(v)})).txId,'old');
  assert.equal(await get(reply(204,null),{onValue:v=>seen.push(v)}),null);
  assert.equal(client._memCache.size,0);
  assert.equal((await get(reply(200,'{"tx_id":"new"}',{'Cache-Control':'public','ETag':'new'}))).txId,'new');
  assert.deepEqual(seen.map(v=>v?.txId ?? null),['old','old',null]);
});
await check('cacheable 204 can be reused on 304 without JSON parsing', async () => {
  const separate = new FixtureClient({baseUrl:'http://offline.invalid',browserCache:false});
  replies.push(reply(204,null,{'Cache-Control':'public','ETag':'absent'}));
  assert.equal(await separate.getOptional(),null);
  replies.push(new Response(null,{status:304,headers:{ETag:'absent'}}));
  assert.equal(await separate.getOptional(),null);
});
await check('no-store calls always dispatch and leave no cache', async () => {
  const separate = new FixtureClient({baseUrl:'http://offline.invalid',browserCache:false});
  const before=calls;
  for(let n=0;n<2;n++) { replies.push(reply(200,'{"tx_id":"x"}')); await separate.getOptional(); }
  assert.equal(calls-before,2); assert.equal(separate._memCache.size,0);
});
assert.equal(replies.length,0);
console.log(`FINISHED ${passed} JavaScript mock-HTTP runtime cases`);
