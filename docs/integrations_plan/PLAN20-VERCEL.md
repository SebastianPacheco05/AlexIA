# PLAN20 — Integración de Vercel con AlexIA

Fecha: 2026-11-03. Estado: **plan propuesto; integración pendiente**. Prioridad: P3, diagnóstico de despliegues.
Audiencia: comunidad técnica, inicialmente instalaciones personales con proyectos autorizados.

## 1. Resultado y alcance

Consultar despliegues y fallos de build; añadir previews autorizados después, manteniendo producción como capacidad separada.

**Primer recorrido entregable:** Elegir proyecto/equipo, ver deployment del SHA revisado y explicar su estado o error de build.

**Alcance inicial:**

- Lectura de proyectos/deployments permitidos.
- Asociación con equipo, proyecto, entorno y commit.
- Logs saneados y enlace a preview existente.

**Ampliaciones posteriores:** Crear previews y cancelarlos; promoción/rollback de producción como proyecto posterior con controles propios.

**Fuera de esta primera entrega:** Leer valores de variables secretas, cambiar dominios/DNS, activar facturación o desplegar producción automáticamente.

## 2. Punto de partida real y prerrequisitos

No hay integración Vercel. El Dockerfile actual de AlexIA no demuestra compatibilidad de hosting Vercel ni es objetivo de este conector; se gestionan proyectos del usuario.

La interfaz actual administra configuración; no existe todavía el catálogo ejecutable de conectores ni la API de trabajos descrita aquí. El estado actual y las dependencias transversales están en [PLAN00 — Base común](PLAN00-BASE-COMUN.md). Las rutas, tipos, módulos y opciones nuevos que siguen son propuestas de implementación, no contratos existentes.

**Prerrequisitos específicos:**

- Proyecto de prueba Vercel existente y token o instalación autorizada.
- Definir team/project IDs y permisos documentados del modo de autenticación.
- Revisar contratos REST vigentes y acceso a previews protegidos.

**Dependencias entre planes:** PLAN00; PLAN01 ayuda a correlacionar SHA, PLAN07 puede validar preview autorizado.

## 3. Decisión técnica y flujo de conexión

Cliente REST con endpoints/versiones fijados al implementar. Piloto BYO token restringido según capacidades reales; distribución comunitaria mediante mecanismo oficial de integración/OAuth que corresponda al caso, sin inventar scopes universales. No reutilizar el token del conector Vercel disponible en este chat dentro de AlexIA. Separar metadatos de despliegue, logs y credenciales de acceso al preview.

**Procedimiento de conexión o habilitación:**

1. Elegir modo BYO o integración y documentar alcance real del token.
2. Consultar identidad/equipo y elegir proyecto por ID.
3. Listar deployments y filtrar SHA/target.
4. Leer estado y eventos de build con límites/redacción.
5. Mostrar URL de preview y si exige autenticación; no eludir protección.
6. Añadir creación de preview solo tras validar lectura, permisos y efectos/coste.

**Autenticación, renovación y revocación:** Bearer del API o token de instalación conforme mecanismo elegido; custodiar por conexión y respetar expiración/revocación. Acceso a un deployment protegido puede requerir autorización distinta a la API de gestión.

**Permisos externos y selección de recursos:** Seleccionar equipo/proyecto y operaciones mínimas disponibles para el token. AlexIA aplica deny-by-default aunque la credencial externa sea más amplia. Producción, variables y dominios no están autorizados por lectura.

Las credenciales residen en el almacén del sistema descrito en PLAN00; la configuración conserva solamente referencias. El navegador, el modelo y la respuesta de Alexa no reciben tokens. Una instalación comunitaria no comparte credenciales con otra. Si el producto exige un servicio OAuth confidencial, su secreto permanece en ese servicio; nunca se distribuye en el frontend, extensión o binario público.

## 4. Capacidades autorizables

Los identificadores de esta tabla son capacidades internas de AlexIA; no son nombres de scopes del proveedor.

| Capacidad propuesta | Operación | Guarda específica |
|---|---|---|
| `vercel.deployments.read` | Leer deployments | Team/project autorizados |
| `vercel.build_logs.read` | Leer logs saneados | Deployment exacto |
| `vercel.previews.create` | Crear preview posteriormente | SHA/config/destino/coste autorizados |

Las lecturas pueden preautorizarse para recursos concretos. Las escrituras o ejecuciones se sujetan a una política explícita por proyecto. Una política previa válida evita pedir confirmación repetida; si falta, preparar una vista previa concreta. La autorización queda ligada a identidad, recurso, parámetros y versión. Texto de un issue, archivo o registro no modifica permisos.

## 5. Superficie externa y contrato del adaptador

| Operación | API, comando o mecanismo | Tratamiento de la respuesta |
|---|---|---|
| Catálogo | `GET https://api.vercel.com/v7/deployments?projectId={id}&teamId={team}&sha={sha}` | Paginación y URL posiblemente null mientras sube |
| Detalle | `GET /v13/deployments/{idOrUrl}?withGitRepoInfo=true&teamId={team}` | Leer gitSource si está disponible; no inferir SHA |
| Eventos de build | `GET /v3/deployments/{idOrUrl}/events` con consulta acotada | Distinguir JSON de stream y cerrar al deadline |
| Preview posterior | `POST /v13/deployments?teamId={team}` | Entrada interna preview y serialización externa probada; reconciliar efectos |

Las rutas anteriores se comprobaron en la referencia oficial el 2026-11-03; capturar su esquema y revalidarlas al implementar. El detalle documenta target null para preview, mientras producción figura como production: el enum interno preview no se serializa sin comprobar el contrato de creación. Prohibir heredar target/configuración de un deployment de producción en una operación de preview. Variables/secret values se excluyen. Un deployment READY no demuestra que los flujos de negocio funcionen; validarlos con PLAN07 si se autoriza.

**Petición interna ilustrativa:**

```json
{
  "integration_id": "vercel",
  "project_id": "project-demo",
  "capability": "vercel.deployments.read",
  "input": {
    "team_id": "team-demo",
    "project_id": "project-demo",
    "commit_sha": "demo-sha"
  },
  "idempotency_key": "demo-vercel-001"
}
```

Se envía al propuesto `POST /api/jobs`, exclusivo de la API administrativa autenticada. Respuesta: HTTP 202 con `job_id`, `status` y `status_url`. `GET /api/jobs/{job_id}` consulta progreso; `POST /api/jobs/{job_id}/cancel` solicita cancelación. Validar esquema y pertenencia antes de consultar credenciales o iniciar procesos. No existe un endpoint que acepte una URL o comando arbitrario del modelo.

**Datos específicos normalizados:** team/project/deployment IDs, target, SHA cuando exista, state, timestamps, URL validada y log coverage.

El resultado incluye `schema_version`, identidad del recurso, instante, versión de herramienta/API, alcance, evidencia saneada, advertencias y estado de cobertura. La respuesta puede terminar con hallazgos aunque el proceso haya funcionado correctamente. Diferenciar ausencia de resultados, permisos insuficientes, cobertura parcial y ejecución fallida.

## 6. Archivos y cambios previstos en AlexIA

Los archivos nuevos se crean durante la implementación. Evitar duplicar contratos del núcleo en el conector.

| Ruta relativa al repositorio | Responsabilidad |
|---|---|
| `src/integrations/vercel.rs` | REST y selección |
| `src/integrations/vercel_logs.rs` | Eventos saneados |
| `tests/integrations_vercel.rs` | Entornos/permisos |
| `ui/src/integrations/vercel.js` | Deployments y previews |

`src/lib.rs` registraría los módulos; `src/control.rs` delegaría rutas administrativas al núcleo de integraciones. `ui/src/main.js` consumiría estados y resultados; `ui/index.html`/`ui/src/components.js` presentarían la conexión. Conservar las guardas loopback actuales y actualizar [BACKEND.md](../../.agents/.docs/BACKEND.md), [UI.md](../../.agents/.docs/UI.md) y el documento operativo correspondiente al implementar.

## 7. Configuración propuesta

| Campo lógico | Tipo/protección | Uso |
|---|---|---|
| `credential_ref` | Secreto | Token/instalación |
| `team_id` | ID | Equipo elegido |
| `project_ids` | Lista | Proyectos |
| `allowed_targets` | Lista | Lectura; preview futuro |
| `api_contract_version` | Versión | Rutas/esquema probados |

Estos campos pertenecen a un registro validado de integración, no a un conjunto de secretos nuevos en `.env`. Escribirlos desde la API local con control de revisión. Exponer solamente `configured`, permisos y errores saneados. La presencia de configuración no equivale a conexión verificada.

## 8. Implementación por entregas verificables

### 20.1 — Conectar y listar

**Trabajo:** Comprobar equipo/proyecto y contratos REST.

**Dependencias:** BC-01 a BC-05 de PLAN00; completar los prerrequisitos de este documento.

**Aceptación:**

- [ ] Proyecto ajeno rechazado.
- [ ] No se leen variables secretas.

**Verificación:** Fixtures y proyecto real de lectura.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### 20.2 — Diagnosticar build

**Trabajo:** Consultar estado/eventos y SHA.

**Dependencias:** Tarea 20.1 completada y contrato de salida estable.

**Aceptación:**

- [ ] Logs saneados.
- [ ] READY se distingue de prueba funcional.

**Verificación:** Deployment fixture fallido y real autorizado.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### 20.3 — UI y diseño de preview

**Trabajo:** Completar revocación y vista previa sin crear recursos.

**Dependencias:** Tarea 20.2 completada y contrato de salida estable.

**Aceptación:**

- [ ] Producción fuera de capacidad.
- [ ] Token revocado no genera fallback.

**Verificación:** CLI y estados ERROR/CANCELED/READY.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### Checkpoint de salida

- [ ] Primera operación real completa desde UI hasta proveedor/herramienta y resultado visible.
- [ ] Denegación comprobada al cambiar identidad, proyecto o recurso autorizado.
- [ ] Cancelación, error y desconexión dejan estados coherentes y permiten recuperación.
- [ ] No se modificó el contrato de firma, certificado, timestamp e ID de Alexa.
- [ ] La documentación distingue simulación, prueba real y funciones pendientes.

## 9. Errores, límites y recuperación

Polling solo de deployment activo, con backoff y fin terminal. Proponer 60 s por diagnóstico; no esperar build completo dentro de Alexa. Una creación futura con timeout queda unknown_outcome hasta buscar deployment por correlación.

Límites iniciales de diseño: una ejecución por proyecto, cola local acotada, cuerpo administrativo de hasta 128 KiB y resultados paginados; los artefactos grandes se entregan por identificador. Los valores se medirán antes de liberar y no describen capacidad demostrada. HTTP 401 expira la sesión; 403 no se corrige ampliando permisos silenciosamente; 429 respeta `Retry-After` cuando exista. Un timeout después de una escritura produce estado `unknown_outcome` hasta reconciliar, sin repetición ciega.

## 10. Riesgos particulares y controles

| Riesgo | Control previsto | Evidencia requerida |
|---|---|---|
| Despliegue a producción por defecto | Target tipado y producción deshabilitada | Caso sin target rechazado |
| Token amplio | Selección interna y scopes reales | Prueba de proyecto ajeno |
| Preview protegido | Autorización oficial separada | 401/403 visible sin bypass |

**Datos y retención:** Build logs pueden revelar variables o rutas: redacción y retención corta. No listar valores de environment variables ni compartir URLs privadas en canales sin permiso.

## 11. Pruebas del adaptador y del flujo completo

**Fixtures y pruebas de contrato:**

- IDs de equipo/proyecto mezclados.
- Estados pendientes/READY/error y logs truncados.
- 403 de preview protegido versus 403 de API y token revocado.

**Prueba real de aceptación:** Leer deployment existente de proyecto de prueba y contrastar SHA, entorno y estado con consola oficial. Esta fase no crea despliegues.

**Prueba de interfaz con la skill Playwright CLI:**

1. Elegir equipo/proyecto.
2. Consultar deployment y entorno.
3. Ver logs saneados y preview protegido.
4. Comprobar ausencia de acción producción con permisos iniciales.

Usar sesión de navegador desechable; obtener referencias con `snapshot` y ejecutar comprobaciones con `run-code --filename=...` cuando se necesiten aserciones. Para el estado simulado, interceptar únicamente la API propuesta e identificar el resultado como prueba de UI. La prueba real debe comprobar el recurso externo o artefacto local. No automatizar pantallas de login ni guardar cookies del usuario para sustituir APIs oficiales.

Comandos de validación cuando se implemente: `cargo test --locked --test integrations_vercel` para el archivo propuesto, `cargo check --locked --all-targets`, las pruebas de UI afectadas y `npm --prefix ui run build`. Estos comandos son objetivos; no se afirma que esos tests existan hoy. Añadir pruebas de esquema, autorización y recuperación antes de habilitar capacidades de escritura.

## 12. Despliegue, reversión y criterio de finalización

Habilitar primero para una instalación piloto mediante flag de integración y solamente las capacidades verificadas. Registrar versión y permisos. Después ampliar a un segundo proyecto y comprobar separación de conexiones; no equivale a un despliegue SaaS multiusuario.

**Reversión específica:** Deshabilitar polling y revocar conexión. No borrar deployments o cambiar producción al desconectar.

Deshabilitar el conector impide nuevos trabajos; cancelar o drenar los existentes según su efecto. Revocar credenciales cuando corresponda. Desconectar no borra recursos ajenos ni deshace efectos externos de forma automática. La reversión del software y la compensación de una operación externa son procedimientos distintos.

**Se considera completada cuando:** Lectura real y diagnóstico funcionan con identidad/entorno correctos; no hay cambios de infraestructura ni acceso a secretos.

**Decisiones o condiciones pendientes:** Mecanismo oficial de distribución, contratos exactos REST y presupuesto antes de habilitar creación de previews.

## 13. Fuentes oficiales y revalidación

Consultadas el 2026-11-03. Sustentan los mecanismos externos; el diseño de AlexIA es una propuesta propia. Revalidar scopes, rutas, versión, licencia y condiciones de distribución antes de implementar. Los enlaces de contratos específicos prevalecen sobre ejemplos generales; una página accesible no demuestra que una cuenta tenga habilitada la función.

- [REST API Vercel](https://vercel.com/docs/rest-api) — Contratos oficiales a fijar
- [Crear integración](https://vercel.com/docs/integrations/create-integration) — Distribución y autenticación

- [Listado de deployments](https://vercel.com/docs/rest-api/deployments/list-deployments) — v7, filtros y paginación.
- [Detalle de deployment](https://vercel.com/docs/rest-api/deployments/get-a-deployment-by-id-or-url) — v13 y gitSource.
- [Eventos de build](https://vercel.com/docs/rest-api/deployments/get-deployment-events) — v3 y stream/JSON.
- [Creación posterior](https://vercel.com/docs/rest-api/deployments/create-a-new-deployment) — v13 y efectos.
