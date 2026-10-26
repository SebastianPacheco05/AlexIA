import './style.css';

const $ = (selector) => document.querySelector(selector);
const state = { data: null, values: new Map(), dirty: false, saving: false };
const labels = {
  BIND_ADDR: 'Dirección del servicio Alexa · requiere reinicio', UI_HOST: 'Host local de la interfaz · requiere reinicio',
  UI_PORT: 'Puerto de la interfaz · requiere reinicio', AI_PROVIDER: 'Proveedor activo',
  UI_MODE: 'dev para Vite, static para la interfaz compilada · requiere reinicio',
  GEMINI_MODEL: 'Nombre base del modelo', GEMINI_LEVEL: 'low, medium o high', ALEXA_SKILL_ID: 'ID de tu skill de Alexa',
  AGY_BIN: 'Ruta de Antigravity; vacío usa la detección automática', ALEXA_NODE_BIN: 'Ruta de Node.js; vacío usa node',
  ALEXA_OPENSSL_BIN: 'Ruta de OpenSSL', ALEXA_VERIFIER_PATH: 'Ruta del verificador; vacío usa la incluida en el proyecto',
};

function notice(message, error = false) {
  const element = $('#notice');
  element.textContent = message;
  element.classList.toggle('hidden', !message);
  element.classList.toggle('error', error);
}

function dirty(value) {
  state.dirty = value;
  $('#save-state').textContent = value ? 'Tienes cambios sin guardar' : 'Configuración guardada';
  $('#dirty-dot').classList.toggle('dirty', value);
  $('#save').disabled = state.saving || !value;
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

function updateValue(key, value, source) {
  state.values.set(key, value);
  document.querySelectorAll('[data-env]').forEach(input => {
    if (input !== source && input.dataset.env === key) input.value = value ?? '';
  });
  dirty(true);
}

function envField(field) {
  const wrapper = document.createElement('div'); wrapper.className = 'field';
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
  $('#metaprompt').value = data.metaprompt;
  $('#prompt-count').textContent = `${data.metaprompt.length} caracteres`;
  $('#backend-address').textContent = data.backend_address;
  $('#ui-address').textContent = data.ui_address;
  $('#restart-notice').classList.toggle('hidden', !data.restart_required);
  $('#connection-label').textContent = data.status === 'ready' ? 'AlexIA conectada' : data.status === 'error' ? 'Revisar conexión' : 'Alexa sin configurar';
  $('#connection-dot').className = `status-dot ${data.status === 'ready' ? 'ready' : 'error'}`;
  $('#settings-fields').disabled = false;
  dirty(false);
  if (data.error) notice(data.error, true);
}

async function load() {
  try { notice(''); render(await request('GET')); }
  catch (error) {
    notice(error.name === 'TimeoutError' ? 'La conexión tardó demasiado. Intenta recargar.' : error.message, true);
    $('#connection-label').textContent = 'Sin conexión'; $('#save-state').textContent = 'No se pudo cargar la configuración';
  }
}

document.querySelectorAll('.nav-item').forEach(button => button.addEventListener('click', () => {
  document.querySelectorAll('.nav-item').forEach(item => {
    item.classList.toggle('active', item === button);
    if (item === button) item.setAttribute('aria-current', 'page'); else item.removeAttribute('aria-current');
  });
  document.querySelectorAll('.settings-panel').forEach(panel => panel.classList.toggle('hidden', panel.id !== `panel-${button.dataset.tab}`));
  $('.save-bar').classList.toggle('hidden', button.dataset.tab === 'integrations');
}));

document.querySelectorAll('select[data-env], #model').forEach(input => input.addEventListener('input', () => updateValue(input.dataset.env, input.value, input)));
$('#metaprompt').addEventListener('input', () => { $('#prompt-count').textContent = `${$('#metaprompt').value.length} caracteres`; dirty(true); });
$('#reload').addEventListener('click', load);
$('#add-variable').addEventListener('click', () => {
  const key = $('#new-key').value.trim();
  if (!/^[A-Za-z_][A-Za-z0-9_]{0,79}$/.test(key)) { notice('Escribe un nombre válido: letras, números y guiones bajos.', true); $('#new-key').focus(); return; }
  if (state.values.has(key)) { notice('Esa variable ya existe.', true); return; }
  state.values.set(key, ''); $('#environment-fields').append(envField({ key, value: '', secret: true, configured: false }));
  $('#new-key').value = ''; dirty(true); notice(''); document.getElementById(`env-${key}`).focus();
});

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
    state.saving = false; $('#reload').disabled = false; $('#save-label').textContent = 'Guardar y aplicar'; $('#save').disabled = !state.dirty;
  }
});
window.addEventListener('beforeunload', event => { if (state.dirty || state.saving) { event.preventDefault(); event.returnValue = ''; } });
load();
