import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const html = await readFile(new URL('./index.html', import.meta.url), 'utf8');
const code = html.slice(html.indexOf('      function bindToggle('), html.indexOf('      for (const [id, series]'));
const bindToggle = new Function(`${code}; return bindToggle;`)();

// Include both event APIs so this catches the previous accumulating-listener bug.
class Button extends EventTarget {
  constructor(pressed) {
    super();
    this.pressed = String(pressed);
    this.state = { textContent: pressed ? 'active' : 'inactive' };
  }
  getAttribute() { return this.pressed; }
  setAttribute(name, value) { this.pressed = value; }
  querySelector() { return this.state; }
  click() {
    if (this.disabled) return;
    const event = new Event('click');
    this.dispatchEvent(event);
    this.onclick?.(event);
  }
}

for (const [name, count, initial] of [['STH', 3, true], ['All', 3, true], ['LTH', 3, true], ['Bounds', 6, false]]) {
  test(`${name}: rebinding still toggles the entire group once per click`, () => {
    const button = new Button(initial);
    const staleCalls = [];
    bindToggle(button, [{ applyOptions: options => staleCalls.push(options) }]);
    const calls = Array.from({ length: count }, () => []);
    bindToggle(button, calls.map(list => ({ applyOptions: options => list.push(options) })));
    assert.equal(button.disabled, false);
    button.click();
    assert.equal(button.pressed, String(!initial));
    assert.equal(button.state.textContent, initial ? 'inactive' : 'active');
    button.click();
    assert.equal(button.pressed, String(initial));
    assert.equal(button.state.textContent, initial ? 'active' : 'inactive');
    assert.deepEqual(staleCalls, []);
    for (const list of calls) assert.deepEqual(list, [{ visible: !initial }, { visible: initial }]);
  });
}

test('a group without series stays disabled', () => {
  const button = new Button(false);
  bindToggle(button, []);
  assert.equal(button.disabled, true);
  button.click();
  assert.equal(button.pressed, 'false');
});
