# PLAN00 — Base común e índice de integraciones AlexIA

Fecha: 2026-11-03. Estado: **implementación parcial local; contratos restantes pendientes**. [Estado real y evidencia](evidence/IMPLEMENTACION-PLAN00-PLAN01.md). Las tablas originales de código observado se conservan como base histórica de planificación.
Revisión de consistencia C1: contratos contrastados con PLAN01 GitHub R2 el 2026-11-03. Las correcciones documentales no acreditan implementación. Hallazgos y condiciones de salida en [la auditoría cruzada](evidence/PLAN00-GITHUB-CONSISTENCIA.md).
Solicitud: comunidad técnica primero; automatización de desarrollo y conexión tanto a entorno local como a servicios externos. Esta entrega crea planes, instala la skill solicitada y verifica la aplicación actual. No conecta cuentas, publica mensajes ni despliega infraestructura.

## 1. Cómo leer y ejecutar estos planes

Cada aplicación tiene un Markdown `PLAN##-APLICACION.md`. Se separan PowerShell de Bash y Linear de Jira porque sus mecanismos, permisos y pruebas no son intercambiables. La numeración identifica documentos; **no obliga a implementar todo en ese orden**. El orden recomendado está en la sección 10.

Este documento es la única definición de contratos transversales propuestos. Los planes individuales añaden particularidades, sin duplicar otro núcleo. El diseño vigente/histórico permanece en [ARCHITECTURE.md](../../.agents/.docs/ARCHITECTURE.md), [BACKEND.md](../../.agents/.docs/BACKEND.md), [INFRASTRUCTURE.md](../../.agents/.docs/INFRASTRUCTURE.md), [DEPLOY.md](../../.agents/.docs/DEPLOY.md), [UI.md](../../.agents/.docs/UI.md) y [DOCKER.md](../../.agents/.docs/DOCKER.md). Se conserva el contexto anterior: estos planes no afirman que V1/V2 ya cambiaron de comportamiento.

| Plan | Aplicación | Prioridad / dependencia principal |
|---|---|---|
| 01 | [GitHub](PLAN01-GITHUB.md) | P1, R2: crear PR de última rama y squash merge por Alexa |
| 02 | [Visual Studio Code](PLAN02-VSCODE.md) | P2, después del primer flujo de trabajos |
| 03 | [PowerShell](PLAN03-POWERSHELL.md) | P1, ejecutor de tareas en Windows |
| 04 | [Bash](PLAN04-BASH.md) | P1 para Linux; P2 para WSL |
| 05 | [GitHub Actions](PLAN05-GITHUB-ACTIONS.md) | P1, diagnóstico de CI |
| 06 | [Docker](PLAN06-DOCKER.md) | P2, ejecución reproducible |
| 07 | [Playwright](PLAN07-PLAYWRIGHT.md) | P2, validación web |
| 08 | [Postman](PLAN08-POSTMAN.md) | P2, pruebas de APIs |
| 09 | [GitLab](PLAN09-GITLAB.md) | P2, alternativa de repositorios |
| 10 | [Gitleaks](PLAN10-GITLEAKS.md) | P1, primera validación de secretos |
| 11 | [OSV-Scanner](PLAN11-OSV-SCANNER.md) | P1, dependencias vulnerables |
| 12 | [Semgrep](PLAN12-SEMGREP.md) | P2, análisis estático |
| 13 | [Trivy](PLAN13-TRIVY.md) | P2, imágenes y configuración |
| 14 | [ZAP](PLAN14-ZAP.md) | P3, seguridad dinámica web |
| 15 | [Linear](PLAN15-LINEAR.md) | P2, tareas técnicas |
| 16 | [Jira](PLAN16-JIRA.md) | P2/P3, equipos que lo utilicen |
| 17 | [Notion](PLAN17-NOTION.md) | P2, contexto documental |
| 18 | [Slack](PLAN18-SLACK.md) | P2, avisos; P3, comandos de equipo |
| 19 | [Sentry](PLAN19-SENTRY.md) | P2, investigación de errores |
| 20 | [Vercel](PLAN20-VERCEL.md) | P3, diagnóstico de despliegues |

## 2. Estado observado frente al diseño objetivo

| Área | Código observado | Trabajo propuesto |
|---|---|---|
| Runtime | Rust, Axum, Tokio, Serde | Conservar stack y añadir módulos pequeños |
| IA | LanguageModel::generate(prompt) devuelve texto; Antigravity local | Núcleo de tareas y herramientas separado del proveedor |
| Arranque | Control::prepare depende de ALEXA_SKILL_ID para preparar servicio | Separar configuración del proveedor, núcleo y canal Alexa |
| UI | Vite vanilla/Tailwind; settings y túnel | Registro de integraciones, permisos, tareas y evidencias |
| GitHub | Tarjeta/modal con conexión deshabilitada | Primer conector remoto real |
| Seguridad local | Loopback, Host/Origin, marcador X-Alexia-UI, rechazo forwarded | Conservar guardas y añadir sesión/pairing para nuevas capacidades |
| Alexa | Firma/certificado/timestamp/skill ID; userId solo se comprueba no vacío | Vincular identidad a permisos de proyecto antes de ejecutar tareas |
| Voz | 800 caracteres y eliminación de bloques de código; respuesta en 7 s | Resumen hablado separado de resultados técnicos |
| Procesos | Proveedor CLI, modo plan y kill_on_drop | Ejecutor específico con límites/árbol/aislamiento probado |
| Datos | .env/metaprompt locales; sin BD del producto | Registro no secreto y retención mínima propuesta, sin añadir Supabase |

La documentación contiene secciones históricas que describen bloqueo total, respuestas pendientes o sesiones antiguas; el código y las actualizaciones fechadas más recientes son la evidencia de comportamiento actual. No usar una tabla antigua para prometer funciones nuevas.

La instrucción «No uses herramientas» del metaprompt actual no es una barrera de permisos. El modo plan de Antigravity tampoco garantiza aislamiento de todas sus herramientas. Dar acceso al repositorio o añadir APIs requiere controles en el runtime, no solamente un prompt diferente.

## 3. Resultado inicial y límites de producto

Primer flujo: **seleccionar repositorio/ramas, ejecutar comprobaciones registradas y recibir un informe con evidencia**. GitHub aporta contexto remoto; PowerShell/Bash aporta ejecución local; Gitleaks y OSV aportan señales concretas. La consulta original de Alexa debe seguir funcionando.

El primer despliegue es una instalación personal por usuario. Una comunidad puede compartir software, recipes y reglas revisadas sin compartir procesos, credenciales ni carpetas. No convertir esta arquitectura en SaaS multiusuario simplemente cambiando BIND_ADDR a 0.0.0.0. Ese salto exige autenticación, tenancy, aislamiento de ejecución, cuotas, retención, operación y costes propios.

Fuera de la primera ola general: producción automática, tareas de administrador/root, comandos libres del modelo, ejecución de repositorios desconocidos en el host y catálogo abierto de conectores sin revisión. La exclusión inicial de merges queda sustituida para GitHub por la solicitud expresa documentada en [PLAN01 R2](PLAN01-GITHUB.md): creación de PR y squash merge condicionado a identidad, destino, tests y permisos. No se provisiona broker OAuth ni infraestructura en esta entrega.

## 4. Arquitectura propuesta y límites de módulos

Flujo lógico:

```text
UI / extensión emparejada / canal Alexa autorizado
  -> identidad + proyecto + validación de entrada
  -> autorización de capacidad y recurso
  -> gestor de trabajos (cola, progreso, cancelación)
  -> conector HTTP o ejecutor local
  -> resultado estructurado + evidencia saneada
  -> explicación IA opcional
  -> panel técnico / resumen hablado
```

El modelo propone acciones tipadas; el núcleo decide si son válidas y autorizadas. Un resultado de herramienta puede explicarse sin ejecutar otra herramienta. Los escáneres pueden funcionar con el proveedor IA no disponible; mostrar su evidencia y marcar explicación no disponible.

La IA es opcional únicamente para capacidades que no la necesitan. Cada acción declara sus dependencias: `github.pull_requests.create` exige el perfil técnico Antigravity disponible y un snapshot aislado; sin ellos no se publica una PR con texto inventado. `github.pull_requests.squash_merge` no requiere generar texto con IA, pero sí identidad, gate de tests y guardas de destino. Readiness se informa por capacidad; la ausencia de ALEXA_SKILL_ID no debe bloquear el núcleo ni tareas iniciadas desde UI.

| Módulo nuevo propuesto | Responsabilidad | No debe hacer |
|---|---|---|
| src/projects/ | IDs, raíz canónica, snapshot/commit | Elegir carpetas desde texto no validado |
| src/jobs/selection | Descubrir objetivos permitidos sin conocer proyecto | Ejecutar efectos o usar IDs ficticios para saltar autorización |
| src/integrations/ | Catálogo, capacidades, estado y adaptadores | Guardar secretos dentro de modelos de UI |
| src/auth/ | Sesión local, pairing e identidad | Confiar en userId suministrado sin vínculo |
| src/policy/ | Grants, efectos y validación por recurso | Delegar autorización al modelo |
| src/jobs/ | Cola, estados, resultados y cancelación | Ejecutar shell genérica |
| src/execution/ | Procesos, argv, límites y perfiles | Prometer sandbox con cwd o modo plan |
| src/secrets/ | Vault del sistema y referencias opacas | Retornar tokens al navegador |
| src/artifacts/ | Acceso por ID, tamaño, MIME y TTL | Servir HTML arbitrario en origen de administración |

Reqwest/Rustls u otra librería HTTP se seleccionan al implementar, con versión compatible con MSRV real y Cargo.lock; no se añade dependencia en esta entrega. HTTP cliente reutilizable por configuración, TLS validado y políticas de redirect/destino explícitas. No hacen falta microservicios ni una base vectorial para validar el primer flujo.

## 5. Modelo de identidad, conexión y autorización

Tipos propuestos:

```text
Principal: local_owner | paired_client | linked_external_user
ProjectRef: project_id, owner_id, source_kind, source_locator, trust_profile
  local: canonical_root obligatoria, permisos de filesystem explícitos
  remote_repository: provider_origin + immutable_repository_id;
                     checkout local opcional y snapshot efímero propio
Connection: connection_id, owner_id, provider, external_account_id,
            instance_origin, resource_allowlist, credential_refs_by_role,
            granted_capabilities, status, verified_at
Grant: principal_id, project_id, capability, resource_scope,
       parameter_constraints, effects, version_policy, expires_at
  version_policy: snapshot(hash/version) | follow_ref(ref, constraints)
```

Un repo remoto sin checkout sigue siendo un proyecto válido: se registra por identidad inmutable, con owner_id y vínculo de conexión. Nombre/owner de GitHub se usan para presentación y se revalidan ante renombre. Materializar un snapshot para Antigravity no concede acceso a carpetas del usuario. Una conexión a un repo compartido no concede acceso al ProjectRef de otro principal.

Estados de conexión canónicos: disconnected, configured_unverified, connecting, connected, degraded, expired, revoked, unavailable. `connected` requiere una llamada de verificación real; configuración sin verificación usa configured_unverified, no un estado alternativo fuera del enum. El catálogo debe mostrar por qué no está disponible un CLI/daemon.

`credential_refs_by_role` permite roles separados como user_access, user_refresh, app_private_key y webhook_signing. Un adaptador de una sola credencial puede usar su campo lógico `credential_ref`, normalizado al rol declarado; nunca interpretar ese campo como todas las credenciales de GitHub. Distinguir owner de conexión, instalación y secreto compartido del servicio. Desconectar un usuario revoca su acceso/jobs sin desinstalar una App compartida ni revocar credenciales de otros usuarios. El installation token no sustituye al user token para ampliar privilegios.

Sesión web propuesta: bootstrap local explícito desde arranque o código de un solo uso, cookie HttpOnly y SameSite estricta, Origin/Host validados y CSRF. El código es efímero y no va a logs persistentes ni URL compartida. X-Alexia-UI continúa como marcador de origen previsto, no contraseña. No abrir CORS para facilitar una extensión: usar un canal autenticado propio.

Pairing de extensión/cliente: código aleatorio de un uso, TTL corto, intercambio ligado a client_id y permisos. Almacenar verificador/hash del token del lado servidor y secreto en vault del cliente. Reutilización, expiración y revocación deben probarse. Procesos maliciosos del mismo usuario del SO siguen siendo una limitación; no prometer seguridad equivalente a aislamiento de máquina.

Alexa: la firma acredita procedencia de Amazon y skill esperada; no concede derechos de repositorio a todos los userId válidos. En piloto usar allowlist/vinculación del propietario con recursos concretos. Para comunidad con skill compartida, account linking y mapeo de identidad se diseñan antes de habilitar escrituras. La voz sola no es autenticación fuerte: se combina con identidad vinculada y grants por proyecto. Una orden concreta dentro de esa política autoriza la operación; no añadir una confirmación universal redundante. Aclarar recursos o ramas ambiguos antes de actuar.

Política: deny-by-default por capacidad. Lecturas, ejecución y escritura se autorizan por separado. Permisos ya concedidos dentro del alcance siguen vigentes sin confirmaciones repetidas. Si hace falta nueva autorización, preparar antes destino, parámetros y diferencias concretas. Un cambio de hash, proyecto, host o efecto fuera del grant invalida esa autorización.

Por defecto version_policy es snapshot. Si el usuario configuró expresamente follow_ref para una PR y destino, un head nuevo no invalida todo el grant: invalida el snapshot/test gate, obliga a revalidar y conserva el mismo plazo original. No extiende permisos a otra PR, repositorio, base o efecto. No activar seguimiento por omisión. Un cambio de versión de política, identidad, destino resuelto o revocación bloquea nuevas escrituras aunque el hash no cambie. El PUT final siempre condiciona el SHA validado.

## 6. Secretos, configuración y persistencia propuesta

Primero introducir interfaz SecretStore. Backend Windows: Credential Manager/DPAPI según librería revisada; macOS: Keychain; Linux: Secret Service o mecanismo equivalente explícito. Si no hay vault soportado, informar indisponibilidad; no escribir tokens en JSON/.env como fallback silencioso. La elección concreta de librería queda pendiente de compatibilidad y revisión.

La configuración no secreta de conexiones y grants puede almacenarse en archivos JSON versionados de esquema bajo directorio de datos del usuario con ACL y escritura atómica. Es una **nueva persistencia propuesta**, distinta de la configuración existente; documentar y aprobar su política al implementar. No almacenar conversaciones de IA por conveniencia.

Propuesta de retención inicial:

| Dato | Política inicial propuesta | Prueba |
|---|---|---|
| Tokens OAuth/keys | Vault hasta revocación; rotación atómica | Reinicio/revocación sin fugas |
| Contexto de código | RAM, solo fragmentos elegidos | Reinicio no recupera conversación |
| Metadatos de jobs | 24 h, sin prompts/salida sensible | Expiración y borrado por propietario |
| Recibos de creación/procedencia de PR bot | Mientras PR esté abierta; propuesta 30 días tras estado terminal | Una PR de varios días conserva autoría; otro usuario no puede adoptarla |
| Recibos de efectos no reconciliados | No borrar por TTL hasta resolver o intervención explícita | Reinicio tras POST no produce duplicado |
| Índice/cursor de pushes y entregas | Ventana explícita propuesta de 30 días mientras conexión activa, con estado de cobertura | Pérdida/expiración marca freshness_unknown |
| Reportes saneados | 24 h o borrado manual | Acceso y TTL por job |
| Reportes crudos sensibles | Evitar; temporal ACL y borrar al normalizar | Centinelas ausentes |
| Logs operativos | IDs, estados y tiempos; sin cuerpos | Auditoría de campos |

Los plazos son diseño pendiente de medición, no funcionalidad actual. Borrar un token local no siempre revoca el token remoto; cada conector implementa su revocación. Recuperación tras fallo de refresh debe evitar perder el nuevo token o usar uno rotado dos veces.

El TTL de 24 h empieza al terminar un job, no elimina jobs activos/esperando/reconciling ni recibos de procedencia referenciados. CreationReceipt es registro de autorización/procedencia independiente del reporte del job; no se deduce de su texto. Límites de capacidad y borrado explícito deben fallar de forma visible: si se perdió procedencia, no autorizar un bot por su label. Reconectar no importa recibos de otra identidad; cualquier revinculación exige mismos IDs de principal/usuario/App/instalación verificados y política explícita.

Antes de enviar una escritura externa, persistir EffectIntent con operation_id, conexión, principal, recurso, acción, snapshot/destino y request fingerprint saneado. Si se reinicia tras enviar sin recibo, reconciliar contra GitHub; no repetir automáticamente. Se propone un journal local con ACL y actualización atómica/recuperación probada; los cuerpos privados y tokens no se guardan. Un prototipo solo en RAM no habilita creación/merge real.

## 7. Contratos API y modelo de resultados

Todas las rutas administrativas de esta sección son **nuevas propuestas**. Mantenerlas en listener local autenticado. El router público Alexa no monta `/api/jobs`, `/api/integrations` ni artefactos. Los adaptadores Alexa llaman al mismo CommandService interno con principal verificado: no necesitan cookie de navegador, no hacen HTTP contra Vite ni falsifican X-Alexia-UI. Los callbacks OAuth y webhooks tienen ingreso separado y autenticación propia.

| Método/ruta propuesta | Entrada/salida | Autorización |
|---|---|---|
| GET /api/integrations | Catálogo y capacidades, sin secretos | Sesión local |
| POST /api/integrations/{id}/connect | Inicio de flujo, no token en respuesta | Owner; state ligado a sesión |
| GET /api/oauth/{provider}/callback | Código/state según proveedor | Excepción de callback acotada, no CORS abierto |
| POST /api/integrations/{id}/verify | Cuenta/recursos comprobados | Owner de conexión |
| POST /api/integrations/{id}/disconnect | Invalida acceso y revoca si procede | Owner |
| POST /api/jobs | Variante selection o execution; ver contrato debajo | Scope de conexión o grant de proyecto, nunca intercambiables |
| GET /api/jobs/{job_id} | Estado/progreso y resultado paginado | Propietario/cliente vinculado |
| POST /api/jobs/{job_id}/cancel | Solicitud de cancelación | Permiso de job |
| POST /api/jobs/{job_id}/input | choice_id/continuation_id de un job esperando | Principal, revisión, alcance y TTL del job |
| GET /api/artifacts/{artifact_id} | Artefacto saneado/MIME seguro | Pertenencia y TTL |

Callbacks OAuth están separados de los endpoints de acción y aceptan únicamente flujos previamente iniciados: state/verifier de un uso, redirect exacto, TTL y no logging de code. Un proveedor que requiera callback HTTPS o client secret puede necesitar un broker; el frontend/instalador no es un lugar confidencial.

Webhooks: listener/rutas específicos con validación del proveedor, límites antes de parsear, firma/frescura cuando estén soportadas y deduplicación. No reutilizar la verificación Alexa para otros proveedores. La ruta pública propuesta `/webhooks/github` verifica firma y solo ingiere observaciones; no publica ni fusiona y nunca expone administración. Polling/manual basta para conectores de lectura, pero no acredita la cronología completa requerida por «última rama» de PLAN01. Sin observación fiable, pedir selección y no declarar terminado el modo automático.

BC-07a cubre ingreso y cobertura y es requisito de G03; las notificaciones BC-07b siguen posteriores. Registrar la observación durable antes de acusar recepción satisfactoria. GitHub no reentrega automáticamente webhooks fallidos: pérdida, desconexión o almacenamiento lleno marca hueco de cobertura y exige reconciliación/redelivery autorizada o selección manual. No prometer que un polling posterior elimina todo hueco; no usar delivery UUID ni hora local de recepción como orden real de pushes.

### Trabajos de selección frente a trabajos de ejecución

Cuando el comando ya contiene un proyecto válido, usar `kind=execution` con project_id, capability e input. Los ejemplos de otros planes que omiten kind se interpretan exclusivamente como execution y siguen exigiendo proyecto real. Una cadena ficticia como resolved-by-authorized-selection no cumple el esquema.

Todo hijo execution requiere ProjectRef real, pertenencia al principal y grant de efecto revalidado; no hereda una autorización de escritura del mero hecho de tener un padre selection.

Cuando Alexa pide «mi última rama/PR» sin proyecto, acusar primero un `kind=selection` acotado a connection_id del principal. Solo admite resolución de objetivos mediante lecturas permitidas; no tiene capacidades de escritura ni ejecución técnica de código. Capability de GitHub: `github.targets.resolve`.

```json
{
  "kind": "selection",
  "integration_id": "github",
  "connection_id": "github-connection-demo",
  "capability": "github.targets.resolve",
  "input": {"operation": "create_pr", "selector": "latest_verified_user_push"},
  "idempotency_key": "verified-channel-request-id"
}
```

El scope de repositorios se deriva de la conexión/grant del servidor, no de una lista del modelo. La selección ocurre fuera del request Alexa. Al resolver, registrar o recuperar el ProjectRef remoto real, fijar SelectionSnapshot, comprobar de nuevo permiso del usuario/proyecto/efecto y crear un hijo execution con parent_job_id. No ejecutar escritura con project_id ausente/null ni transformar una selección en escritura sin autorización. El job padre permite consultar/cancelar todo el recorrido por el ID acusado a Alexa.

Si falta rama/proyecto, waiting_for_input conserva continuation_id opaco, elecciones permitidas, principal y expiración. UI o intent de aclaración envía una elección vinculada; revalidar recurso antes de crear el hijo. Un token de otro usuario, vencido o ya usado se rechaza. Aclarar puede mantener sesión con reprompt, pero no exige dejarla abierta durante el job; si ya cerró, continuar con el ID correspondiente en nueva invocación o panel. Dos jobs pendientes ambiguos no se resuelven por un puntero global.

Petición de tarea:

```json
{
  "integration_id": "gitleaks",
  "project_id": "project-demo",
  "capability": "gitleaks.scan.directory",
  "input": {"snapshot_id": "snapshot-demo", "mode": "directory"},
  "idempotency_key": "demo-check-001"
}
```

HTTP 202 devuelve job_id/status_url tras validación local, sin enumerar todos los repositorios dentro del plazo de Alexa. La clave única es namespace de origen + principal + connection/scope + capability + idempotency_key. El digest canónico de entrada es un atributo almacenado para comparar, **no parte de la clave única**: misma clave/entrada devuelve el mismo job; misma clave con otra entrada devuelve 409. El adaptador Alexa deduplica también por application_id/user_id/requestId verificados antes de resolver conexión/proyecto.

Una repetición devuelve el mismo padre, snapshot ya seleccionado, hijos y recibos; no vuelve a elegir «lo último» ni crea otro efecto si hubo un push posterior. Una nueva orden genuina lleva nuevo request ID. La idempotencia local no convierte POST/PUT externos en atómicos: EffectIntent/recibo y reconciliación deciden si se puede reintentar.

La capacidad del ejemplo, `gitleaks.scan.directory`, se define en [PLAN10](PLAN10-GITLEAKS.md); cada adaptador declara las suyas. El ejemplo de creación/merge por voz de GitHub se define en [PLAN01 R2](PLAN01-GITHUB.md).

Estados canónicos de job: queued, running, waiting_for_input, waiting_for_external, reconciling, completed, failed, cancelled, timed_out, interrupted, unknown_outcome. `stage` describe pasos del adaptador; `result_code` describe un resultado de negocio. awaiting_selection/awaiting_authorization son stage de waiting_for_input; waiting_for_checks es stage de waiting_for_external; already_exists/nothing_to_open son result_code de completed/no_change. No añadir esos valores al enum de job de manera informal.

`cancel_requested` es un flag, no prueba de terminación. Su establecimiento impide enviar nuevos EffectIntent/hijos; el límite de envío usa exclusión con cancelación y revocación. Una escritura ya enviada no se cancela retroactivamente: reconciling/unknown_outcome hasta comprobar qué ocurrió. Padre e hijos propagan cancelación y comparten el plazo original; un padre no figura cancelled si un hijo fusionó. Jobs al reiniciar se interrumpen o reconcilian; no se reanudan escrituras sin volver a verificar política y permisos.

result_status canónico: clean/findings para scanners, success, no_change, blocked, partial, error. `execution_status` refleja el estado del job; los nombres de etapas no lo sustituyen. Fallo operativo -> failed/error; selección sin cambios -> completed/no_change; requisito no satisfecho al terminar -> completed/blocked. Un merge que espera un nuevo commit autorizado mantiene waiting_for_external y gate invalidado hasta deadline, sin fusionar con tests fallidos.

```json
{
  "schema_version": 1,
  "job_id": "job-demo",
  "execution_status": "completed",
  "result_status": "clean",
  "source": {"provider": "gitleaks", "version": "version-fijada"},
  "target": {"project_id": "project-demo", "snapshot_id": "snapshot-demo"},
  "coverage": {"status": "complete", "skipped": []},
  "findings": [],
  "evidence_ids": [],
  "warnings": [],
  "explanation_status": "not_requested"
}
```

`execution_status` y `result_status` no significan lo mismo. Un scanner que encontró problemas puede ejecutarse correctamente; una caída de herramienta no significa cero problemas. Finding común: tool, rule/advisory ID, severity original, confidence si existe, ubicación, evidencia saneada, remediation propuesta y fingerprint. Agrupar aliases sin borrar procedencia. El modelo no inventa severidad/certeza de explotación.

Errores internos normalizados: unauthenticated, permission_denied, unavailable, unsupported_version, rate_limited, invalid_input, output_too_large, partial_coverage, provider_error, timed_out, cancelled y unknown_outcome. Mensaje al usuario saneado, trace_id operativo sin contenido.

## 8. Ejecución y acceso a datos no confiables

TaskSpec contiene executable_id, argv tipado, canonical cwd, environment allowlist, stdin policy, time/resource limits, efectos y recipe hash. El modelo propone task_id, no una línea shell. Deshabilitar hooks/instalación automática al inspeccionar repositorios. npm test/cargo build/scripts son ejecución de código del proyecto, aunque su nombre parezca benigno.

PowerShell: -File revisado, argumentos separados, NoProfile y NonInteractive; no Invoke-Expression ni bypass. Unix: argv directo o script fijo, BASH_ENV/ENV eliminados, sin login shell. Windows Job Object o grupo Unix para cierre de descendientes; probar nietos y salida que llena pipes. Esto no constituye sandbox de filesystem/red.

Proyectos desconocidos: contenedor endurecido/VM o runner dedicado antes de ejecutar. No montar HOME, agentes SSH ni socket Docker. Docker daemon tiene privilegios amplios; no ofrecerlo a código de terceros como una herramienta libre. Separar provisioning de dependencias y ejecución de tareas para controlar descargas/licencias.

HTTP: base origins registradas, TLS activo, redirects controlados, DNS/rangos acordes al caso. Una instancia Self-Hosted privada requiere autorización específica, no una excepción global SSRF. No reenviar Authorization a hosts de descarga distintos. Limitar JSON, ZIP, bytes comprimidos/expandidos, paths y artefactos.

Prompt injection: repositorios, issues, documentos, registros y resultados externos son datos. No pueden pedir nuevas credenciales ni cambiar proyecto/destino de escritura. No basta una frase en metaprompt: las capacidades se validan fuera del proveedor.

## 9. UI, voz y experiencia de comunidad

Mantener Vite vanilla/Tailwind; no añadir React por necesidad del catálogo. UI propuesta: catálogo por propósito, estado real, cuenta/recursos, permisos legibles, desconectar y última verificación. Un conector futuro figura como no disponible, no conectado.

Panel de tareas: origen, proyecto, alcance, estado, progreso, cancelación, resultado y evidencia. Texto externo se inserta con textContent o renderizador saneado. Descargas HTML/trace quedan aisladas del origen administrativo. Teclado, foco, estados accesibles y viewport móvil se comprueban.

Conservar PreguntarIntent y respuesta directa actual. Las tareas nuevas requieren intents/canal propios y contrato documentado: acuse breve y resultado recuperable en panel; no intentar mantener un escaneo dentro de los siete segundos actuales. No cambiar el flujo de voz existente solo por este plan. Vincular identidad a connection_id antes de una selección; vincular a ProjectRef/grant real antes de ejecutar efectos. El acuse selection no afirma que una PR exista ni que un merge se haya completado.

Un enlace http://localhost no sirve a otro miembro de Slack o a un teléfono remoto. Empezar con uso local; un portal remoto requiere autorización y diseño adicionales. No publicar UI ni terminal mediante el túnel Alexa para resolverlo.

## 10. Orden recomendado y dependencias

| Ola | Entregable | Planes | Criterio de salida |
|---|---|---|---|
| 0 | Proyecto, identidad, permisos y tareas con fixture | PLAN00 BC-01..05 | Flujo local observable y denegaciones probadas |
| 1 | Validación de repositorio en Windows o Linux | PLAN03 o 04, PLAN10, PLAN11 | Informe real sin secretos ni comandos libres |
| 1b | Contexto remoto y diagnóstico CI | PLAN01, PLAN05 | PR/SHA y logs reales con lectura limitada |
| 1c | Flujos GitHub solicitados en revisión R2 | PLAN01 R2 + BC-06 + gate de PLAN05 | PR desde rama propia sin duplicar; squash a destino con tests válidos |
| 2 | Trabajo desde editor y pruebas web/API | PLAN02, PLAN06, PLAN07, PLAN08 | Evidencias y cancelación reproducibles |
| 2b | Análisis estático/configuración y GitLab | PLAN09, PLAN12, PLAN13 | Cobertura honesta y alternativa de repositorio |
| 3 | Coordinación y producción observada | PLAN15..20 | Identidad, audiencia, privacidad y permisos comprobados |
| 3b | Seguridad web dinámica | PLAN14 | Alcance/aislamiento demostrados en staging |

Las olas son prioridades, no promesas de calendario. No estimar semanas sin conocer mantenedores, SO, distribución OAuth y disponibilidad de cuentas fixture. No implementar veinte conectores a la vez. Primer piloto: una tarea útil completa y después ampliar. Una nueva integración HTTP de lectura no debe esperar innecesariamente un backend de ejecución que no utiliza: BC-04 puede declararse no aplicable para esa entrega.

## 11. Tareas del núcleo y checkpoints

### BC-01 — Registrar un proyecto y su contexto permitido

Trabajo BC-01a: tipos/registro de proyecto local o remote_repository y snapshot con fixtures, sin requerir conexión GitHub real. Trabajo BC-01b: validación de raíz canónica solo para fuentes locales/materialización autorizada. Archivos propuestos: src/projects/mod.rs, src/projects/snapshot.rs y tests/projects.rs. Cada entrega S/M.

- [ ] Identidad estable y source_locator autorizado; raíz obligatoria solo para local.
- [ ] Traversal, symlink/junction y archivo secreto excluidos; fijar commit/hash.
- [ ] Prueba con dos raíces y Unicode; inspección del contexto entregado.

Dependencias: ninguna. Verificación: tests de rutas reales en host soportado, no solo normalización de strings.

### BC-02 — Crear y consultar un trabajo fixture

Trabajo BC-02a: gestor con fixtures y estados tipados, aún sin efectos externos. BC-02b: journal durable de comandos/EffectIntent/recibos y recuperación, antes de habilitar escrituras reales. Archivos propuestos en src/jobs/ y tests focalizados; separar entregas para no exceder cinco archivos cada una. El prototipo en memoria no satisface el criterio de producción de GitHub.

- [ ] POST devuelve 202; GET devuelve estados terminales coherentes.
- [ ] Idempotencia distingue entradas diferentes y límite de cola.
- [ ] Cancelación y reinicio simulado no producen éxito ficticio.
- [ ] Selección sin proyecto no escribe; un efecto enviado sin recibo se reconcilia y no se repite.

Dependencias: BC-01. Verificación: tests de concurrencia, cancelación e IDs ajenos.

### Checkpoint A — Proyecto a resultado fixture

- [ ] UI futura puede seguir un job sin usar IA ni secretos.
- [ ] Ninguna ruta administrativa nueva existe en public_router; ingress de proveedor, si se habilita, queda separado y verificado.
- [ ] Mantener pruebas de Alexa y configuración existentes.

### BC-03 — Autorizar identidad y custodiar credenciales

Trabajo dividido en BC-03a sesión/pairing y BC-03b SecretStore/política; cada uno S/M con hasta cinco archivos. Módulos src/auth/, src/secrets/, src/policy/ y sus tests. Definir contrato antes de conectar OAuth.

- [ ] Grants por principal/proyecto/capacidad y parámetros; IDs ajenos denegados.
- [ ] Pairing de un uso, TTL, revocación y no reutilización.
- [ ] Vault disponible o error explícito; sin fallback en texto plano.

Dependencias: BC-01a/BC-02a para contratos; BC-02b antes de efectos reales. OAuth/identidad concretos GitHub se implementan en G01, no como requisito circular de este núcleo. Verificación: solicitudes de otro origen, código vencido y secreto centinela. La persistencia nueva se documenta y valida con permisos/ACL.

### BC-04 — Ejecutar una receta fixture en host soportado

Trabajo dividido en BC-04a argv/manifiesto y BC-04b procesos/cancelación, un backend SO inicial. Seguir PLAN03 o PLAN04. Docker es una entrega aparte, no requisito oculto.

Para GitHub G05, añadir perfil técnico de solo lectura con aislamiento de snapshot/proveedor y prueba de ausencia de credenciales GitHub; no basta que un script fixture corra sin inyección. Si no existe aislamiento compatible con Antigravity, la capacidad de creación dependiente permanece no disponible, sin degradar a un proceso con acceso libre. Merge/lecturas que no invocan IA no dependen de BC-04.

El perfil técnico se identifica como BC-04t en el grafo; consume BC-04a/04b, no requiere ejecutar código del repositorio.

- [ ] Metacaracteres son datos; scripts hash y ejecutable fijos.
- [ ] Timeout termina hijos/nietos y drena stdout/stderr.
- [ ] Env por allowlist y efectos autorizados; confianza no equivale a sandbox.

Dependencias: BC-01..03. Verificación: pruebas reales de proceso/descendientes y salida máxima. Marcar no aplicable para un conector HTTP de lectura que no ejecuta código.

### BC-05 — Catálogo y resultado visibles de extremo a extremo

Trabajo dividido en BC-05a contrato registro/rutas y BC-05b UI catálogo/jobs. Archivos candidatos: src/integrations/mod.rs, src/control.rs, ui/src/integrations.js, ui/src/jobs.js y tests focalizados. Dividir si supera cinco archivos por entrega.

- [ ] Estado de conexión verificado, capacidades y recursos reales.
- [ ] Resultado con cobertura, evidencias, errores y cancelación.
- [ ] Navegación por teclado, móvil, consola y red comprobados con CLI.

Dependencias de BC-05a: BC-01a/BC-02a/BC-03, fixtures y catálogo deshabilitado; **no exige conector real**. BC-05b incorpora el primer recorrido real después de G06/G07 o su equivalente. Ese recorrido valida la entrega, no es prerrequisito para construir el cliente que lo hace posible. BC-04 aplica solo si se ejecuta un proceso. No habilitar botones que no tengan recorrido completo.

### Checkpoint B — Primer conector real

- [ ] Elegir PLAN03+10/11 o PLAN01 según piloto; no implementar ambos incompletos.
- [ ] Dos proyectos no comparten recursos ni credenciales.
- [ ] Desconectar/revocar y recuperar tras error está probado.
- [ ] Informe distingue evidencia de herramienta y explicación IA.

### BC-06 — Canal Alexa para trabajos, requisito de GitHub R2

Nuevo intent/canal que resuelve identidad vinculada y encola; preservar consulta directa vigente. Pruebas de firma/timestamp/skill ID intactas, userId autorizado, timeout y cierre. Documentar modelo de interacción y probar simulador/dispositivo aparte.

BC-06a implementa despacho/acuse y CommandService con fixtures después de BC-02a/BC-03, sin exigir los flujos GitHub completos. G07/G10 incorporan sus handlers reales y BC-06b valida simulador/dispositivo después. No poner BC-06b como dependencia de G01 ni resolver globalmente repositorios antes de acusar.

Revisión 2026-11-03: [PLAN01 R2](PLAN01-GITHUB.md) solicita «Alexa, abre mi agente», creación de PR desde última rama subida sin PR y squash merge a rama nombrada. BC-06 debe completarse para ese recorrido, sin esperar a todos los demás conectores. Identidad GitHub humana por user access token y recibos de creación para PR bot; no inferir propiedad por repo o autores de commits. La disponibilidad del nombre de invocación se verifica en Amazon, sin cambiar la activación del dispositivo desde el backend.

### BC-07 — Ingreso de eventos y notificaciones con prioridades distintas

Webhooks firmados o transporte saliente, deduplicación y outbox. Cada evento verifica pertenencia y grant. No enviar mensajes a terceros durante implementación sin autorización explícita; las fixtures no publican. Quietud si no hay cambio útil.

BC-07a (G03): contrato de ingreso, journal/coverage y fallo de entrega; se entrega antes de afirmar «última rama» automática. BC-07b: outbox de notificaciones posterior, sin ser dependencia de crear PR o hacer merge. No mezclar publicación de mensajes con ingestión de eventos.

BC-07a usa BC-02b para persistir observación/cursor antes del acuse. Una fixture en RAM sirve para construir el parser, pero no acredita continuidad del índice. Si no se provisiona receptor, ofrecer selección manual y rotular el modo automático no disponible.

### Orden acíclico mínimo para GitHub

BC-01a -> BC-02a -> BC-03 -> BC-05a; BC-02b se añade antes de efectos y del ingreso durable BC-07a. G01 consume ese núcleo. G02 y G04 consumen G01; G03 consume G01 + BC-07a. G05 consume G02/G03 + BC-04 técnico. G06 consume G01..G05 + BC-02b. BC-06a se construye con el núcleo y G07 añade creación real. G08 consume G04 + lector CI de PLAN05, G09 consume G08 + BC-02b + destino validado, G10 añade intent real con BC-06a. G11/BC-05b/BC-06b verifican el recorrido ya construido. Ninguna flecha exige un efecto real para poder construir su cliente.

La tabla siguiente es el grafo canónico para ese recorrido; `CI-READ` es el subconjunto lector de PLAN05, no su finalización completa. `DEST-GUARD` es una condición de habilitación aún pendiente, no una capacidad ya comprobada. BC-05b/06b incorporan UI y validación de canal antes de la aceptación global G11.

| Nodo | Dependencias directas |
|---|---|
| BC-01a | ninguna |
| BC-02a | BC-01a |
| BC-03 | BC-01a, BC-02a |
| BC-02b | BC-02a, BC-03 |
| BC-04a | BC-03 |
| BC-04b | BC-04a |
| BC-04t | BC-04b |
| BC-05a | BC-03 |
| BC-06a | BC-02a, BC-03 |
| BC-07a | BC-02b, BC-03 |
| G01 | BC-03, BC-05a |
| G02 | G01 |
| G03 | G01, BC-07a |
| G04 | G01 |
| G05 | G02, G03, BC-04t |
| G06 | G01, G02, G03, G04, G05, BC-02b |
| G07 | G06, BC-06a |
| CI-READ | G01 |
| G08 | G04, CI-READ |
| DEST-GUARD | G02, G08 |
| G09 | G08, BC-02b, DEST-GUARD |
| G10 | G09, BC-06a |
| BC-05b | G07, G10 |
| BC-06b | G07, G10 |
| G11 | BC-05b, BC-06b |

### BC-08 — Distribución comunitaria

Instalador y diagnóstico por SO, provisioning explícito de CLIs, licencias/firmas, actualizaciones y revocación. Separar BYO App de broker administrado. Medir recursos/costes antes de prometer disponibilidad. Conectores aportados por comunidad necesitan revisión: no descargar/ejecutar plugins por petición del modelo.

## 12. Validación futura por integración

### Condiciones de habilitación que no se cierran con una revisión de Markdown

- G05: comprobar aislamiento real del perfil Antigravity y ausencia de credenciales GitHub antes de permitir creación dependiente de IA.
- G03: comprobar ingreso durable/cobertura o declarar modo manual; no afirmar que polling garantiza último push.
- G09: `destination_guard_status` permanece unverified hasta demostrar la precondición de destino frente a retarget concurrente. La API REST de merge condiciona head, no base; un lock local/relectura no prueba atomicidad. Mientras no exista control verificable suficiente, devolver destination_guard_unverified **antes de retarget o merge**, sin PATCH/PUT. No aprobar esta condición por documentar el riesgo, ni saltar protecciones para hacerlo funcionar.
- BC-02b: verificar caída/reinicio en el límite de envío y reconciliación sin doble efecto. Guardas del servidor y pruebas reales son requisitos adicionales a enlaces/esquemas correctos.

Capas: esquema/parser -> política/identidad -> adaptador con fixture -> recurso real de prueba -> UI real -> recuperación/revocación. Las pruebas UI con mocks verifican presentación; no acreditan acceso remoto. Un código HTTP exitoso no sustituye comprobar JSON, permisos y efecto.

La implementación modifica documentación afectada en el mismo cambio. Ejecutar checks pertinentes del repositorio y los tests nuevos realmente creados. No añadir pruebas que repitan texto del plan; probar límites y comportamiento. Tests de agentes/editor/procesos no se sustituyen por Playwright en navegador.

Matriz mínima: éxito, recurso ajeno, expiración/revocación, rate limit, timeout, cancelación, respuesta inválida/grande, dato externo hostil, pérdida de red, reinicio y cobertura parcial. Escrituras añaden idempotencia/reconciliación y cambio concurrente del recurso.

## 13. Fuentes, licencias y contratos pendientes

Cada plan enlaza fuentes oficiales consultadas en esta sesión. Las páginas y opciones cambian; fijar versión probada y revalidar contrato antes de implementar. No se copian ejemplos con tokens reales ni se presupone gratuidad, compatibilidad, cuota o permiso de distribución. Las estimaciones de límites son propuestas, no mediciones.

Limitaciones de investigación: el navegador no recuperó el manual GNU Bash ni su espejo Savannah; revalidar esas fuentes y el manual de la versión instalada antes de implementar. Workspace Trust se recuperó posteriormente mediante búsqueda oficial. Las rutas iniciales supuestas de Vercel no se recuperaron, y se localizaron después sus referencias oficiales correctas: PLAN20 ya registra v7 para listado, v13 para detalle/creación y v3 para eventos. Los demás planes citan mecanismos revisados, sin afirmar pruebas reales de las aplicaciones externas.

No se instala aquí ninguno de los veinte productos propuestos. La única instalación solicitada para esta entrega es la skill y el ejecutable Playwright CLI usados para validar la aplicación actual.

## 14. Registro de la instalación solicitada

- Skill instalada **antes de crear estos Markdown** desde microsoft/playwright-cli, ruta skills/playwright-cli.
- Instalador utilizado: skill-installer de Codex con --repo microsoft/playwright-cli --path skills/playwright-cli.
- Destino local: `C:/Users/sebas/.codex/skills/playwright-cli/SKILL.md`.
- Ejecutable instalado: paquete oficial `@playwright/cli`, versión observada **0.1.23**.
- Runtime observado: Node **24.21.0**, npm **11.19.0**.
- La instalación global no añadió dependencias a Cargo.toml ni ui/package.json. La skill se descubrirá automáticamente desde el siguiente turno; sus instrucciones se leyeron explícitamente para esta prueba.

## 15. Validación de esta entrega documental

Estado: **entrega documental validada; integraciones propuestas sin implementar**.

- Skill oficial instalada antes de crear los Markdown; Playwright CLI 0.1.23 ejecutado contra Chrome.
- `cargo build --locked --offline`: correcto. Advertencia ambiental de hard links en caché incremental; Cargo copió los archivos y terminó con éxito.
- `npm --prefix ui run build`: correcto, Vite 8.3.5.
- **41 comprobaciones UI/API correctas**, backend Rust real y assets compilados, sin interceptar ni simular peticiones.
- Escritorio 1440x900 y móvil 390x844; ancho móvil observado 390, sin desbordamiento horizontal global.
- Cero errores de consola, cero warnings de consola y cero excepciones JavaScript durante el recorrido registrado.
- Verificados redacción de secreto sintético, guardas locales, aislamiento del servidor público, HTTP 422 sin mutación y bloqueo Alexa HTTP 503 sin ID.
- Modales GitHub/Alexa, foco/Escape, filtros, edición/descarte, guardado fixture y ausencia real del API futuro comprobados.
- Hashes SHA-256 de `.env` y `prompts/meta-prompt.txt` originales iguales antes/después. Los cambios de prueba se limitaron a `target/integration-plans/runtime`.
- Revisión documental automatizada: 21 planes numerados, 20 aplicaciones, 22 ejemplos JSON válidos y todos los enlaces locales resolubles. `git diff --check` sin errores de whitespace. Sin patrones de credenciales reales en los planes.
- El primer acceso falló por permisos de sockets del entorno restringido; repetir servidor y navegador en el mismo entorno autorizado resolvió la conectividad sin cambiar código.

**Observación abierta OBS-01:** el metaprompt CRLF se normaliza a LF al cargar un textarea y `hasChanges` compara strings exactos. La barra de cambios aparece sin editar. Se reprodujo con copia temporal del archivo; un guardado de esa copia normalizó LF y permitió probar el resto del flujo. No se modificó código del producto ni el metaprompt original. Antes de implementar BC-05, normalizar saltos de línea en la comparación y probar CRLF/LF, espacios y cambios reales; no eliminar contenido significativo ni desactivar detección de cambios.

Evidencia persistida en [informe de validación](evidence/VALIDACION-2026-11-03.md), [resultado JSON](evidence/playwright-result.json), [captura de escritorio](evidence/current-integrations-desktop.png) y [captura móvil](evidence/current-integrations-mobile.png). Las capturas se inspeccionaron visualmente. Estas pruebas acreditan el estado actual de UI/API, no la implementación de las veinte integraciones, autenticación externa, funcionamiento de Antigravity ni prueba de un dispositivo Echo.
