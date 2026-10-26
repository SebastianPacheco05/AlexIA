const { test } = require('node:test');
const assert = require('node:assert/strict');
const { verifyRequest } = require('./verify.cjs');

const skillId = 'amzn1.ask.skill.test';
function input(overrides = {}) {
  return {
    skillId,
    headers: {
      'signaturecertchainurl': 'https://s3.amazonaws.com/echo.api/test.pem',
      'signature-256': 'ZmFrZQ=='
    },
    body: JSON.stringify({
      context: { System: { application: { applicationId: skillId } } },
      session: { application: { applicationId: skillId } },
      request: { timestamp: new Date().toISOString(), type: 'LaunchRequest' }
    }),
    ...overrides
  };
}

test('missing signatures fail without a certificate download', async () => {
  await assert.rejects(verifyRequest(input({ headers: {} })), /signature/);
});

test('rejects certificate URLs outside the exact Amazon location', async () => {
  for (const url of [
    'http://s3.amazonaws.com/echo.api/x.pem',
    'https://localhost/echo.api/x.pem',
    'https://s3.amazonaws.com.evil.test/echo.api/x.pem',
    'https://s3.amazonaws.com:444/echo.api/x.pem',
    'https://user@s3.amazonaws.com/echo.api/x.pem',
    'https://s3.amazonaws.com/echo.api/../other.pem',
    'https://s3.amazonaws.com/echo.api/x.pem?redirect=1'
  ]) {
    const request = input();
    request.headers.signaturecertchainurl = url;
    await assert.rejects(verifyRequest(request), /certificate_url/);
  }
});

test('rejects expired, future and malformed timestamps', async () => {
  for (const timestamp of [new Date(Date.now() - 151000).toISOString(), new Date(Date.now() + 151000).toISOString(), 'invalid']) {
    const request = input();
    const body = JSON.parse(request.body);
    body.request.timestamp = timestamp;
    request.body = JSON.stringify(body);
    await assert.rejects(verifyRequest(request), /timestamp/);
  }
});

test('both application IDs must match the configured skill', async () => {
  for (const key of ['context', 'session']) {
    const request = input();
    const body = JSON.parse(request.body);
    if (key === 'context') body.context.System.application.applicationId = 'other';
    else body.session.application.applicationId = 'other';
    request.body = JSON.stringify(body);
    await assert.rejects(verifyRequest(request), /skill_id/);
  }
});

test('only a verified signature authorizes the original unmodified body', async () => {
  const request = input();
  // Dependency injection is test-only; the CLI always uses Amazon's verifier.
  let checked = false;
  await verifyRequest(request, { verify: async (body, headers) => {
    checked = true;
    assert.equal(body, request.body);
    assert.equal(headers['signature-256'], 'ZmFrZQ==');
  }});
  assert.equal(checked, true);
  await assert.rejects(verifyRequest(request, { verify: async () => { throw new Error('invalid signature'); } }));
});
