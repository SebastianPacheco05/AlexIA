'use strict';

// Requirements: https://developer.amazon.com/en-US/docs/alexa/custom-skills/host-a-custom-skill-as-a-web-service.html
// Native certificate validation avoids the unpatched node-forge advisory
// GHSA-86w9-cpqp-85rv in ask-sdk-express-adapter 2.14.0.
const crypto = require('node:crypto');
const https = require('node:https');
const tls = require('node:tls');
const fs = require('node:fs/promises');
const os = require('node:os');
const path = require('node:path');
const { execFile } = require('node:child_process');
const { promisify } = require('node:util');
const exec = promisify(execFile);

async function verifyCertificate(pem, openssl, roots = tls.rootCertificates) {
  const certificates = pem.match(/-----BEGIN CERTIFICATE-----[\s\S]*?-----END CERTIFICATE-----/g);
  if (!certificates || certificates.length > 8) throw new Error('certificate_chain');
  const leaf = new crypto.X509Certificate(certificates[0]);
  if (leaf.checkHost('echo-api.amazon.com', { subject: 'never', wildcards: false }) !== 'echo-api.amazon.com') throw new Error('certificate_identity');
  if (leaf.publicKey.asymmetricKeyType !== 'rsa' || leaf.publicKey.asymmetricKeyDetails.modulusLength < 2048) throw new Error('certificate_key');
  const directory = await fs.mkdtemp(path.join(os.tmpdir(), 'alexia-cert-'));
  const files = ['roots.pem', 'leaf.pem', 'chain.pem'].map(name => path.join(directory, name));
  try {
    await fs.writeFile(files[0], roots.join('\n'), { mode: 0o600 });
    await fs.writeFile(files[1], certificates[0], { mode: 0o600 });
    await fs.writeFile(files[2], certificates.slice(1).join('\n'), { mode: 0o600 });
    const args = ['verify', '-x509_strict', '-auth_level', '2', '-verify_depth', '5',
      '-no-CApath', '-no-CAstore', '-CAfile', files[0]];
    if (certificates.length > 1) args.push('-untrusted', files[2]);
    args.push(files[1]);
    await exec(openssl, args, { timeout: 2000, maxBuffer: 4096, windowsHide: true,
      env: { PATH: process.env.PATH, SystemRoot: process.env.SystemRoot, TEMP: os.tmpdir() } });
    return leaf;
  } finally {
    // Only remove the three files created above, never recursively delete a path.
    for (const file of files) await fs.unlink(file).catch(() => {});
    await fs.rmdir(directory).catch(() => {});
  }
}

function downloadCertificate(url) {
  return new Promise((resolve, reject) => {
    const request = https.get(url, { timeout: 2000 }, response => {
      if (response.statusCode !== 200) { response.destroy(); reject(new Error('certificate_fetch')); return; }
      const chunks = [];
      let length = 0;
      response.on('data', chunk => {
        length += chunk.length;
        if (length > 65536) { response.destroy(new Error('certificate_size')); return; }
        chunks.push(chunk);
      });
      response.on('error', reject);
      response.on('end', () => resolve(Buffer.concat(chunks).toString('utf8')));
    });
    // https.get never follows redirects. TLS validation remains enabled.
    request.on('timeout', () => request.destroy(new Error('certificate_timeout')));
    request.on('error', reject);
  });
}

function verifyBody(body, signature, leaf) {
  if (!crypto.verify('RSA-SHA256', Buffer.from(body, 'utf8'),
    { key: leaf.publicKey, padding: crypto.constants.RSA_PKCS1_PADDING }, Buffer.from(signature, 'base64'))) {
    throw new Error('signature');
  }
}

const nativeVerifier = openssl => ({ verify: async (body, headers) => {
  const pem = await downloadCertificate(new URL(headers.signaturecertchainurl));
  const leaf = await verifyCertificate(pem, openssl);
  verifyBody(body, headers['signature-256'], leaf);
}});

async function verifyRequest(input, verifier = nativeVerifier(input.opensslBinary || 'openssl')) {
  const { headers, body, skillId } = input;
  if (typeof skillId !== 'string' || !skillId.startsWith('amzn1.ask.skill.')) throw new Error('skill_config');
  if (typeof body !== 'string' || Buffer.byteLength(body) > 65536) throw new Error('body');
  const signature = headers?.['signature-256'];
  const certificate = headers?.signaturecertchainurl;
  if (typeof signature !== 'string' || !/^[A-Za-z0-9+/]+={0,2}$/.test(signature) || signature.length > 2048) throw new Error('signature');
  if (typeof certificate !== 'string' || certificate.length > 2048) throw new Error('certificate_url');
  let url;
  try { url = new URL(certificate); } catch { throw new Error('certificate_url'); }
  if (url.protocol !== 'https:' || url.hostname !== 's3.amazonaws.com' ||
      (url.port && url.port !== '443') || url.username || url.password ||
      !url.pathname.startsWith('/echo.api/') || url.search || url.hash || certificate.includes('\\')) {
    throw new Error('certificate_url');
  }
  const envelope = JSON.parse(body);
  const timestamp = envelope.request?.timestamp;
  const time = typeof timestamp === 'string' && /^\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d(?:\.\d{1,3})?Z$/.test(timestamp)
    ? Date.parse(timestamp) : NaN;
  if (!Number.isFinite(time) || Math.abs(Date.now() - time) > 150000) throw new Error('timestamp');
  if (envelope.context?.System?.application?.applicationId !== skillId ||
      (envelope.session && envelope.session.application?.applicationId !== skillId)) throw new Error('skill_id');
  // The original body string is passed unchanged; no reserialization before verification.
  await verifier.verify(body, { 'signature-256': signature, signaturecertchainurl: certificate });
}

if (require.main === module && process.argv[2] === '--check') {
  exec(process.argv[3] || 'openssl', ['version'], { timeout: 2000, maxBuffer: 4096, windowsHide: true })
    .then(() => process.exit(0), () => process.exit(1));
} else if (require.main === module) {
  const timer = setTimeout(() => process.exit(1), 4500);
  let input = '';
  process.stdin.setEncoding('utf8');
  process.stdin.on('data', chunk => {
    input += chunk;
    if (Buffer.byteLength(input) > 512 * 1024) process.exit(1);
  });
  process.stdin.on('end', async () => {
    try {
      await verifyRequest(JSON.parse(input));
      process.stdout.write('{"verified":true}');
      clearTimeout(timer);
    } catch {
      // Do not print request data, credentials or certificate errors.
      clearTimeout(timer);
      process.exitCode = 1;
    }
  });
}

module.exports = { verifyRequest, verifyCertificate, verifyBody };
