import { test } from 'node:test';
import assert from 'node:assert/strict';
import { githubConnectionState } from './github-state.js';

test('disconnect is disabled with no connection or after successful revocation', () => {
  assert.equal(githubConnectionState(null).canDisconnect, false);
  assert.equal(githubConnectionState({ status: 'revoked' }).canDisconnect, false);
  for (const status of ['connected', 'connecting', 'degraded', 'expired']) {
    assert.equal(githubConnectionState({ status }).canDisconnect, true);
  }
  const pending = githubConnectionState({ status: 'revoked', last_error: 'disconnect_cleanup_pending' });
  assert.equal(pending.canDisconnect, true);
  assert.equal(pending.cleanupPending, true);
});

test('authorization without installation offers verification and a concrete installation instruction', () => {
  const state = githubConnectionState({ status: 'configured_unverified', github_user_id: 7, last_error: 'app_installation_required' });
  assert.equal(state.connected, false);
  assert.equal(state.canVerify, true);
  assert.match(state.description, /falta instalar/);
  assert.match(state.label, /Falta instalar/);
});
test('old failed authorization cannot invoke vault verification with no saved identity', () => {
  const state = githubConnectionState({ status: 'degraded', github_user_id: null });
  assert.equal(state.canVerify, false);
  assert.match(state.description, /Desconectar/);
  assert.equal(githubConnectionState(null).canVerify, false);
});
test('connected identity and revoked or expired credentials have different controls', () => {
  const connection = { status: 'connected', github_user_id: 7, login: 'fixture', repositories: [] };
  assert.equal(githubConnectionState(connection).canVerify, true);
  assert.equal(githubConnectionState(connection).connected, true);
  for (const status of ['revoked', 'expired', 'connecting']) {
    assert.equal(githubConnectionState({ ...connection, status }).canVerify, false);
  }
});

test('new frontend identifies an older backend instead of requesting another broken authorization', () => {
  const state = githubConnectionState({ status: 'degraded', github_user_id: null }, 1);
  assert.equal(state.requiresRestart, true);
  assert.equal(state.canVerify, false);
  assert.match(state.description, /Ctrl\+C/);
  assert.match(state.description, /start-alexa/);
  assert.equal(githubConnectionState(null, 1).requiresRestart, true);
  assert.equal(githubConnectionState(null, 2).requiresRestart, undefined);
});
