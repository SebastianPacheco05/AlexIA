# PLAN16 — Integración de Jira con AlexIA

Fecha: 2026-11-03. Estado: **plan propuesto; integración pendiente**. Prioridad: P2/P3, equipos que lo utilicen.
Audiencia: comunidad técnica, inicialmente instalaciones personales con proyectos autorizados.

## 1. Resultado y alcance

Consultar issues de Jira Cloud y relacionarlos con tareas de desarrollo sin imponer un gestor nuevo al equipo.

**Primer recorrido entregable:** Autorizar un sitio, seleccionar proyecto e issue y preparar un diagnóstico con enlaces a código.

**Alcance inicial:**

- Jira Cloud REST v3 y OAuth 2.0 3LO.
- Sitio/cloudId y proyectos seleccionados.
- Lectura y borradores; comentarios/transiciones posteriores.

**Ampliaciones posteriores:** Publicar comentarios ADF y transiciones permitidas; Data Center como adaptador distinto.

**Fuera de esta primera entrega:** Administración Jira, asumir que todo issue puede transicionar a Done y reutilizar Cloud OAuth para Data Center.

## 2. Punto de partida real y prerrequisitos

No hay Jira. Un issue key no es identidad suficiente: el mismo ABC-1 puede existir en varios sitios.

La interfaz actual administra configuración; no existe todavía el catálogo ejecutable de conectores ni la API de trabajos descrita aquí. El estado actual y las dependencias transversales están en [PLAN00 — Base común](PLAN00-BASE-COMUN.md). Las rutas, tipos, módulos y opciones nuevos que siguen son propuestas de implementación, no contratos existentes.

**Prerrequisitos específicos:**

- App Atlassian 3LO y sitio fixture.
- Definir scopes por endpoint y formato Atlassian Document Format.
- Resolver componentes confidenciales sin empaquetar client secret.

**Dependencias entre planes:** PLAN00; vínculos con PLAN01/09 opcionales.

## 3. Decisión técnica y flujo de conexión

El flujo 3LO obtiene recursos accesibles y fija cloudId. Consultar vía https://api.atlassian.com/ex/jira/{cloudId}/rest/api/3. Usar búsqueda mejorada /search/jql y paginación de su contrato vigente, sin copiar /search histórico. JQL se construye desde campos y filtros permitidos; no convertir la pregunta directamente en una consulta global.

**Procedimiento de conexión o habilitación:**

1. Registrar app 3LO y callback exacto.
2. Autorizar scopes de lectura y offline_access si se requiere renovación.
3. Consultar accessible-resources y elegir sitio/cloudId.
4. Seleccionar proyecto autorizado y listar issues con campos mínimos.
5. Leer issue por ID/key dentro del sitio y producir borrador.
6. Desconectar y gestionar expiración/revocación de refresh.

**Autenticación, renovación y revocación:** OAuth 3LO con canje del lado confidencial; refresh rotativo si se solicita offline_access y es soportado. La distribución local debe resolver custodia del secret antes de implementar; un API token personal solo sería modo BYO explícito, no sustitución silenciosa.

**Permisos externos y selección de recursos:** Elegir scopes de lectura documentados para cada endpoint; read:jira-work es el conjunto clásico a evaluar. Escritura requiere su scope y permisos Browse/Edit/Transition del usuario en proyecto. No solicitar manage:jira-configuration.

Las credenciales residen en el almacén del sistema descrito en PLAN00; la configuración conserva solamente referencias. El navegador, el modelo y la respuesta de Alexa no reciben tokens. Una instalación comunitaria no comparte credenciales con otra. Si el producto exige un servicio OAuth confidencial, su secreto permanece en ese servicio; nunca se distribuye en el frontend, extensión o binario público.

## 4. Capacidades autorizables

Los identificadores de esta tabla son capacidades internas de AlexIA; no son nombres de scopes del proveedor.

| Capacidad propuesta | Operación | Guarda específica |
|---|---|---|
| `jira.issues.read` | Leer issues | cloudId+project permitidos |
| `jira.comments.preview` | Preparar comentario | ADF validado y sin publicación |
| `jira.transitions.apply` | Transición posterior | Consultar transiciones actuales y permiso concreto |

Las lecturas pueden preautorizarse para recursos concretos. Las escrituras o ejecuciones se sujetan a una política explícita por proyecto. Una política previa válida evita pedir confirmación repetida; si falta, preparar una vista previa concreta. La autorización queda ligada a identidad, recurso, parámetros y versión. Texto de un issue, archivo o registro no modifica permisos.

## 5. Superficie externa y contrato del adaptador

| Operación | API, comando o mecanismo | Tratamiento de la respuesta |
|---|---|---|
| Recursos OAuth | GET https://api.atlassian.com/oauth/token/accessible-resources | Elegir cloudId validado |
| Búsqueda mejorada | POST /rest/api/3/search/jql | nextPageToken y campos mínimos |
| Issue | GET /rest/api/3/issue/{issueIdOrKey} | Verificar proyecto y sitio |
| Transiciones posteriores | GET/POST .../issue/{id}/transitions | ID de transición vigente, no nombre asumido |

Procesar ADF como estructura tipada y convertir a texto seguro; no insertar HTML. Búsqueda puede tener consistencia eventual: usar updated y lectura individual para precondiciones de escritura. El scope OAuth nunca amplía permisos Jira del usuario.

**Petición interna ilustrativa:**

```json
{
  "integration_id": "jira",
  "project_id": "project-demo",
  "capability": "jira.issues.read",
  "input": {
    "cloud_id": "site-demo",
    "project_id": "10000",
    "issue_key": "DEMO-1"
  },
  "idempotency_key": "demo-jira-001"
}
```

Se envía al propuesto `POST /api/jobs`, exclusivo de la API administrativa autenticada. Respuesta: HTTP 202 con `job_id`, `status` y `status_url`. `GET /api/jobs/{job_id}` consulta progreso; `POST /api/jobs/{job_id}/cancel` solicita cancelación. Validar esquema y pertenencia antes de consultar credenciales o iniciar procesos. No existe un endpoint que acepte una URL o comando arbitrario del modelo.

**Datos específicos normalizados:** cloudId, projectId, issueId/key, updated, estado, campos permitidos y vínculo repo.

El resultado incluye `schema_version`, identidad del recurso, instante, versión de herramienta/API, alcance, evidencia saneada, advertencias y estado de cobertura. La respuesta puede terminar con hallazgos aunque el proceso haya funcionado correctamente. Diferenciar ausencia de resultados, permisos insuficientes, cobertura parcial y ejecución fallida.

## 6. Archivos y cambios previstos en AlexIA

Los archivos nuevos se crean durante la implementación. Evitar duplicar contratos del núcleo en el conector.

| Ruta relativa al repositorio | Responsabilidad |
|---|---|
| `src/integrations/jira.rs` | REST v3/OAuth |
| `src/integrations/jira_adf.rs` | ADF seguro |
| `tests/integrations_jira.rs` | Sitios y scopes |
| `ui/src/integrations/jira.js` | Sitio/proyecto/issues |

`src/lib.rs` registraría los módulos; `src/control.rs` delegaría rutas administrativas al núcleo de integraciones. `ui/src/main.js` consumiría estados y resultados; `ui/index.html`/`ui/src/components.js` presentarían la conexión. Conservar las guardas loopback actuales y actualizar [BACKEND.md](../../.agents/.docs/BACKEND.md), [UI.md](../../.agents/.docs/UI.md) y el documento operativo correspondiente al implementar.

## 7. Configuración propuesta

| Campo lógico | Tipo/protección | Uso |
|---|---|---|
| `oauth_client_id` | Público | App 3LO |
| `credential_ref` | Secreto | Tokens/broker |
| `cloud_id` | ID | Sitio seleccionado |
| `project_ids` | Lista | Proyectos |
| `field_allowlist` | Lista | Campos mínimos |

Estos campos pertenecen a un registro validado de integración, no a un conjunto de secretos nuevos en `.env`. Escribirlos desde la API local con control de revisión. Exponer solamente `configured`, permisos y errores saneados. La presencia de configuración no equivale a conexión verificada.

## 8. Implementación por entregas verificables

### 16.1 — Conectar sitio

**Trabajo:** OAuth y accessible-resources.

**Dependencias:** BC-01 a BC-05 de PLAN00; completar los prerrequisitos de este documento.

**Aceptación:**

- [ ] cloudId no se confía del prompt.
- [ ] Otro sitio rechazado.

**Verificación:** Dos sitios fixture y sandbox real.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### 16.2 — Consultar issues

**Trabajo:** Búsqueda mejorada y ADF.

**Dependencias:** Tarea 16.1 completada y contrato de salida estable.

**Aceptación:**

- [ ] Paginación correcta.
- [ ] ADF hostil no ejecuta HTML.

**Verificación:** Issue fixture con descripción estructurada.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### 16.3 — Preparar borradores y recuperación

**Trabajo:** Vista previa/refresh/desconexión.

**Dependencias:** Tarea 16.2 completada y contrato de salida estable.

**Aceptación:**

- [ ] No modifica estado al analizar.
- [ ] Transición desconocida no se inventa.

**Verificación:** Fixtures de permisos y CLI.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### Checkpoint de salida

- [ ] Primera operación real completa desde UI hasta proveedor/herramienta y resultado visible.
- [ ] Denegación comprobada al cambiar identidad, proyecto o recurso autorizado.
- [ ] Cancelación, error y desconexión dejan estados coherentes y permiten recuperación.
- [ ] No se modificó el contrato de firma, certificado, timestamp e ID de Alexa.
- [ ] La documentación distingue simulación, prueba real y funciones pendientes.

## 9. Errores, límites y recuperación

Respetar 429/Retry-After y límites por endpoint; no recorrer todos los proyectos. Tras timeout de comentario/transición futura, reconciliar antes de repetir. Error 404 no asegura inexistencia si faltan permisos.

Límites iniciales de diseño: una ejecución por proyecto, cola local acotada, cuerpo administrativo de hasta 128 KiB y resultados paginados; los artefactos grandes se entregan por identificador. Los valores se medirán antes de liberar y no describen capacidad demostrada. HTTP 401 expira la sesión; 403 no se corrige ampliando permisos silenciosamente; 429 respeta `Retry-After` cuando exista. Un timeout después de una escritura produce estado `unknown_outcome` hasta reconciliar, sin repetición ciega.

## 10. Riesgos particulares y controles

| Riesgo | Control previsto | Evidencia requerida |
|---|---|---|
| Sitio equivocado | Clave cloudId+issueId | DEMO-1 en dos sitios |
| JQL amplio | Filtros tipados y proyecto obligatorio | Consulta no sale de proyecto |
| Refresh confidencial distribuido | Broker/BYO documentado | Secret ausente de assets |

**Datos y retención:** Minimizar datos personales y custom fields; no leer adjuntos por defecto. Texto de issue es dato no confiable.

## 11. Pruebas del adaptador y del flujo completo

**Fixtures y pruebas de contrato:**

- nextPageToken y respuestas parciales.
- ADF con enlaces hostiles y custom fields desconocidos.
- Permisos de usuario insuficientes pese a OAuth válido.

**Prueba real de aceptación:** Consultar issue sandbox Jira Cloud y comparar sitio/proyecto/updated; generar borrador sin publicarlo.

**Prueba de interfaz con la skill Playwright CLI:**

1. Elegir sitio y proyecto.
2. Abrir issue y campos mínimos.
3. Ver borrador ADF representado seguro.
4. Probar desconexión y 403.

Usar sesión de navegador desechable; obtener referencias con `snapshot` y ejecutar comprobaciones con `run-code --filename=...` cuando se necesiten aserciones. Para el estado simulado, interceptar únicamente la API propuesta e identificar el resultado como prueba de UI. La prueba real debe comprobar el recurso externo o artefacto local. No automatizar pantallas de login ni guardar cookies del usuario para sustituir APIs oficiales.

Comandos de validación cuando se implemente: `cargo test --locked --test integrations_jira` para el archivo propuesto, `cargo check --locked --all-targets`, las pruebas de UI afectadas y `npm --prefix ui run build`. Estos comandos son objetivos; no se afirma que esos tests existan hoy. Añadir pruebas de esquema, autorización y recuperación antes de habilitar capacidades de escritura.

## 12. Despliegue, reversión y criterio de finalización

Habilitar primero para una instalación piloto mediante flag de integración y solamente las capacidades verificadas. Registrar versión y permisos. Después ampliar a un segundo proyecto y comprobar separación de conexiones; no equivale a un despliegue SaaS multiusuario.

**Reversión específica:** Revocar conexión y deshabilitar adapter; no revertir estados de issues automáticamente.

Deshabilitar el conector impide nuevos trabajos; cancelar o drenar los existentes según su efecto. Revocar credenciales cuando corresponda. Desconectar no borra recursos ajenos ni deshace efectos externos de forma automática. La reversión del software y la compensación de una operación externa son procedimientos distintos.

**Se considera completada cuando:** Issue real de sitio correcto se consulta y representa con permisos/ciclo OAuth verificados.

**Decisiones o condiciones pendientes:** Modelo de broker 3LO y scopes finales por operación; Data Center queda fuera.

## 13. Fuentes oficiales y revalidación

Consultadas el 2026-11-03. Sustentan los mecanismos externos; el diseño de AlexIA es una propuesta propia. Revalidar scopes, rutas, versión, licencia y condiciones de distribución antes de implementar. Los enlaces de contratos específicos prevalecen sobre ejemplos generales; una página accesible no demuestra que una cuenta tenga habilitada la función.

- [Jira OAuth 3LO](https://developer.atlassian.com/cloud/jira/platform/oauth-2-3lo-apps/) — Autenticación y recursos
- [Búsqueda v3](https://developer.atlassian.com/cloud/jira/platform/rest/v3/api-group-issue-search/) — search/jql y paginación
- [REST v3](https://developer.atlassian.com/cloud/jira/platform/rest/v3/intro/) — ADF y permisos
