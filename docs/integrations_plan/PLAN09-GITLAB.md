# PLAN09 — Integración de GitLab con AlexIA

Fecha: 2026-11-03. Estado: **plan propuesto; integración pendiente**. Prioridad: P2, alternativa de repositorios.
Audiencia: comunidad técnica, inicialmente instalaciones personales con proyectos autorizados.

## 1. Resultado y alcance

Ofrecer lectura de proyectos, merge requests y pipelines para usuarios de GitLab sin convertir GitHub en requisito.

**Primer recorrido entregable:** Autorizar una instancia GitLab, elegir un proyecto y revisar un MR con SHA y estado de pipeline.

**Alcance inicial:**

- GitLab.com inicialmente; Self-Managed mediante instancia registrada.
- OAuth con PKCE cuando el modo de distribución lo permita.
- Lectura de MR/diff y pipeline, sin escribir variables o hooks.

**Ampliaciones posteriores:** Comentarios y borradores de MR con permisos explícitos; webhooks verificados.

**Fuera de esta primera entrega:** Administración de grupos, runners, usuarios o variables CI.

## 2. Punto de partida real y prerrequisitos

No hay cliente GitLab. El futuro modelo RepositoryRef debe incluir proveedor e instancia y evitar asumir owner/repo de GitHub.

La interfaz actual administra configuración; no existe todavía el catálogo ejecutable de conectores ni la API de trabajos descrita aquí. El estado actual y las dependencias transversales están en [PLAN00 — Base común](PLAN00-BASE-COMUN.md). Las rutas, tipos, módulos y opciones nuevos que siguen son propuestas de implementación, no contratos existentes.

**Prerrequisitos específicos:**

- Núcleo de conexión y cliente HTTPS.
- Proyecto fixture y aplicación OAuth registrada en la instancia.
- Política de hosts y CA para Self-Managed sin desactivar TLS.

**Dependencias entre planes:** PLAN00; reutilizar conceptos de PLAN01 sin reutilizar sus tokens o nombres de scopes.

## 3. Decisión técnica y flujo de conexión

Adaptador REST v4 con base URL validada y project_id numérico estable. La identidad incluye host/instancia, pues IDs pueden repetirse en servidores distintos. Para clientes públicos usar authorization code con PKCE conforme al soporte de GitLab; si hay componente confidencial, custodiar secret en servicio. La instancia Self-Managed puede estar en red privada solo por registro explícito del propietario; no permitir SSRF desde una pregunta.

**Procedimiento de conexión o habilitación:**

1. Registrar origen GitLab y comprobar TLS/version/capacidades.
2. Configurar aplicación OAuth y redirect exacto; generar state y verifier PKCE con TTL.
3. Autorizar lectura y canjear código en componente apropiado.
4. Consultar usuario y proyectos, y guardar selección explícita.
5. Leer MR y pipeline fijando diff refs/head SHA.
6. Desconectar y revocar token; informar pérdida de permisos de grupo.

**Autenticación, renovación y revocación:** Tokens de acceso y refresh por conexión e instancia; renovación atómica y revocación mediante OAuth. No copiar cookies de sesión web.

**Permisos externos y selección de recursos:** Comenzar con read_api para REST de lectura; read_repository solo si se necesita Git HTTP. La escritura API suele exigir api y por ello se añade en fase separada tras revisar alcance real.

Las credenciales residen en el almacén del sistema descrito en PLAN00; la configuración conserva solamente referencias. El navegador, el modelo y la respuesta de Alexa no reciben tokens. Una instalación comunitaria no comparte credenciales con otra. Si el producto exige un servicio OAuth confidencial, su secreto permanece en ese servicio; nunca se distribuye en el frontend, extensión o binario público.

## 4. Capacidades autorizables

Los identificadores de esta tabla son capacidades internas de AlexIA; no son nombres de scopes del proveedor.

| Capacidad propuesta | Operación | Guarda específica |
|---|---|---|
| `gitlab.projects.read` | Leer proyectos | Host e IDs permitidos |
| `gitlab.merge_requests.read` | Leer MR/diff | Head SHA fijado |
| `gitlab.pipelines.read` | Consultar pipeline | Proyecto e instancia correctos |

Las lecturas pueden preautorizarse para recursos concretos. Las escrituras o ejecuciones se sujetan a una política explícita por proyecto. Una política previa válida evita pedir confirmación repetida; si falta, preparar una vista previa concreta. La autorización queda ligada a identidad, recurso, parámetros y versión. Texto de un issue, archivo o registro no modifica permisos.

## 5. Superficie externa y contrato del adaptador

| Operación | API, comando o mecanismo | Tratamiento de la respuesta |
|---|---|---|
| Identidad | GET /api/v4/user | ID y host |
| Proyectos | GET /api/v4/projects?membership=true | Paginar y filtrar selección |
| MR | GET /api/v4/projects/{id}/merge_requests/{iid} | No confundir iid con id global |
| Diff y pipelines | GET /api/v4/projects/{id}/merge_requests/{iid}/diffs y GET /api/v4/projects/{id}/pipelines | Comprobar soporte de versión y límites |

Codificar paths con librería URL; preferir project_id a path de namespace. Pagination y rate limits dependen de endpoint/instancia. No asumir mismos webhooks, firma o scopes de GitHub. Si se añaden hooks, verificar mecanismo documentado de la versión y secreto por conexión antes de encolar.

**Petición interna ilustrativa:**

```json
{
  "integration_id": "gitlab",
  "project_id": "project-demo",
  "capability": "gitlab.merge_requests.read",
  "input": {
    "instance_id": "gitlab-com",
    "project_external_id": 42,
    "merge_request_iid": 7
  },
  "idempotency_key": "demo-gitlab-001"
}
```

Se envía al propuesto `POST /api/jobs`, exclusivo de la API administrativa autenticada. Respuesta: HTTP 202 con `job_id`, `status` y `status_url`. `GET /api/jobs/{job_id}` consulta progreso; `POST /api/jobs/{job_id}/cancel` solicita cancelación. Validar esquema y pertenencia antes de consultar credenciales o iniciar procesos. No existe un endpoint que acepte una URL o comando arbitrario del modelo.

**Datos específicos normalizados:** instance_id, project_id, MR iid/id, base/head/start SHA, pipeline_id y coverage de diff.

El resultado incluye `schema_version`, identidad del recurso, instante, versión de herramienta/API, alcance, evidencia saneada, advertencias y estado de cobertura. La respuesta puede terminar con hallazgos aunque el proceso haya funcionado correctamente. Diferenciar ausencia de resultados, permisos insuficientes, cobertura parcial y ejecución fallida.

## 6. Archivos y cambios previstos en AlexIA

Los archivos nuevos se crean durante la implementación. Evitar duplicar contratos del núcleo en el conector.

| Ruta relativa al repositorio | Responsabilidad |
|---|---|
| `src/integrations/gitlab/client.rs` | HTTP y OAuth |
| `src/integrations/gitlab/mod.rs` | Modelos y capacidades |
| `tests/integrations_gitlab.rs` | Host, MR y permisos |
| `ui/src/integrations/gitlab.js` | Instancia y selección |

`src/lib.rs` registraría los módulos; `src/control.rs` delegaría rutas administrativas al núcleo de integraciones. `ui/src/main.js` consumiría estados y resultados; `ui/index.html`/`ui/src/components.js` presentarían la conexión. Conservar las guardas loopback actuales y actualizar [BACKEND.md](../../.agents/.docs/BACKEND.md), [UI.md](../../.agents/.docs/UI.md) y el documento operativo correspondiente al implementar.

## 7. Configuración propuesta

| Campo lógico | Tipo/protección | Uso |
|---|---|---|
| `instance_origin` | URL validada | GitLab.com o host explícito |
| `oauth_client_id` | Público | Aplicación de instancia |
| `oauth_credential_ref` | Referencia secreta | Tokens |
| `project_ids` | Lista | Recursos permitidos |

Estos campos pertenecen a un registro validado de integración, no a un conjunto de secretos nuevos en `.env`. Escribirlos desde la API local con control de revisión. Exponer solamente `configured`, permisos y errores saneados. La presencia de configuración no equivale a conexión verificada.

## 8. Implementación por entregas verificables

### 09.1 — Conectar instancia

**Trabajo:** OAuth y lectura de identidad.

**Dependencias:** BC-01 a BC-05 de PLAN00; completar los prerrequisitos de este documento.

**Aceptación:**

- [ ] State/PKCE inválidos rechazados.
- [ ] TLS inválido no se omite.

**Verificación:** Mock OAuth y conexión de prueba.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### 09.2 — Leer MR

**Trabajo:** Normalizar diff refs y pipeline.

**Dependencias:** Tarea 09.1 completada y contrato de salida estable.

**Aceptación:**

- [ ] iid correcto por proyecto.
- [ ] Truncamiento visible.

**Verificación:** MR real y fixture con diff incompleto.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### 09.3 — Aislar y revocar

**Trabajo:** Validar host/IDs y renovación.

**Dependencias:** Tarea 09.2 completada y contrato de salida estable.

**Aceptación:**

- [ ] ID igual en otro host no cruza datos.
- [ ] Revocar invalida trabajos nuevos.

**Verificación:** Dos conexiones fixture y UI CLI.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### Checkpoint de salida

- [ ] Primera operación real completa desde UI hasta proveedor/herramienta y resultado visible.
- [ ] Denegación comprobada al cambiar identidad, proyecto o recurso autorizado.
- [ ] Cancelación, error y desconexión dejan estados coherentes y permiten recuperación.
- [ ] No se modificó el contrato de firma, certificado, timestamp e ID de Alexa.
- [ ] La documentación distingue simulación, prueba real y funciones pendientes.

## 9. Errores, límites y recuperación

Timeout propuesto 10 s/HTTP y 60 s/MR, con páginas limitadas. 404 puede representar falta de permiso: no afirmar borrado sin evidencia. No cambiar a PAT silenciosamente si OAuth falla.

Límites iniciales de diseño: una ejecución por proyecto, cola local acotada, cuerpo administrativo de hasta 128 KiB y resultados paginados; los artefactos grandes se entregan por identificador. Los valores se medirán antes de liberar y no describen capacidad demostrada. HTTP 401 expira la sesión; 403 no se corrige ampliando permisos silenciosamente; 429 respeta `Retry-After` cuando exista. Un timeout después de una escritura produce estado `unknown_outcome` hasta reconciliar, sin repetición ciega.

## 10. Riesgos particulares y controles

| Riesgo | Control previsto | Evidencia requerida |
|---|---|---|
| SSRF Self-Managed | Origen registrado, CA y DNS controlados | Host externo del prompt rechazado |
| Confusión de IDs | Clave host+project_id | Fixtures con IDs repetidos |
| Scope demasiado amplio | Lectura inicial y política interna | Escritura rechazada |

**Datos y retención:** Diff privado se selecciona y sanea antes del modelo; token por instancia y TTL de resultados.

## 11. Pruebas del adaptador y del flujo completo

**Fixtures y pruebas de contrato:**

- Paginación, 429 y token renovado.
- MR iid repetido en otro proyecto.
- Certificado inválido, redirect externo y diff overflow.

**Prueba real de aceptación:** Leer MR de proyecto fixture en GitLab.com, comparar head SHA y pipeline, y revocar acceso.

**Prueba de interfaz con la skill Playwright CLI:**

1. Conectar instancia y proyecto.
2. Consultar MR y ver cobertura.
3. Mostrar acceso revocado.
4. Comprobar selector distingue instancias.

Usar sesión de navegador desechable; obtener referencias con `snapshot` y ejecutar comprobaciones con `run-code --filename=...` cuando se necesiten aserciones. Para el estado simulado, interceptar únicamente la API propuesta e identificar el resultado como prueba de UI. La prueba real debe comprobar el recurso externo o artefacto local. No automatizar pantallas de login ni guardar cookies del usuario para sustituir APIs oficiales.

Comandos de validación cuando se implemente: `cargo test --locked --test integrations_gitlab` para el archivo propuesto, `cargo check --locked --all-targets`, las pruebas de UI afectadas y `npm --prefix ui run build`. Estos comandos son objetivos; no se afirma que esos tests existan hoy. Añadir pruebas de esquema, autorización y recuperación antes de habilitar capacidades de escritura.

## 12. Despliegue, reversión y criterio de finalización

Habilitar primero para una instalación piloto mediante flag de integración y solamente las capacidades verificadas. Registrar versión y permisos. Después ampliar a un segundo proyecto y comprobar separación de conexiones; no equivale a un despliegue SaaS multiusuario.

**Reversión específica:** Desactivar cliente, revocar conexión y detener polling; no borrar hooks o recursos no creados por AlexIA.

Deshabilitar el conector impide nuevos trabajos; cancelar o drenar los existentes según su efecto. Revocar credenciales cuando corresponda. Desconectar no borra recursos ajenos ni deshace efectos externos de forma automática. La reversión del software y la compensación de una operación externa son procedimientos distintos.

**Se considera completada cuando:** MR real y pipeline se leen con identidad correcta y revocación comprobada.

**Decisiones o condiciones pendientes:** Versiones y hosts Self-Managed soportados se fijan después del piloto GitLab.com.

## 13. Fuentes oficiales y revalidación

Consultadas el 2026-11-03. Sustentan los mecanismos externos; el diseño de AlexIA es una propuesta propia. Revalidar scopes, rutas, versión, licencia y condiciones de distribución antes de implementar. Los enlaces de contratos específicos prevalecen sobre ejemplos generales; una página accesible no demuestra que una cuenta tenga habilitada la función.

- [OAuth GitLab](https://docs.gitlab.com/api/oauth2/) — PKCE y renovación
- [Projects API](https://docs.gitlab.com/api/projects/) — Selección y IDs
- [REST API](https://docs.gitlab.com/api/rest/) — Versionado y paginación
