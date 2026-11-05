# PLAN18 — Integración de Slack con AlexIA

Fecha: 2026-11-03. Estado: **plan propuesto; integración pendiente**. Prioridad: P2, avisos; P3, comandos de equipo.
Audiencia: comunidad técnica, inicialmente instalaciones personales con proyectos autorizados.

## 1. Resultado y alcance

Notificar resultados autorizados y recibir solicitudes vinculadas a identidades AlexIA, sin conceder acceso a todo el equipo por estar en un canal.

**Primer recorrido entregable:** Un usuario vinculado solicita una validación y recibe un aviso breve con enlace al resultado permitido.

**Alcance inicial:**

- OAuth de App e integración de notificaciones opt-in.
- Canales seleccionados y mensajes mínimos.
- Socket Mode para piloto local de comandos; HTTP firmado para distribución pública posterior.

**Ampliaciones posteriores:** Comandos/interactividad y Marketplace tras cumplir requisitos de distribución.

**Fuera de esta primera entrega:** Leer historiales completos, ejecutar texto de cualquier miembro o publicar datos privados por defecto.

## 2. Punto de partida real y prerrequisitos

No hay Slack ni identidad multiusuario. Una credencial del bot no vincula automáticamente miembros Slack con el propietario del equipo local.

La interfaz actual administra configuración; no existe todavía el catálogo ejecutable de conectores ni la API de trabajos descrita aquí. El estado actual y las dependencias transversales están en [PLAN00 — Base común](PLAN00-BASE-COMUN.md). Las rutas, tipos, módulos y opciones nuevos que siguen son propuestas de implementación, no contratos existentes.

**Prerrequisitos específicos:**

- Workspace fixture y App con permisos mínimos.
- Vinculación Slack team_id/user_id a identidad local con pairing.
- Decidir bot solo notificaciones frente a comandos y distribución.

**Dependencias entre planes:** PLAN00; consume resultados de cualquier plan sin exigir GitHub.

## 3. Decisión técnica y flujo de conexión

Separar transporte y capacidades. Avisos salientes usan Web API; comandos locales pueden usar Socket Mode con conexión saliente, app-level token y bot token distintos. La documentación indica que Socket Mode no se admite actualmente para apps del Marketplace público: no prometer ese camino de distribución. Para HTTP futuro, endpoint separado firma peticiones y devuelve acuse rápido, luego encola.

**Procedimiento de conexión o habilitación:**

1. Crear App de prueba y seleccionar scopes para las funciones concretas.
2. Instalar con OAuth v2; guardar team_id, bot identity y tokens en vault.
3. Elegir canal al que el bot tenga acceso y autorizar notificaciones.
4. Si se activan comandos, vincular al usuario mediante pairing explícito.
5. Enviar mensaje de prueba aprobado con datos ficticios.
6. Revocar/desconectar y detener conexiones/socket y avisos pendientes.

**Autenticación, renovación y revocación:** OAuth v2; bot token para Web API y app-level token con connections:write solo si hay Socket Mode. Si se habilita token rotation, implementar renovación según su contrato. No usar token del bot como identidad del usuario solicitante.

**Permisos externos y selección de recursos:** chat:write para mensajes; commands para comandos si se implementan; app_mentions:read solo para menciones. No pedir history o users:read sin necesidad demostrada. Canales y destinatarios se autorizan aparte.

Las credenciales residen en el almacén del sistema descrito en PLAN00; la configuración conserva solamente referencias. El navegador, el modelo y la respuesta de Alexa no reciben tokens. Una instalación comunitaria no comparte credenciales con otra. Si el producto exige un servicio OAuth confidencial, su secreto permanece en ese servicio; nunca se distribuye en el frontend, extensión o binario público.

## 4. Capacidades autorizables

Los identificadores de esta tabla son capacidades internas de AlexIA; no son nombres de scopes del proveedor.

| Capacidad propuesta | Operación | Guarda específica |
|---|---|---|
| `slack.notifications.send` | Enviar aviso opt-in | Canal y contenido/política autorizados |
| `slack.commands.receive` | Recibir solicitud | Usuario vinculado y proyecto permitido |
| `slack.connections.disconnect` | Desconectar | Revocar tokens y parar avisos |

Las lecturas pueden preautorizarse para recursos concretos. Las escrituras o ejecuciones se sujetan a una política explícita por proyecto. Una política previa válida evita pedir confirmación repetida; si falta, preparar una vista previa concreta. La autorización queda ligada a identidad, recurso, parámetros y versión. Texto de un issue, archivo o registro no modifica permisos.

## 5. Superficie externa y contrato del adaptador

| Operación | API, comando o mecanismo | Tratamiento de la respuesta |
|---|---|---|
| Mensaje | POST https://slack.com/api/chat.postMessage | Comprobar ok:false aunque HTTP200 |
| OAuth | OAuth v2 authorize/token | State y scope de instalación |
| Socket Mode | apps.connections.open y WebSocket autenticado | Acuse envelope_id y deduplicación |
| HTTP futuro | Ruta /webhooks/slack propuesta | Firma v0 y timestamp según documentación Slack |

El enlace a localhost no funciona para otro miembro del workspace: enviar solo referencias útiles al usuario local o usar un portal autenticado futuro, nunca publicar el panel local mediante túnel. Una respuesta pública en canal requiere política de audiencia. No transformar mensajes de Slack en comandos shell.

**Petición interna ilustrativa:**

```json
{
  "integration_id": "slack",
  "project_id": "project-demo",
  "capability": "slack.notifications.send",
  "input": {
    "channel_id": "channel-demo",
    "job_id": "job-demo",
    "template_id": "task-completed"
  },
  "idempotency_key": "demo-slack-001"
}
```

Se envía al propuesto `POST /api/jobs`, exclusivo de la API administrativa autenticada. Respuesta: HTTP 202 con `job_id`, `status` y `status_url`. `GET /api/jobs/{job_id}` consulta progreso; `POST /api/jobs/{job_id}/cancel` solicita cancelación. Validar esquema y pertenencia antes de consultar credenciales o iniciar procesos. No existe un endpoint que acepte una URL o comando arbitrario del modelo.

**Datos específicos normalizados:** team_id, user_id vinculado, channel_id, message ts, event/envelope ID, job_id y delivery_state.

El resultado incluye `schema_version`, identidad del recurso, instante, versión de herramienta/API, alcance, evidencia saneada, advertencias y estado de cobertura. La respuesta puede terminar con hallazgos aunque el proceso haya funcionado correctamente. Diferenciar ausencia de resultados, permisos insuficientes, cobertura parcial y ejecución fallida.

## 6. Archivos y cambios previstos en AlexIA

Los archivos nuevos se crean durante la implementación. Evitar duplicar contratos del núcleo en el conector.

| Ruta relativa al repositorio | Responsabilidad |
|---|---|
| `src/integrations/slack.rs` | Web API y OAuth |
| `src/integrations/slack_events.rs` | Socket/HTTP posterior |
| `tests/integrations_slack.rs` | Identidad y deduplicación |
| `ui/src/integrations/slack.js` | Canales y opt-in |

`src/lib.rs` registraría los módulos; `src/control.rs` delegaría rutas administrativas al núcleo de integraciones. `ui/src/main.js` consumiría estados y resultados; `ui/index.html`/`ui/src/components.js` presentarían la conexión. Conservar las guardas loopback actuales y actualizar [BACKEND.md](../../.agents/.docs/BACKEND.md), [UI.md](../../.agents/.docs/UI.md) y el documento operativo correspondiente al implementar.

## 7. Configuración propuesta

| Campo lógico | Tipo/protección | Uso |
|---|---|---|
| `team_id` | ID | Workspace |
| `bot_token_ref` | Secreto | Web API |
| `app_token_ref` | Secreto opcional | Socket Mode |
| `allowed_channel_ids` | Lista | Destinos |
| `notification_policy` | Objeto | Eventos/plantillas opt-in |

Estos campos pertenecen a un registro validado de integración, no a un conjunto de secretos nuevos en `.env`. Escribirlos desde la API local con control de revisión. Exponer solamente `configured`, permisos y errores saneados. La presencia de configuración no equivale a conexión verificada.

## 8. Implementación por entregas verificables

### 18.1 — Conectar y elegir destino

**Trabajo:** OAuth y política de mensajes.

**Dependencias:** BC-01 a BC-05 de PLAN00; completar los prerrequisitos de este documento.

**Aceptación:**

- [ ] No lee historial sin scope.
- [ ] Canal no autorizado rechazado.

**Verificación:** Workspace fixture y mocks.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### 18.2 — Enviar aviso controlado

**Trabajo:** Plantilla mínima y estado de entrega.

**Dependencias:** Tarea 18.1 completada y contrato de salida estable.

**Aceptación:**

- [ ] HTTP200/ok:false falla.
- [ ] Reintento no duplica mensaje.

**Verificación:** Mensaje ficticio explícitamente autorizado.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### 18.3 — Vincular comandos opcionales

**Trabajo:** Pairing y recepción/deduplicación.

**Dependencias:** Tarea 18.2 completada y contrato de salida estable.

**Aceptación:**

- [ ] Miembro no vinculado no ejecuta.
- [ ] Evento repetido no duplica job.

**Verificación:** Dos usuarios fixture y CLI de UI.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### Checkpoint de salida

- [ ] Primera operación real completa desde UI hasta proveedor/herramienta y resultado visible.
- [ ] Denegación comprobada al cambiar identidad, proyecto o recurso autorizado.
- [ ] Cancelación, error y desconexión dejan estados coherentes y permiten recuperación.
- [ ] No se modificó el contrato de firma, certificado, timestamp e ID de Alexa.
- [ ] La documentación distingue simulación, prueba real y funciones pendientes.

## 9. Errores, límites y recuperación

Acusar eventos dentro del plazo documentado sin esperar al job. Respetar 429 y cola por canal; outbox con idempotencia lógica. Timeout de mensaje necesita reconciliar antes de reenviar. En heartbeat/notificaciones mantener silencio si el estado no cambia.

Límites iniciales de diseño: una ejecución por proyecto, cola local acotada, cuerpo administrativo de hasta 128 KiB y resultados paginados; los artefactos grandes se entregan por identificador. Los valores se medirán antes de liberar y no describen capacidad demostrada. HTTP 401 expira la sesión; 403 no se corrige ampliando permisos silenciosamente; 429 respeta `Retry-After` cuando exista. Un timeout después de una escritura produce estado `unknown_outcome` hasta reconciliar, sin repetición ciega.

## 10. Riesgos particulares y controles

| Riesgo | Control previsto | Evidencia requerida |
|---|---|---|
| Usuario Slack obtiene permisos locales | Pairing y autorización por proyecto | Miembro no vinculado denegado |
| Datos privados en canal | Plantilla mínima y audiencia | Centinela no publicado |
| Socket Mode confundido con Marketplace | Separar distribución | Decisión documentada |

**Datos y retención:** No copiar código/logs completos a canales. Adjuntos y enlaces requieren acceso del destinatario; mensajes no contienen credenciales ni rutas privadas innecesarias.

## 11. Pruebas del adaptador y del flujo completo

**Fixtures y pruebas de contrato:**

- Slack ok:false, token_revoked y channel_not_found.
- Evento duplicado, usuario no vinculado y canal privado.
- Timeout de envío y reconexión Socket Mode.

**Prueba real de aceptación:** En workspace fixture, conectar y enviar un aviso de datos ficticios cuando la implementación tenga autorización de publicación; hasta entonces validar con mocks sin mandar mensajes.

**Prueba de interfaz con la skill Playwright CLI:**

1. Configurar canal y opt-in.
2. Ver vista previa de mensaje/destinatario.
3. Mostrar fallo de entrega saneado.
4. Revocar y comprobar que no se encolan avisos nuevos.

Usar sesión de navegador desechable; obtener referencias con `snapshot` y ejecutar comprobaciones con `run-code --filename=...` cuando se necesiten aserciones. Para el estado simulado, interceptar únicamente la API propuesta e identificar el resultado como prueba de UI. La prueba real debe comprobar el recurso externo o artefacto local. No automatizar pantallas de login ni guardar cookies del usuario para sustituir APIs oficiales.

Comandos de validación cuando se implemente: `cargo test --locked --test integrations_slack` para el archivo propuesto, `cargo check --locked --all-targets`, las pruebas de UI afectadas y `npm --prefix ui run build`. Estos comandos son objetivos; no se afirma que esos tests existan hoy. Añadir pruebas de esquema, autorización y recuperación antes de habilitar capacidades de escritura.

## 12. Despliegue, reversión y criterio de finalización

Habilitar primero para una instalación piloto mediante flag de integración y solamente las capacidades verificadas. Registrar versión y permisos. Después ampliar a un segundo proyecto y comprobar separación de conexiones; no equivale a un despliegue SaaS multiusuario.

**Reversión específica:** Desactivar avisos, drenar/cancelar outbox y revocar tokens; borrar solo mensajes propios si una acción explícita lo solicita.

Deshabilitar el conector impide nuevos trabajos; cancelar o drenar los existentes según su efecto. Revocar credenciales cuando corresponda. Desconectar no borra recursos ajenos ni deshace efectos externos de forma automática. La reversión del software y la compensación de una operación externa son procedimientos distintos.

**Se considera completada cuando:** Avisos opt-in e identidad se verifican; comandos no dan permisos implícitos y no hay duplicados ante reintentos.

**Decisiones o condiciones pendientes:** Distribución Marketplace versus instalación interna y portal de resultados para usuarios remotos.

## 13. Fuentes oficiales y revalidación

Consultadas el 2026-11-03. Sustentan los mecanismos externos; el diseño de AlexIA es una propuesta propia. Revalidar scopes, rutas, versión, licencia y condiciones de distribución antes de implementar. Los enlaces de contratos específicos prevalecen sobre ejemplos generales; una página accesible no demuestra que una cuenta tenga habilitada la función.

- [OAuth Slack](https://docs.slack.dev/authentication/installing-with-oauth/) — Instalación y tokens
- [Web API](https://docs.slack.dev/apis/web-api/) — Operaciones y autenticación
- [Socket Mode](https://docs.slack.dev/apis/events-api/using-socket-mode/) — Conexión saliente y restricciones
