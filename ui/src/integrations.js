import { githubErrors, githubConnectionState } from './github-state.js';
const $ = selector => document.querySelector(selector);
const messages = {
  ...githubErrors,
  queued: 'En cola', running: 'En curso', completed: 'Terminado', failed: 'Error', cancelled: 'Cancelado',
  waiting_for_input: 'Necesita selección', interrupted: 'Interrumpido tras reinicio', reconciling: 'Reconciliando efecto',
  fixture_completed: 'Cola local verificada con fixture', freshness_unknown: 'No hay una última subida confirmada. Elige una rama.',
  technical_isolation_unverified: 'Creación bloqueada: aislamiento técnico pendiente', destination_guard_unverified: 'Merge bloqueado: protección del destino pendiente',
  already_exists: 'La rama ya tiene una PR abierta', connection_required: 'Conecta y verifica GitHub primero.',
  repositories_required: 'Guarda primero un repositorio autorizado.', unauthenticated: 'Desbloquea las integraciones con el código local.',
  invalid_app_configuration: 'Comprueba el App ID y el Client ID públicos.',
};

export function setupIntegrations() {
  let csrf = '', connection = null, available = [], busy = false, jobsVersion = '';
  const message = (text, error = false) => { $('#github-message').textContent = messages[text] || text; $('#github-message').classList.toggle('field-error', error); };
  const alexaMessage = (text, error = false) => { $('#alexa-link-message').textContent = messages[text] || text; $('#alexa-link-message').classList.toggle('field-error', error); };
  async function api(path, method = 'GET', body) {
    const response = await fetch(path, { method, credentials: 'same-origin', headers: {
      'X-Alexia-UI': '1', 'X-Alexia-CSRF': csrf, ...(body !== undefined ? { 'Content-Type': 'application/json' } : {}),
    }, body: body !== undefined ? JSON.stringify(body) : undefined, signal: AbortSignal.timeout(120000) });
    const data = await response.json();
    if (!response.ok) throw new Error(messages[data.error] || data.error || 'No se pudo completar la operación.');
    return data;
  }
  const perform = async (action, notify = message) => {
    if (busy) return;
    busy = true; notify('Procesando…');
    try { await action(); } catch (error) { notify(error.message, true); }
    finally { busy = false; }
  };
  function unlocked() {
    $('#integration-unlock').classList.add('hidden'); $('#github-controls').classList.remove('hidden');
    $('#refresh-jobs').disabled = false; $('#fixture-job').disabled = false;
  }
  async function catalog() {
    const data = await api('/api/integrations');
    const wasConnecting = connection?.status === 'connecting';
    const integration = data.integrations[0];
    connection = integration.connection;
    const view = githubConnectionState(connection, integration.authorization_flow_version || 1), connected = view.connected;
    $('#github-status').textContent = view.label;
    $('#github-account').textContent = view.description;
    $('#github-modal-status').textContent = view.label;
    $('#github-modal-status').dataset.connected = String(connected);
    $('#verify-github').disabled = !view.canVerify;
    $('#disconnect-github').disabled = !view.canDisconnect;
    $('#disconnect-github').textContent = view.cleanupPending ? 'Reintentar limpieza' : 'Desconectar';
    $('#load-github-repos').disabled = !connected;
    $('#link-github-alexa').disabled = !connected;
    $('#select-github-branch').disabled = $('#review-github-merge').disabled = !connected || !connection.repositories.length;
    $('#connect-github').disabled = Boolean(view.requiresRestart || view.cleanupPending || (connection && connection.status !== 'revoked'));
    $('#github-device').classList.toggle('hidden', Boolean(view.requiresRestart || connection?.status !== 'connecting'));
    if (view.requiresRestart || (wasConnecting && connection?.status !== 'connecting')) message(view.description, !connected);
  }
  function node(tag, text, className) { const element = document.createElement(tag); element.textContent = text; if (className) element.className = className; return element; }
  async function jobs() {
    const data = await api('/api/jobs');
    const version = JSON.stringify(data.jobs.map(job => [job.id, job.revision]));
    if (version === jobsVersion) return;
    jobsVersion = version;
    const list = $('#jobs-list'); list.replaceChildren();
    $('#jobs-message').textContent = data.jobs.length ? `${data.jobs.length} trabajo(s)` : 'Todavía no hay trabajos.';
    data.jobs.sort((a, b) => b.created_at - a.created_at).forEach(job => {
      const card = node('article', '', 'job-item');
      card.append(node('strong', `${job.request.integration_id} · ${messages[job.status] || job.status}`));
      card.append(node('p', messages[job.result_code] || job.result_code || job.stage));
      if (!['completed', 'failed', 'cancelled', 'interrupted', 'timed_out', 'unknown_outcome'].includes(job.status)) {
        const cancel = node('button', 'Cancelar', 'button secondary small'); cancel.type = 'button';
        cancel.addEventListener('click', () => perform(async () => { await api(`/api/jobs/${job.id}/cancel`, 'POST', {}); await jobs(); message('Cancelación registrada.'); }));
        card.append(cancel);
      }
      if (job.continuation_id && job.result.choices?.length) {
        const select = document.createElement('select'); select.setAttribute('aria-label', 'Rama del trabajo');
        job.result.choices.forEach(choice => { const option = node('option', `${choice.repository} · ${choice.reference}`); option.value = choice.choice_id; select.append(option); });
        const choose = node('button', 'Seleccionar esta rama', 'button secondary small'); choose.type = 'button';
        choose.addEventListener('click', () => perform(async () => { await api(`/api/jobs/${job.id}/input`, 'POST', { continuation_id: job.continuation_id, revision: job.revision, choice_id: select.value }); await jobs(); message('Selección enviada.'); }));
        card.append(select, choose);
      }
      const details = document.createElement('details'); details.append(node('summary', 'Evidencia y alcance'), node('pre', JSON.stringify(job.result, null, 2)));
      card.append(details); list.append(card);
    });
  }
  $('#integration-unlock').addEventListener('submit', event => { event.preventDefault(); perform(async () => {
    const data = await api('/api/session/bootstrap', 'POST', { code: $('#integration-code').value.trim() });
    csrf = data.csrf; $('#integration-code').value = ''; unlocked(); await catalog(); await jobs(); message('Sesión local activa.');
  }); });
  $('#connect-github').addEventListener('click', () => perform(async () => {
    const data = await api('/api/integrations/github/connect', 'POST', { app_id: Number($('#github-app-id').value), client_id: $('#github-client-id').value.trim() });
    $('#github-user-code').textContent = data.user_code; $('#github-device').classList.remove('hidden');
    await catalog(); message('Autoriza el código en GitHub. Comprobaremos la conexión automáticamente.');
  }));
  $('#verify-github').addEventListener('click', () => perform(async () => { await api('/api/integrations/github/verify', 'POST', {}); await catalog(); message('Conexión verificada.'); }));
  $('#disconnect-github').addEventListener('click', () => perform(async () => {
    $('#disconnect-github').disabled = true;
    try { await api('/api/integrations/github/disconnect', 'POST', {}); }
    catch (error) { await catalog(); throw error; }
    await catalog(); await jobs();
    $('#github-repository-settings').classList.add('hidden');
    message('Acceso local revocado. Puedes retirar también la autorización en los ajustes de GitHub.');
  }));
  $('#load-github-repos').addEventListener('click', () => perform(async () => {
    available = await api('/api/integrations/github/repositories');
    $('#github-repository').replaceChildren(...available.map((repo, index) => { const option = node('option', repo.full_name); option.value = index; return option; }));
    $('#github-repository-settings').classList.toggle('hidden', available.length === 0); defaultBase(); message(available.length ? 'Selecciona el repositorio y sus ramas.' : 'Esta App no tiene repositorios accesibles.');
  }));
  function defaultBase() { $('#github-base').value = available[Number($('#github-repository').value)]?.default_branch || ''; }
  $('#github-repository').addEventListener('change', defaultBase);
  $('#save-github-repo').addEventListener('click', () => perform(async () => {
    const repo = available[Number($('#github-repository').value)]; if (!repo) return;
    await api('/api/integrations/github/repositories', 'PUT', [{ repository_id: repo.repository_id, installation_id: repo.installation_id,
      full_name: repo.full_name, creation_base_branch: $('#github-base').value.trim(), merge_targets: $('#github-targets').value.split(',').map(s => s.trim()).filter(Boolean) }]);
    await catalog(); message('Repositorio guardado. Las operaciones de escritura siguen pendientes de sus guardas.');
  }));
  $('#link-github-alexa').addEventListener('click', () => perform(async () => { await api('/api/integrations/github/alexa', 'POST', { skill_id: $('#github-alexa-skill').value.trim(), user_id: $('#github-alexa-user').value.trim() }); alexaMessage('Identidad Alexa vinculada.'); }, alexaMessage));
  async function submit(operation) {
    await api('/api/jobs', 'POST', { kind: 'selection', integration_id: 'github', connection_id: connection.id, project_id: null,
      capability: 'github.targets.resolve', input: { operation, target_branch_alias: $('#merge-branch').value.trim() }, idempotency_key: crypto.randomUUID() });
    await jobs(); message('Trabajo encolado.');
  }
  $('#select-github-branch').addEventListener('click', () => perform(() => submit('create_pr')));
  $('#review-github-merge').addEventListener('click', () => perform(() => submit('squash_merge')));
  $('#fixture-job').addEventListener('click', () => perform(async () => {
    await api('/api/jobs', 'POST', { kind: 'execution', integration_id: 'fixture', connection_id: null, project_id: 'fixture', capability: 'fixture.inspect', input: {}, idempotency_key: crypto.randomUUID() }); await jobs(); message('Fixture local encolada.');
  }));
  $('#refresh-jobs').addEventListener('click', () => perform(jobs));
  async function resume() { try { const data = await api('/api/session'); if (data.authenticated) { csrf = data.csrf; unlocked(); await catalog(); await jobs(); } } catch { message('No se pudo cargar la sesión de integraciones.', true); } }
  resume();
  setInterval(async () => {
    if (!csrf || busy || document.hidden) return;
    try { if (connection?.status === 'connecting') await catalog(); if (!$('#panel-integrations').classList.contains('hidden')) await jobs(); }
    catch (error) { $('#jobs-message').textContent = error.message; }
  }, 5000);
}
