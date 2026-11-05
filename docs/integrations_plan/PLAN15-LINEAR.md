# PLAN15 — Integración de Linear con AlexIA

Fecha: 2026-11-03. Estado: **plan propuesto; integración pendiente**. Prioridad: P2, tareas técnicas.
Audiencia: comunidad técnica, inicialmente instalaciones personales con proyectos autorizados.

## 1. Resultado y alcance

Relacionar issues y proyectos de Linear con repositorios y tareas de AlexIA, conservando equipo y organización correctos.

**Primer recorrido entregable:** Seleccionar un issue, reunir contexto y preparar una propuesta de trabajo; después publicar una actualización revisada.

**Alcance inicial:**

- OAuth de lectura, equipos/proyectos permitidos e issues.
- Enlaces entre issue_id y repository_id sin inferencias por nombre.
- Borrador de comentario local; publicación posterior con permiso propio.

**Ampliaciones posteriores:** Mutaciones y webhooks de issues con firma, deduplicación y bucles controlados.

**Fuera de esta primera entrega:** Administrar organización, cerrar issues por una respuesta del modelo o suscribirse a todos los eventos.

## 2. Punto de partida real y prerrequisitos

No existe cliente GraphQL ni modelo de issue externo. El proyecto debe introducir ExternalIssueRef con provider, workspace y ID.

La interfaz actual administra configuración; no existe todavía el catálogo ejecutable de conectores ni la API de trabajos descrita aquí. El estado actual y las dependencias transversales están en [PLAN00 — Base común](PLAN00-BASE-COMUN.md). Las rutas, tipos, módulos y opciones nuevos que siguen son propuestas de implementación, no contratos existentes.

**Prerrequisitos específicos:**

- Aplicación OAuth registrada y workspace fixture.
- Cliente GraphQL con operaciones fijas y variables tipadas.
- Modelo local de vínculo issue/repositorio.

**Dependencias entre planes:** PLAN00; PLAN01/09 para flujos que también consulten código.

## 3. Decisión técnica y flujo de conexión

Consumir GraphQL mediante POST https://api.linear.app/graphql con queries propias y variables, sin aceptar consultas libres del modelo. Verificar errores GraphQL aunque HTTP sea 200. OAuth con PKCE cuando corresponda al cliente público; custodiar client secret solo en componente confidencial. La documentación vigente utiliza refresh tokens: guardar/rotar de forma atómica.

**Procedimiento de conexión o habilitación:**

1. Registrar OAuth y redirect exacto; elegir modelo de aplicación local/confidencial.
2. Iniciar state/PKCE y solicitar read.
3. Consultar viewer/organización/equipos y permitir selección.
4. Leer un issue por ID y vincularlo explícitamente a un repositorio.
5. Preparar comentario sin publicarlo; mostrar destino y contenido.
6. Añadir escritura o webhooks solo tras entregar lectura y revocación.

**Autenticación, renovación y revocación:** Access/refresh por workspace y usuario, expiración conforme respuesta; no asumir token permanente. Revocación invalida conexiones. Guardar nuevo refresh antes de invalidar el anterior y serializar renovaciones.

**Permisos externos y selección de recursos:** OAuth read inicialmente; write solo para mutaciones elegidas. No solicitar admin para leer issues. Los permisos del token no sustituyen la selección interna de equipos.

Las credenciales residen en el almacén del sistema descrito en PLAN00; la configuración conserva solamente referencias. El navegador, el modelo y la respuesta de Alexa no reciben tokens. Una instalación comunitaria no comparte credenciales con otra. Si el producto exige un servicio OAuth confidencial, su secreto permanece en ese servicio; nunca se distribuye en el frontend, extensión o binario público.

## 4. Capacidades autorizables

Los identificadores de esta tabla son capacidades internas de AlexIA; no son nombres de scopes del proveedor.

| Capacidad propuesta | Operación | Guarda específica |
|---|---|---|
| `linear.issues.read` | Leer issue y contexto | Workspace/equipo permitido |
| `linear.comments.preview` | Preparar actualización | Sin efecto externo |
| `linear.comments.publish` | Publicar en fase posterior | Issue/versión/contenido autorizados |

Las lecturas pueden preautorizarse para recursos concretos. Las escrituras o ejecuciones se sujetan a una política explícita por proyecto. Una política previa válida evita pedir confirmación repetida; si falta, preparar una vista previa concreta. La autorización queda ligada a identidad, recurso, parámetros y versión. Texto de un issue, archivo o registro no modifica permisos.

## 5. Superficie externa y contrato del adaptador

| Operación | API, comando o mecanismo | Tratamiento de la respuesta |
|---|---|---|
| GraphQL | POST https://api.linear.app/graphql | Variables y pageInfo/endCursor |
| OAuth | /oauth/authorize y api.linear.app/oauth/token | State/PKCE y expiración |
| Webhook futuro | POST /webhooks/linear propuesto | Linear-Signature HMAC y timestamp firmado |

Operaciones GraphQL de lectura y mutación se registran por nombre y esquema. No interpolar título de issue en GraphQL. En webhook validar HMAC sobre bytes originales, frescura de webhookTimestamp del cuerpo y Linear-Delivery antes de encolar. Usar comparación constante. Eventos originados por AlexIA no deben disparar el mismo flujo indefinidamente.

**Petición interna ilustrativa:**

```json
{
  "integration_id": "linear",
  "project_id": "project-demo",
  "capability": "linear.issues.read",
  "input": {
    "workspace_id": "linear-demo",
    "issue_id": "issue-demo"
  },
  "idempotency_key": "demo-linear-001"
}
```

Se envía al propuesto `POST /api/jobs`, exclusivo de la API administrativa autenticada. Respuesta: HTTP 202 con `job_id`, `status` y `status_url`. `GET /api/jobs/{job_id}` consulta progreso; `POST /api/jobs/{job_id}/cancel` solicita cancelación. Validar esquema y pertenencia antes de consultar credenciales o iniciar procesos. No existe un endpoint que acepte una URL o comando arbitrario del modelo.

**Datos específicos normalizados:** workspace/team/project/issue IDs, identifier humano, updatedAt, estado, vínculos y origen de cambios.

El resultado incluye `schema_version`, identidad del recurso, instante, versión de herramienta/API, alcance, evidencia saneada, advertencias y estado de cobertura. La respuesta puede terminar con hallazgos aunque el proceso haya funcionado correctamente. Diferenciar ausencia de resultados, permisos insuficientes, cobertura parcial y ejecución fallida.

## 6. Archivos y cambios previstos en AlexIA

Los archivos nuevos se crean durante la implementación. Evitar duplicar contratos del núcleo en el conector.

| Ruta relativa al repositorio | Responsabilidad |
|---|---|
| `src/integrations/linear.rs` | OAuth y operaciones GraphQL |
| `src/integrations/linear_queries/` | Queries revisadas |
| `tests/integrations_linear.rs` | Errores GraphQL y scopes |
| `ui/src/integrations/linear.js` | Selección y borradores |

`src/lib.rs` registraría los módulos; `src/control.rs` delegaría rutas administrativas al núcleo de integraciones. `ui/src/main.js` consumiría estados y resultados; `ui/index.html`/`ui/src/components.js` presentarían la conexión. Conservar las guardas loopback actuales y actualizar [BACKEND.md](../../.agents/.docs/BACKEND.md), [UI.md](../../.agents/.docs/UI.md) y el documento operativo correspondiente al implementar.

## 7. Configuración propuesta

| Campo lógico | Tipo/protección | Uso |
|---|---|---|
| `oauth_client_id` | Público | App registrada |
| `credential_ref` | Secreto | Access/refresh |
| `workspace_id` | ID | Organización validada |
| `team_ids` | Lista | Equipos permitidos |
| `webhook_secret_ref` | Secreto opcional | Solo webhook posterior |

Estos campos pertenecen a un registro validado de integración, no a un conjunto de secretos nuevos en `.env`. Escribirlos desde la API local con control de revisión. Exponer solamente `configured`, permisos y errores saneados. La presencia de configuración no equivale a conexión verificada.

## 8. Implementación por entregas verificables

### 15.1 — Conectar y leer issue

**Trabajo:** OAuth, viewer y equipos.

**Dependencias:** BC-01 a BC-05 de PLAN00; completar los prerrequisitos de este documento.

**Aceptación:**

- [ ] Token de otro workspace no cruza recursos.
- [ ] HTTP 200 con errors se reporta.

**Verificación:** Mocks GraphQL y issue real fixture.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### 15.2 — Vincular y preparar actualización

**Trabajo:** Elegir repositorio y construir borrador.

**Dependencias:** Tarea 15.1 completada y contrato de salida estable.

**Aceptación:**

- [ ] Vínculo se confirma por ID.
- [ ] No publica al generar borrador.

**Verificación:** Dos issues con mismo título.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### 15.3 — Renovar/revocar y probar UI

**Trabajo:** Completar ciclo de conexión.

**Dependencias:** Tarea 15.2 completada y contrato de salida estable.

**Aceptación:**

- [ ] Refresh simultáneo se serializa.
- [ ] Revocación corta acceso.

**Verificación:** Expiración fixture y recorrido CLI.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### Checkpoint de salida

- [ ] Primera operación real completa desde UI hasta proveedor/herramienta y resultado visible.
- [ ] Denegación comprobada al cambiar identidad, proyecto o recurso autorizado.
- [ ] Cancelación, error y desconexión dejan estados coherentes y permiten recuperación.
- [ ] No se modificó el contrato de firma, certificado, timestamp e ID de Alexa.
- [ ] La documentación distingue simulación, prueba real y funciones pendientes.

## 9. Errores, límites y recuperación

Paginación con cursor y límite; proponer 50 issues por página y techo por consulta. Respetar límites/costes GraphQL. Tras timeout de mutación futura, buscar marcador propio antes de repetir.

Límites iniciales de diseño: una ejecución por proyecto, cola local acotada, cuerpo administrativo de hasta 128 KiB y resultados paginados; los artefactos grandes se entregan por identificador. Los valores se medirán antes de liberar y no describen capacidad demostrada. HTTP 401 expira la sesión; 403 no se corrige ampliando permisos silenciosamente; 429 respeta `Retry-After` cuando exista. Un timeout después de una escritura produce estado `unknown_outcome` hasta reconciliar, sin repetición ciega.

## 10. Riesgos particulares y controles

| Riesgo | Control previsto | Evidencia requerida |
|---|---|---|
| Errores GraphQL ignorados | Revisar errors y data parcial | Fixture HTTP200/errors |
| Bucles de eventos | Origen/idempotencia por issue+evento | Evento propio no se repite |
| Comentarios duplicados | Reconciliar marcador y updatedAt | Timeout no duplica |

**Datos y retención:** Issues y comentarios pueden incluir incidentes o secretos. Seleccionar campos, redacción y TTL; no indexar todo el workspace.

## 11. Pruebas del adaptador y del flujo completo

**Fixtures y pruebas de contrato:**

- Scopes read/write y equipos privados.
- GraphQL partial data, cursor inválido y rate limit.
- Webhook repetido o timestamp alterado en fase posterior.

**Prueba real de aceptación:** Leer issue fixture real y verificar equipo/updatedAt; generar borrador local y revocar. Escritura real solo si esa fase se implementa y autoriza.

**Prueba de interfaz con la skill Playwright CLI:**

1. Conectar y elegir equipo.
2. Abrir issue y vincular repo.
3. Ver borrador y destino.
4. Desconectar y verificar pérdida de acceso.

Usar sesión de navegador desechable; obtener referencias con `snapshot` y ejecutar comprobaciones con `run-code --filename=...` cuando se necesiten aserciones. Para el estado simulado, interceptar únicamente la API propuesta e identificar el resultado como prueba de UI. La prueba real debe comprobar el recurso externo o artefacto local. No automatizar pantallas de login ni guardar cookies del usuario para sustituir APIs oficiales.

Comandos de validación cuando se implemente: `cargo test --locked --test integrations_linear` para el archivo propuesto, `cargo check --locked --all-targets`, las pruebas de UI afectadas y `npm --prefix ui run build`. Estos comandos son objetivos; no se afirma que esos tests existan hoy. Añadir pruebas de esquema, autorización y recuperación antes de habilitar capacidades de escritura.

## 12. Despliegue, reversión y criterio de finalización

Habilitar primero para una instalación piloto mediante flag de integración y solamente las capacidades verificadas. Registrar versión y permisos. Después ampliar a un segundo proyecto y comprobar separación de conexiones; no equivale a un despliegue SaaS multiusuario.

**Reversión específica:** Deshabilitar conector y revocar OAuth; retirar solo webhooks creados por AlexIA con ownership probado. No cerrar issues ni borrar comentarios ajenos.

Deshabilitar el conector impide nuevos trabajos; cancelar o drenar los existentes según su efecto. Revocar credenciales cuando corresponda. Desconectar no borra recursos ajenos ni deshace efectos externos de forma automática. La reversión del software y la compensación de una operación externa son procedimientos distintos.

**Se considera completada cuando:** Lectura real aislada, vínculo por IDs y borrador sin efecto; renovación/revocación y errores GraphQL probados.

**Decisiones o condiciones pendientes:** Broker OAuth comunitario y distribución pública; revisar políticas y scopes antes de publicación.

## 13. Fuentes oficiales y revalidación

Consultadas el 2026-11-03. Sustentan los mecanismos externos; el diseño de AlexIA es una propuesta propia. Revalidar scopes, rutas, versión, licencia y condiciones de distribución antes de implementar. Los enlaces de contratos específicos prevalecen sobre ejemplos generales; una página accesible no demuestra que una cuenta tenga habilitada la función.

- [OAuth Linear](https://linear.app/developers/oauth-2-0-authentication) — PKCE/refresh/scopes
- [Webhooks Linear](https://linear.app/developers/webhooks) — Firma, frescura y delivery
- [GraphQL y SDK](https://linear.app/developers) — Operaciones y paginación
