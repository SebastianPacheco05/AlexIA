import './style.css';
import { icons, reveal, setupDialogs } from './components.js';
import { hasChanges } from './settings-state.js';
import { setupIntegrations } from './integrations.js';

const $ = selector => document.querySelector(selector);
const state = { data: null, values: new Map(), dirty: false, saving: false };
const openDialog = setupDialogs();
setupIntegrations();
icons();
const pages = {
  provider: ['Inteligencia', 'Un poco de inteligencia.', 'Mucho de ti.', 'Elige cómo piensa tu asistente. Nosotros nos encargamos del resto.'],
  prompt: ['Metaprompt', 'Su voz.', 'Tus instrucciones.', 'Define la personalidad y el estilo que acompañan cada respuesta.'],
  environment: ['Entorno', 'Cada pieza.', 'En su lugar.', 'Gestiona la configuración de AlexIA desde un solo espacio.'],
  integrations: ['Integraciones', 'Tus herramientas.', 'Más cerca.', 'Un punto de encuentro para las aplicaciones de tu asistente.'],
  settings: ['Configuración', 'Tu asistente.', 'Bien conectado.', 'Prepara la conexión de AlexIA con Alexa desde un solo lugar.'],
};
const labels = {
  BIND_ADDR: 'Dirección del servicio Alexa · requiere reinicio', UI_HOST: 'Host local de la interfaz · requiere reinicio',
  UI_PORT: 'Puerto de la interfaz · requiere reinicio', AI_PROVIDER: 'Proveedor activo',
  UI_MODE: 'dev para Vite, static para la interfaz compilada · requiere reinicio',
  GEMINI_MODEL: 'Nombre base del modelo', GEMINI_LEVEL: 'low, medium o high', ALEXA_SKILL_ID: 'ID de tu skill de Alexa',
  AGY_BIN: 'Ruta de Antigravity; vacío usa la detección automática', ALEXA_NODE_BIN: 'Ruta de Node.js; vacío usa node',
  ALEXA_OPENSSL_BIN: 'Ruta de OpenSSL', ALEXA_VERIFIER_PATH: 'Ruta del verificador; vacío usa la incluida en el proyecto',
};

function notice(message, error = false) {
  $('#notice').textContent = message;
  $('#notice').classList.toggle('hidden', !message);
  $('#notice').classList.toggle('error', error);
}
function dirty() {
  const value = hasChanges(state.data, state.values, $('#metaprompt').value);
  state.dirty = value;
  $('#save-state').textContent = value ? 'Tienes cambios sin guardar' : 'Configuración guardada';
  $('#save-state-hint').textContent = value ? 'Guarda para aplicar tus ajustes al asistente.' : 'Todo está al día. Puedes seguir ajustándolo.';
  $('#dirty-dot').classList.toggle('dirty', value);
  $('#save').disabled = state.saving || !value;
  $('.save-bar').classList.toggle('hidden', !value && !state.saving);
}
async function request(method, body) {
  const response = await fetch('/api/settings', {
    method, headers: { 'X-Alexia-UI': '1', ...(body ? { 'Content-Type': 'application/json' } : {}) },
    body: body ? JSON.stringify(body) : undefined, signal: AbortSignal.timeout(60000),
  });
  const data = await response.json().catch(() => ({ error: 'El servicio no devolvió una respuesta válida.' }));
  if (!response.ok) throw new Error(data.error || 'No se pudo completar la operación.');
  return data;
}
function syncReasoning() {
  document.querySelectorAll('[name="reasoning"]').forEach(input => { input.checked = input.value === state.values.get('GEMINI_LEVEL'); });
}
function updateValue(key, value, source) {
  state.values.set(key, value);
  document.querySelectorAll('[data-env]').forEach(input => {
    if (input !== source && input.dataset.env === key) input.value = value ?? '';
  });
  if (key === 'GEMINI_LEVEL') syncReasoning();
  dirty();
}
function filterVariables() {
  const query = $('#env-search').value.trim().toLowerCase();
  const rows = [...document.querySelectorAll('.env-row')];
  rows.forEach(row => row.classList.toggle('hidden', !row.dataset.search.includes(query)));
  const count = rows.filter(row => !row.classList.contains('hidden')).length;
  $('#env-count').textContent = query ? `${count}/${rows.length}` : `${rows.length}`;
  $('#env-empty').classList.toggle('hidden', count !== 0);
}
function envField(field) {
  const wrapper = document.createElement('div'); wrapper.className = 'field env-row';
  wrapper.dataset.search = `${field.key} ${labels[field.key] || ''}`.toLowerCase();
  const label = document.createElement('label'); label.htmlFor = `env-${field.key}`; label.textContent = field.key;
  const input = document.createElement('input');
  input.id = label.htmlFor; input.dataset.env = field.key; input.value = field.value ?? '';
  input.type = field.secret ? 'password' : 'text'; input.autocomplete = 'off'; input.spellcheck = false;
  input.placeholder = field.secret && field.configured ? 'Guardado · deja vacío para conservar' : 'Sin configurar';
  const help = document.createElement('p'); help.className = 'hint'; help.id = `help-${field.key}`;
  help.textContent = labels[field.key] || (field.secret ? 'Valor protegido; no se muestra al cargar.' : 'Variable personalizada');
  input.setAttribute('aria-describedby', help.id);
  input.addEventListener('input', () => updateValue(field.key, field.secret && input.value === '' ? null : input.value, input));
  wrapper.append(label, input, help); return wrapper;
}
function render(data) {
  state.data = data;
  state.values = new Map(data.environment.map(field => [field.key, field.value]));
  $('#environment-fields').replaceChildren(...data.environment.map(envField));
  document.querySelectorAll('[data-env]').forEach(input => { input.value = state.values.get(input.dataset.env) ?? ''; });
  syncReasoning(); filterVariables();
  $('#metaprompt').value = data.metaprompt;
  $('#prompt-count').textContent = `${data.metaprompt.length} caracteres`;
  $('#backend-address').textContent = data.backend_address;
  $('#ui-address').textContent = data.ui_address;
  $('#restart-notice').classList.toggle('hidden', !data.restart_required);
  $('#connection-label').textContent = data.status === 'ready' ? 'AlexIA conectada' : data.status === 'error' ? 'Revisar conexión' : 'Alexa sin configurar';
  $('#connection-dot').className = `status-dot ${data.status === 'ready' ? 'ready' : 'error'}`;
  $('#assistant-status-title').textContent = data.status === 'ready' ? 'Lista para escucharte.' : 'Revisemos la conexión.';
  $('#assistant-status-description').textContent = data.status === 'ready' ? 'Tu configuración está cargada. Abre tu asistente y empieza a conversar.' : 'Revisa la configuración del servicio antes de iniciar una conversación.';
  $('#settings-fields').disabled = false;
  dirty();
  if (data.error) notice(data.error, true);
}
async function load() {
  try { notice(''); render(await request('GET')); }
  catch (error) {
    notice(error.name === 'TimeoutError' ? 'La conexión tardó demasiado. Intenta recargar.' : error.message, true);
    $('#connection-label').textContent = 'Sin conexión'; $('#connection-dot').className = 'status-dot error';
    $('#save-state').textContent = 'No se pudo cargar la configuración';
    $('#assistant-status-title').textContent = 'Servicio no disponible.';
    $('#assistant-status-description').textContent = 'No pudimos consultar el estado. Comprueba que AlexIA siga en ejecución.';
  }
}

document.querySelectorAll('.nav-item').forEach(button => button.addEventListener('click', () => {
  document.querySelectorAll('.nav-item').forEach(item => {
    item.classList.toggle('active', item === button);
    if (item === button) item.setAttribute('aria-current', 'page'); else item.removeAttribute('aria-current');
  });
  document.querySelectorAll('.settings-panel').forEach(panel => panel.classList.toggle('hidden', panel.id !== `panel-${button.dataset.tab}`));
  if (button.dataset.tab === 'environment') {
    $('#env-search').value = ''; filterVariables();
  }
  const [breadcrumb, title, accent, description] = pages[button.dataset.tab];
  $('#breadcrumb-current').textContent = breadcrumb;
  const span = document.createElement('span'); span.textContent = accent;
  $('#page-title').replaceChildren(document.createTextNode(title), document.createElement('br'), span);
  $('#page-description').textContent = description;
  reveal(document.getElementById(`panel-${button.dataset.tab}`));
}));
document.querySelectorAll('select[data-env], #model').forEach(input => input.addEventListener('input', () => updateValue(input.dataset.env, input.value, input)));
document.querySelectorAll('[name="reasoning"]').forEach(input => input.addEventListener('change', () => updateValue('GEMINI_LEVEL', input.value, input)));
$('#env-search').addEventListener('input', filterVariables);
$('#metaprompt').addEventListener('input', () => { $('#prompt-count').textContent = `${$('#metaprompt').value.length} caracteres`; dirty(); });
document.querySelectorAll('[data-prompt-recipe]').forEach(button => button.addEventListener('click', () => {
  const editor = $('#metaprompt');
  const addition = `${editor.value.trimEnd() ? '\n\n' : ''}${button.dataset.promptRecipe}`;
  if (editor.value.length + addition.length > editor.maxLength) { notice('El metaprompt alcanzó su longitud máxima.', true); return; }
  editor.value = editor.value.trimEnd() + addition;
  editor.dispatchEvent(new Event('input')); editor.focus(); editor.setSelectionRange(editor.value.length, editor.value.length);
}));
$('#reload').addEventListener('click', () => { if (state.dirty) openDialog('discard-modal', $('#reload')); else load(); });
$('#confirm-discard').addEventListener('click', () => { $('#discard-modal').close(); load(); });
$('#open-variable').addEventListener('click', () => { $('#variable-error').classList.add('hidden'); openDialog('variable-modal', $('#open-variable')); $('#new-key').focus(); });
$('#add-variable').addEventListener('click', () => {
  const key = $('#new-key').value.trim();
  const error = !/^[A-Za-z_][A-Za-z0-9_]{0,79}$/.test(key) ? 'Usa letras, números y guiones bajos; empieza por una letra o guion bajo.' : state.values.has(key) ? 'Esa variable ya existe.' : '';
  $('#variable-error').textContent = error; $('#variable-error').classList.toggle('hidden', !error);
  if (error) { $('#new-key').focus(); return; }
  state.values.set(key, ''); $('#environment-fields').append(envField({ key, value: '', secret: true, configured: false }));
  $('#new-key').value = ''; $('#env-search').value = ''; filterVariables();
  $('#variable-modal').addEventListener('close', () => document.getElementById(`env-${key}`)?.focus(), { once: true });
  $('#variable-modal').close(); dirty(); notice('');
});
$('#new-key').addEventListener('keydown', event => { if (event.key === 'Enter') { event.preventDefault(); $('#add-variable').click(); } });
$('#settings-form').addEventListener('submit', async event => {
  event.preventDefault(); if (!state.data || state.saving) return;
  const payload = { revision: state.data.revision, environment: Object.fromEntries(state.values), metaprompt: $('#metaprompt').value };
  state.saving = true; $('#settings-fields').disabled = true; $('#save').disabled = true; $('#reload').disabled = true;
  $('#save-label').textContent = 'Aplicando…'; notice('Preparando Antigravity con tus ajustes. Puede tardar unos segundos.');
  try {
    render(await request('PUT', payload));
    notice('Cambios guardados y aplicados. Tu asistente ya usa la nueva configuración.');
  } catch (error) {
    notice(error.name === 'TimeoutError' ? 'El guardado sigue sin confirmarse. Recarga para comprobar el estado antes de reintentar.' : error.message, true);
    $('#settings-fields').disabled = false;
  } finally {
    state.saving = false; $('#reload').disabled = false; $('#save-label').textContent = 'Guardar cambios'; dirty();
  }
});
window.addEventListener('beforeunload', event => { if (state.dirty || state.saving) { event.preventDefault(); event.returnValue = ''; } });
$('#open-github').addEventListener('click', () => openDialog('github-modal', $('#open-github')));
$('#close-github').addEventListener('click', () => $('#github-modal').close());
$('#cancel-github').addEventListener('click', () => $('#github-modal').close());
$('#open-alexa').addEventListener('click', () => { openDialog('alexa-modal', $('#open-alexa')); loadTunnel(); });
$('#alexa-open-environment').addEventListener('click', () => {
  $('#alexa-modal').addEventListener('close', () => {
    const environmentButton = document.querySelector('.nav-item[data-tab="environment"]');
    environmentButton.click(); environmentButton.focus();
  }, { once: true });
  $('#alexa-modal').close();
});

async function loadTunnel() {
  try {
    const response = await fetch('/api/tunnel', { headers: { 'X-Alexia-UI': '1' }, signal: AbortSignal.timeout(5000) });
    if (!response.ok) throw new Error('Túnel no disponible');
    const { endpoint } = await response.json();
    $('#tunnel-endpoint').value = endpoint || '';
    $('#copy-tunnel').disabled = !endpoint;
    $('#alexa-endpoint').value = endpoint || '';
    $('#copy-alexa-endpoint').disabled = !endpoint;
    $('#tunnel-help').textContent = endpoint ? 'Pega esta URL en Amazon Developer → Endpoint → HTTPS.' : 'Inicia scripts/start-tunnel.ps1 para ver la URL.';
  } catch {
    $('#tunnel-endpoint').value = '';
    $('#copy-tunnel').disabled = true;
    $('#alexa-endpoint').value = '';
    $('#copy-alexa-endpoint').disabled = true;
    $('#tunnel-help').textContent = 'No pudimos consultar el túnel. Reintentando…';
  }
}
async function copyTunnel(statusElement) {
  try {
    await navigator.clipboard.writeText($('#tunnel-endpoint').value);
    statusElement.textContent = 'Endpoint copiado, incluido /alexa.';
  } catch {
    const input = $('#alexa-modal').open ? $('#alexa-endpoint') : $('#tunnel-endpoint');
    input.focus(); input.select();
    statusElement.textContent = 'Copia el texto seleccionado con Ctrl+C.';
  }
}
$('#copy-tunnel').addEventListener('click', () => copyTunnel($('#tunnel-help')));
$('#copy-alexa-endpoint').addEventListener('click', () => copyTunnel($('#alexa-copy-status')));
loadTunnel();
window.setInterval(() => { if (!document.hidden) loadTunnel(); }, 5000);
window.addEventListener('focus', loadTunnel);
load().then(() => reveal($('.workspace-grid')));
