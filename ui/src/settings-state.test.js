import { test } from 'node:test';
import assert from 'node:assert/strict';
import { hasChanges } from './settings-state.js';

const saved = { environment: [{ key: 'MODEL', value: 'fast' }, { key: 'TOKEN', value: null }], metaprompt: 'Sé breve.' };
test('textarea line endings do not fabricate edits, while whitespace remains meaningful', () => {
  const crlf = { ...saved, metaprompt: 'Primera\r\nSegunda\r\n' };
  const values = new Map(saved.environment.map(field => [field.key, field.value]));
  assert.equal(hasChanges(crlf, values, 'Primera\nSegunda\n'), false);
  assert.equal(hasChanges(crlf, values, 'Primera \nSegunda\n'), true);
  assert.equal(hasChanges(crlf, values, 'Primera\nSegunda'), true);
});
test('only unsaved edits reveal the save controls', () => {
  const values = new Map(saved.environment.map(field => [field.key, field.value]));
  assert.equal(hasChanges(null, values, ''), false);
  assert.equal(hasChanges(saved, values, saved.metaprompt), false);
  values.set('MODEL', 'deep');
  assert.equal(hasChanges(saved, values, saved.metaprompt), true);
  values.set('MODEL', 'fast');
  assert.equal(hasChanges(saved, values, saved.metaprompt), false);
  assert.equal(hasChanges(saved, values, 'Otra instrucción.'), true);
  values.set('NEW_TOKEN', '');
  assert.equal(hasChanges(saved, values, saved.metaprompt), true);
});
