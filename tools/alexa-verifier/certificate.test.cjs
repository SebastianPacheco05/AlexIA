const { test } = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs/promises');
const os = require('node:os');
const path = require('node:path');
const crypto = require('node:crypto');
const { execFile } = require('node:child_process');
const { promisify } = require('node:util');
const { verifyCertificate, verifyBody } = require('./verify.cjs');
const exec = promisify(execFile);

test('native trust validation rejects unknown roots and altered signatures', async () => {
  const openssl = process.env.ALEXA_OPENSSL_BIN || 'openssl';
  const directory = await fs.mkdtemp(path.join(os.tmpdir(), 'alexia-test-cert-'));
  const run = args => exec(openssl, args, { cwd: directory, windowsHide: true, timeout: 10000 });
  const files = ['root.key','root.pem','root.srl','leaf.key','leaf.csr','leaf.pem','extensions'];
  try {
    await run(['req','-x509','-newkey','rsa:2048','-nodes','-keyout','root.key','-out','root.pem','-days','1',
      '-subj','/CN=AlexIA test root','-addext','basicConstraints=critical,CA:TRUE',
      '-addext','keyUsage=critical,keyCertSign,cRLSign']);
    await run(['req','-newkey','rsa:2048','-nodes','-keyout','leaf.key','-out','leaf.csr','-subj','/CN=echo-api.amazon.com']);
    await fs.writeFile(path.join(directory,'extensions'),
      'basicConstraints=critical,CA:FALSE\nkeyUsage=critical,digitalSignature\nsubjectAltName=DNS:echo-api.amazon.com\nsubjectKeyIdentifier=hash\nauthorityKeyIdentifier=keyid,issuer\n');
    await run(['x509','-req','-in','leaf.csr','-CA','root.pem','-CAkey','root.key','-CAcreateserial',
      '-out','leaf.pem','-days','1','-extfile','extensions']);
    const [leaf, root, key] = await Promise.all(['leaf.pem','root.pem','leaf.key'].map(file => fs.readFile(path.join(directory,file),'utf8')));
    await assert.rejects(verifyCertificate(leaf + root, openssl));
    const certificate = await verifyCertificate(leaf, openssl, [root]);
    const body = '{"original":"body"}';
    const signature = crypto.sign('RSA-SHA256', Buffer.from(body), key).toString('base64');
    verifyBody(body, signature, certificate);
    assert.throws(() => verifyBody(body + ' ', signature, certificate), /signature/);
    assert.throws(() => verifyBody(body, 'ZmFrZQ==', certificate), /signature/);
    // A trusted certificate with the wrong SAN must still be rejected.
    await fs.writeFile(path.join(directory,'extensions'),
      'basicConstraints=critical,CA:FALSE\nkeyUsage=critical,digitalSignature\nsubjectAltName=DNS:wrong.example\nsubjectKeyIdentifier=hash\nauthorityKeyIdentifier=keyid,issuer\n');
    await run(['x509','-req','-in','leaf.csr','-CA','root.pem','-CAkey','root.key','-CAcreateserial',
      '-out','leaf.pem','-days','1','-extfile','extensions']);
    await assert.rejects(verifyCertificate(await fs.readFile(path.join(directory,'leaf.pem'),'utf8'), openssl, [root]), /certificate_identity/);
  } finally {
    for (const file of files) await fs.unlink(path.join(directory,file)).catch(() => {});
    await fs.rmdir(directory);
  }
});
