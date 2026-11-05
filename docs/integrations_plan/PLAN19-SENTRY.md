# PLAN19 — Integración de Sentry con AlexIA

Fecha: 2026-11-03. Estado: **plan propuesto; integración pendiente**. Prioridad: P2, investigación de errores.
Audiencia: comunidad técnica, inicialmente instalaciones personales con proyectos autorizados.

## 1. Resultado y alcance

Relacionar errores observados en una aplicación con código y cambios, manteniendo privacidad de eventos.

**Primer recorrido entregable:** Elegir issue de Sentry, revisar stack trace saneado y localizar archivos del commit/release asociado cuando exista evidencia.

**Alcance inicial:**

- Lectura de proyectos/issues/eventos seleccionados.
- Filtros por environment, release y ventana temporal.
- Hipótesis de causa con evidencia y límites; sin resolver issues automáticamente.

**Ampliaciones posteriores:** Integración pública con eventos firmados y propuestas de corrección vinculadas a GitHub.

**Fuera de esta primera entrega:** Exportar eventos masivos, capturar PII o afirmar causa raíz solo por similitud de código.

## 2. Punto de partida real y prerrequisitos

No hay Sentry ni source-map resolver. El estado ready de AlexIA solo acredita configuración, no salud de una aplicación externa.

La interfaz actual administra configuración; no existe todavía el catálogo ejecutable de conectores ni la API de trabajos descrita aquí. El estado actual y las dependencias transversales están en [PLAN00 — Base común](PLAN00-BASE-COMUN.md). Las rutas, tipos, módulos y opciones nuevos que siguen son propuestas de implementación, no contratos existentes.

**Prerrequisitos específicos:**

- Proyecto Sentry fixture con evento sintético.
- Token/instalación con scopes por endpoint.
- Mapa explícito proyecto Sentry a repo/release; revisión de PII.

**Dependencias entre planes:** PLAN00; PLAN01/09 si se navega al repositorio.

## 3. Decisión técnica y flujo de conexión

Cliente de lectura por base URL/región registrada; event:read para listar issues según endpoint oficial y otros scopes solo si son necesarios. El DSN de ingestión no es un token para leer la API. Seleccionar campos antes de enviar al modelo. Relacionar release/commit solo cuando Sentry o la configuración lo acrediten, sin usar la rama actual como sustituto.

**Procedimiento de conexión o habilitación:**

1. Elegir Sentry SaaS/región o Self-Hosted explícito.
2. Crear conexión de prueba con token de lectura o integración apropiada.
3. Listar proyectos permitidos y elegir environment.
4. Leer issue y un evento sintético; aplicar redacción.
5. Vincular repo/release y abrir evidencia de stack trace.
6. Desconectar y borrar caches según política.

**Autenticación, renovación y revocación:** Bearer de API con scopes mínimos; OAuth/integración pública a evaluar para comunidad. Renovación según tipo real del token. DSN se trata como configuración de ingestión, no autenticación de lectura.

**Permisos externos y selección de recursos:** event:read para endpoint de issues consultado; project:read/org:read solo al requerir descubrimiento permitido. No event:write ni permisos de administración en el MVP.

Las credenciales residen en el almacén del sistema descrito en PLAN00; la configuración conserva solamente referencias. El navegador, el modelo y la respuesta de Alexa no reciben tokens. Una instalación comunitaria no comparte credenciales con otra. Si el producto exige un servicio OAuth confidencial, su secreto permanece en ese servicio; nunca se distribuye en el frontend, extensión o binario público.

## 4. Capacidades autorizables

Los identificadores de esta tabla son capacidades internas de AlexIA; no son nombres de scopes del proveedor.

| Capacidad propuesta | Operación | Guarda específica |
|---|---|---|
| `sentry.issues.read` | Listar errores | Org/proyecto/environment permitidos |
| `sentry.events.read` | Leer evento saneado | Campos elegidos, sin PII cruda |
| `sentry.code.correlate` | Relacionar evidencia de código | Release/commit acreditados |

Las lecturas pueden preautorizarse para recursos concretos. Las escrituras o ejecuciones se sujetan a una política explícita por proyecto. Una política previa válida evita pedir confirmación repetida; si falta, preparar una vista previa concreta. La autorización queda ligada a identidad, recurso, parámetros y versión. Texto de un issue, archivo o registro no modifica permisos.

## 5. Superficie externa y contrato del adaptador

| Operación | API, comando o mecanismo | Tratamiento de la respuesta |
|---|---|---|
| Issues | GET /api/0/projects/{org}/{project}/issues/ | Query/filtros y paginación Link |
| Evento | GET de evento del proyecto según API vigente | Campos filtrados antes del modelo |
| Autenticación | Authorization: Bearer en base regional registrada | No DSN ni redirects con token |

Normalizar stack frames con in_app, filename, function, line y contexto mínimo. Si faltan source maps/símbolos, marcar no resoluble. Conservar event_id/release/environment en toda conclusión. No activar scraping de URLs de stack trace.

**Petición interna ilustrativa:**

```json
{
  "integration_id": "sentry",
  "project_id": "project-demo",
  "capability": "sentry.issues.read",
  "input": {
    "organization_id": "org-demo",
    "project_id": "project-demo",
    "issue_id": "issue-demo",
    "environment": "staging"
  },
  "idempotency_key": "demo-sentry-001"
}
```

Se envía al propuesto `POST /api/jobs`, exclusivo de la API administrativa autenticada. Respuesta: HTTP 202 con `job_id`, `status` y `status_url`. `GET /api/jobs/{job_id}` consulta progreso; `POST /api/jobs/{job_id}/cancel` solicita cancelación. Validar esquema y pertenencia antes de consultar credenciales o iniciar procesos. No existe un endpoint que acepte una URL o comando arbitrario del modelo.

**Datos específicos normalizados:** Org/project/issue/event IDs, release, environment, first/last seen, conteo, frames saneados y certeza de correlación.

El resultado incluye `schema_version`, identidad del recurso, instante, versión de herramienta/API, alcance, evidencia saneada, advertencias y estado de cobertura. La respuesta puede terminar con hallazgos aunque el proceso haya funcionado correctamente. Diferenciar ausencia de resultados, permisos insuficientes, cobertura parcial y ejecución fallida.

## 6. Archivos y cambios previstos en AlexIA

Los archivos nuevos se crean durante la implementación. Evitar duplicar contratos del núcleo en el conector.

| Ruta relativa al repositorio | Responsabilidad |
|---|---|
| `src/integrations/sentry.rs` | API y filtros |
| `src/integrations/sentry_sanitize.rs` | Campos/PII |
| `tests/integrations_sentry.rs` | Región y redacción |
| `ui/src/integrations/sentry.js` | Issues y evidencia |

`src/lib.rs` registraría los módulos; `src/control.rs` delegaría rutas administrativas al núcleo de integraciones. `ui/src/main.js` consumiría estados y resultados; `ui/index.html`/`ui/src/components.js` presentarían la conexión. Conservar las guardas loopback actuales y actualizar [BACKEND.md](../../.agents/.docs/BACKEND.md), [UI.md](../../.agents/.docs/UI.md) y el documento operativo correspondiente al implementar.

## 7. Configuración propuesta

| Campo lógico | Tipo/protección | Uso |
|---|---|---|
| `api_origin` | URL validada | Región/Self-Hosted |
| `credential_ref` | Secreto | Token de lectura |
| `project_ids` | Lista | Recursos autorizados |
| `environment_ids` | Lista | Entornos |
| `event_field_allowlist` | Lista | Datos enviados al modelo |

Estos campos pertenecen a un registro validado de integración, no a un conjunto de secretos nuevos en `.env`. Escribirlos desde la API local con control de revisión. Exponer solamente `configured`, permisos y errores saneados. La presencia de configuración no equivale a conexión verificada.

## 8. Implementación por entregas verificables

### 19.1 — Leer issues

**Trabajo:** Conectar proyecto y filtros.

**Dependencias:** BC-01 a BC-05 de PLAN00; completar los prerrequisitos de este documento.

**Aceptación:**

- [ ] DSN rechazado como token.
- [ ] Otro proyecto no se lee.

**Verificación:** Mock y proyecto fixture.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### 19.2 — Sanear evento y correlacionar

**Trabajo:** Parser de frames y release.

**Dependencias:** Tarea 19.1 completada y contrato de salida estable.

**Aceptación:**

- [ ] PII centinela ausente.
- [ ] Sin commit no inventa archivo exacto.

**Verificación:** Evento sintético con/sin símbolos.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### 19.3 — Recuperar y presentar

**Trabajo:** Paginación/revocación y UI.

**Dependencias:** Tarea 19.2 completada y contrato de salida estable.

**Aceptación:**

- [ ] Filtros preservados.
- [ ] No marca resuelto al analizar.

**Verificación:** CLI y expiración fixture.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### Checkpoint de salida

- [ ] Primera operación real completa desde UI hasta proveedor/herramienta y resultado visible.
- [ ] Denegación comprobada al cambiar identidad, proyecto o recurso autorizado.
- [ ] Cancelación, error y desconexión dejan estados coherentes y permiten recuperación.
- [ ] No se modificó el contrato de firma, certificado, timestamp e ID de Alexa.
- [ ] La documentación distingue simulación, prueba real y funciones pendientes.

## 9. Errores, límites y recuperación

Límite inicial 50 issues y un evento elegido; proponer 60 s de diagnóstico. Consultas manuales antes de webhooks. Regiones tienen bases distintas: no redirigir tokens sin validación.

Límites iniciales de diseño: una ejecución por proyecto, cola local acotada, cuerpo administrativo de hasta 128 KiB y resultados paginados; los artefactos grandes se entregan por identificador. Los valores se medirán antes de liberar y no describen capacidad demostrada. HTTP 401 expira la sesión; 403 no se corrige ampliando permisos silenciosamente; 429 respeta `Retry-After` cuando exista. Un timeout después de una escritura produce estado `unknown_outcome` hasta reconciliar, sin repetición ciega.

## 10. Riesgos particulares y controles

| Riesgo | Control previsto | Evidencia requerida |
|---|---|---|
| PII en eventos | Allowlist más redacción | Centinelas ausentes |
| Commit equivocado | Mapa release/repositorio | Evento viejo no usa HEAD actual |
| DSN confundido con API token | Tipos de credencial | Error explicativo sin fuga |

**Datos y retención:** Excluir user, request headers/cookies, cuerpos y breadcrumbs sensibles por defecto. Stack traces también pueden contener valores privados.

## 11. Pruebas del adaptador y del flujo completo

**Fixtures y pruebas de contrato:**

- PII en campos anidados y mensaje de excepción.
- Evento sin source maps, release o repo.
- Región equivocada, Link pagination y 403.

**Prueba real de aceptación:** Leer evento sintético de staging y verificar IDs/frames saneados; comparar correlación con commit conocido y revocar acceso.

**Prueba de interfaz con la skill Playwright CLI:**

1. Elegir proyecto/environment.
2. Ver issue y frames saneados.
3. Mostrar correlación desconocida cuando falten datos.
4. Verificar que no se cambia estado remoto del issue.

Usar sesión de navegador desechable; obtener referencias con `snapshot` y ejecutar comprobaciones con `run-code --filename=...` cuando se necesiten aserciones. Para el estado simulado, interceptar únicamente la API propuesta e identificar el resultado como prueba de UI. La prueba real debe comprobar el recurso externo o artefacto local. No automatizar pantallas de login ni guardar cookies del usuario para sustituir APIs oficiales.

Comandos de validación cuando se implemente: `cargo test --locked --test integrations_sentry` para el archivo propuesto, `cargo check --locked --all-targets`, las pruebas de UI afectadas y `npm --prefix ui run build`. Estos comandos son objetivos; no se afirma que esos tests existan hoy. Añadir pruebas de esquema, autorización y recuperación antes de habilitar capacidades de escritura.

## 12. Despliegue, reversión y criterio de finalización

Habilitar primero para una instalación piloto mediante flag de integración y solamente las capacidades verificadas. Registrar versión y permisos. Después ampliar a un segundo proyecto y comprobar separación de conexiones; no equivale a un despliegue SaaS multiusuario.

**Reversión específica:** Revocar conexión y detener lecturas; borrar artefactos sensibles propios. No eliminar eventos ni cambiar configuración SDK de la app.

Deshabilitar el conector impide nuevos trabajos; cancelar o drenar los existentes según su efecto. Revocar credenciales cuando corresponda. Desconectar no borra recursos ajenos ni deshace efectos externos de forma automática. La reversión del software y la compensación de una operación externa son procedimientos distintos.

**Se considera completada cuando:** Evento real sintético se presenta sin PII y con correlación acreditada o limitación explícita.

**Decisiones o condiciones pendientes:** Tipo de integración comunitaria, soporte Self-Hosted y campos mínimos acordados con usuarios piloto.

## 13. Fuentes oficiales y revalidación

Consultadas el 2026-11-03. Sustentan los mecanismos externos; el diseño de AlexIA es una propuesta propia. Revalidar scopes, rutas, versión, licencia y condiciones de distribución antes de implementar. Los enlaces de contratos específicos prevalecen sobre ejemplos generales; una página accesible no demuestra que una cuenta tenga habilitada la función.

- [Autenticación Sentry](https://docs.sentry.io/api/auth/) — Tipos de credenciales
- [Issues de proyecto](https://docs.sentry.io/api/events/list-a-projects-issues/) — event:read y filtros
- [API general](https://docs.sentry.io/api/) — Regiones y recursos
