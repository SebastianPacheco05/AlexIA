# PLAN01 — GitHub: crear la PR de mi última rama y hacer squash merge por voz

Fecha de revisión: 2026-11-03. Revisión: **R2**. Estado: **implementación parcial: conexión/selección y diagnóstico; creación/merge bloqueados por sus guardas**. [Estado real y evidencia](evidence/IMPLEMENTACION-PLAN00-PLAN01.md). Aceptación remota pendiente.
Consistencia C1: contratos contrastados con PLAN00; [hallazgos, correcciones y condiciones de salida](evidence/PLAN00-GITHUB-CONSISTENCIA.md).
Prioridad: P1. Canal inicial: Alexa. Integraciones implicadas: GitHub, Antigravity y comprobaciones de CI de PLAN05.

Esta revisión sustituye el alcance inicial limitado a lectura. La solicitud vigente incluye crear PR y squash merge. Se conserva el contrato común de [PLAN00](PLAN00-BASE-COMUN.md), con las excepciones de alcance aquí documentadas. Ninguna llamada de escritura a GitHub se ejecuta al redactar este plan.

## 1. Resultado solicitado y decisiones de producto

El usuario inicia la skill diciendo **«Alexa, abre mi agente»**. Dentro de la sesión puede pedir:

| Expresión del usuario | Significado confirmado | Resultado |
|---|---|---|
| «Crea la última PR subida» | Crear una PR nueva desde la última rama subida por el usuario conectado, si esa rama no tiene PR | Antigravity lee los cambios, redacta título/descripción y AlexIA publica una PR |
| «Haz merge a la rama {Rama}» | Seleccionar la última PR abierta del usuario o creada por AlexIA/Antigravity para él, comprobar el destino y esperar tests válidos | Squash and merge a la rama resuelta, si se cumplen las condiciones |

**Aclaración confirmada por el propietario en esta revisión:** el primer comando crea una PR desde una rama; no busca una PR abierta para crear otra copia ni completa título/descripción de una PR ajena. Conservar la expresión solicitada como alias y enseñar también la frase más explícita «Crea una PR de mi última rama subida».

Decisiones propuestas para completar los detalles que no vienen en la frase:

- Buscar únicamente en repositorios seleccionados al conectar GitHub y accesibles para ese usuario.
- Crear hacia `creation_base_branch` configurada por repositorio; ofrecer la rama predeterminada durante configuración, sin decidirla mediante IA.
- «Última rama» significa último push atribuible al usuario cuya evidencia tiene la integración, no rama con commit más reciente. Los límites de observación se explican en la sección 3.
- «Última PR» se ordena por `created_at`, no por comentarios ni `updated_at`. Si el propietario prefiere otro criterio, cambiar esta política explícitamente.
- Si la última rama seleccionada ya tiene una PR abierta, informar su número/estado y terminar sin duplicar ni modificar esa PR. No saltar a una rama anterior para lograr una creación.
- Para merge, la última PR se selecciona una sola vez y queda fijada durante la espera. Una PR posterior no reemplaza el objetivo del trabajo.
- Propuesta de seguimiento: permitir nuevos commits de esa misma PR durante la espera solo si el usuario configuró el grant `follow_head_updates`/version_policy=follow_ref; default snapshot sin seguimiento. Cada cambio invalida el gate anterior y exige comprobar otra vez el snapshot. El PUT final fija el SHA recién validado. Sin ese permiso, detenerse como stale_head; no pedir confirmación redundante cuando la política sí lo permite.
- El primer alcance soporta ramas dentro del mismo repositorio. PR desde forks y stacked PR requieren una extensión de contrato; no caer a otro mecanismo de merge automáticamente.

Fuera de alcance: modificar código para que los tests pasen, hacer push, cambiar la configuración de CI, aprobar revisiones en nombre de otros, modificar protecciones, force push, cambiar permisos o borrar la rama tras fusionar. Antigravity redacta metadatos; no recibe permisos para publicar o fusionar por su cuenta.

## 2. Invocación Alexa, intents y diálogo

### Entrada y separación entre palabra de activación y skill

La configuración debe distinguir palabra de activación del dispositivo de nombre de invocación de la skill. «Mi agente» es el nombre de invocación solicitado; «Alexa» corresponde a la activación si el dispositivo está configurado así. Comprobar disponibilidad y reconocimiento de ese nombre en el locale y dispositivo del propietario. No asumir que cambiar una frase del backend renombra una skill existente ni cambia la palabra de activación del Echo.

Recorrido previsto:

1. Usuario: «Alexa, abre mi agente».
2. LaunchRequest: «Puedes crear la PR de tu última rama o pedirme un merge indicando la rama».
3. La sesión escucha el segundo comando. Conservar además PreguntarIntent para consultas actuales.
4. Tras validar localmente identidad, conexión y comando, encolar un job selection y responder antes del presupuesto de Alexa: «Voy a buscar tu última rama para preparar la PR» o «Voy a buscar tu última PR y revisar el merge a desarrollo». Enumerar repositorios ocurre después del acuse; este no afirma que exista un candidato.
5. Cerrar la sesión tras el acuse, siguiendo la preferencia actual del proyecto de dejar de escuchar. El job sigue en segundo plano.
6. El resultado se consulta en el panel o con un intent auxiliar «Consulta el resultado de GitHub». Ese intent es soporte operativo, no una tercera operación de escritura.

No prometer que Alexa hablará espontáneamente cuando termine un job. Notificaciones proactivas requerirían un canal admitido y autorizado por separado. El acuse nunca dice «PR creada» o «fusionada» antes de comprobar el efecto en GitHub.

Alexa llama al CommandService interno con principal verificado, sin cookies de UI ni HTTP administrativo contra Vite. El padre selection devuelve su job_id antes de resolver proyecto. Después crea un hijo execution solo con ProjectRef real, snapshot y grant de efecto; el padre permite consultar/cancelar ambos. Ante ambigüedad, waiting_for_input conserva continuation_id vinculado a principal/revisión/TTL. UI o una nueva invocación continúa ese job; nunca usar un puntero de «último job» global ni aceptar un proyecto ficticio.

| Intent propuesto | Ejemplos | Slots |
|---|---|---|
| `CrearUltimaPrIntent` | «crea la última PR subida», «crea una PR de mi última rama», «crea mi última pull request» | Ninguno obligatorio |
| `MergeUltimaPrIntent` | «haz merge a la rama {Rama}», «haz merge con la rama {Rama}», «fusiona mi última PR con {Rama}» | Rama requerido; Proyecto opcional para aclaración |
| `ResultadoGithubIntent` | «consulta el resultado de GitHub», «cómo va la PR» | Job opcional, resuelto solo dentro del usuario |
| Ayuda/cancelación | Intents built-in correspondientes | No reutilizar CancelIntent para cancelar otro usuario/job |

Definir slot Rama con catálogo de ramas autorizadas y aliases por proyecto; evaluar entidades dinámicas donde se soporten. Nombres como `release/1.2`, guiones o mayúsculas necesitan resolución a un ref exacto. Una frase o alias que identifica dos ramas provoca aclaración; nunca convertir por intuición «desarrollo» a `main`. Si el nombre literal no se reconoce, permitir elegirlo en panel. Validar existencia con GitHub, no solo con el catálogo local.

Nuevo intents van al orquestador de trabajos. No enviarlos como preguntas libres al metaprompt de voz actual: exige una frase breve y no usar herramientas. La operación actual de consulta conserva su respuesta directa y su deadline; no sustituirla globalmente por jobs.

## 3. Identidad del usuario, propiedad y selección de lo último

### Cuenta humana y vínculo con Alexa

Instalar una GitHub App no identifica de forma suficiente al humano que habla. Añadir autorización de usuario de la App y consultar `GET /user` con su user access token. Guardar `github_user_id`/node ID estable y login para mostrarlo. Un cambio de login no debe convertir una PR propia en ajena ni apropiarse de otra.

Vínculo propuesto: `alexa_application_id + alexa_user_id` verificados -> principal AlexIA -> connection_id GitHub -> github_user_id. En instalación privada, emparejar al propietario desde UI. Para comunidad con skill compartida, diseñar account linking/autorización individual antes de habilitar escrituras. Nunca vincular por nombre hablado ni seleccionar la conexión del último usuario que abrió el panel.

### PR propia frente a ajena

Se admite una PR si:

```text
repository_id está autorizado y pertenece a la instalación correcta
AND
(
  pull_request.user.id == connected_github_user_id
  OR existe recibo confiable creado por AlexIA para ese usuario y esa PR exacta
)
```

Ser dueño del repositorio, revisor, assignee, autor de algunos commits o creador de una rama no convierte automáticamente una PR ajena en propia. La regla evita tocar PR de colaboradores aunque se encuentren en repositorios del usuario.

Antigravity no es una identidad GitHub universal. Si AlexIA crea con user access token, la PR queda atribuida al humano mediante la App. Si se usa installation token, el autor puede ser el bot: guardar `creation_receipt` con principal, connection_id, repo ID, PR node ID/número, job ID y actor real. No aceptar como prueba una etiqueta, un título o una línea de descripción que cualquiera pueda copiar. PR de bots de otros usuarios no se incluyen. Sin recibo verificable, pedir selección explícita y una política nueva, sin adoptarlas automáticamente.

CreationReceipt sigue retención independiente: mientras la PR siga abierta y propuesta de 30 días tras estado terminal. Expirar el job/reporte a 24 h no elimina procedencia. Efectos no reconciliados no se purgan por TTL. Reconectar otra identidad no importa recibos; desconectar a una persona no desinstala una App compartida ni revoca a otros usuarios.

### Evidencia de último push de una rama

La API de ramas no proporciona por sí sola una cronología fiable de pushes humanos. `commit.author.date`, `commit.committer.date` y `repository.pushed_at` no identifican por sí solos la última rama subida por esa persona. Tampoco basta un `git status` local para conocer todos sus repositorios conectados.

Diseño recomendado:

1. Suscribirse a eventos `push` de los repositorios autorizados mediante webhook validado. Conservar entrega, instalación, repo/ref, before/after SHA, `sender.id`, borrado/force push y momento de recepción.
2. Atribuir el push mediante actor autenticado de la entrega, no mediante email de commit ni `pusher.name`. Si el actor es un bot y no hay recibo de delegación, no atribuirlo al humano.
3. Mantener un índice `PushObservation` por conexión/usuario/repositorio/rama, con procedencia, cobertura y cursor. No almacenar cuerpos completos de commits innecesariamente.
4. Reconciliar con Events API del usuario autenticado cuando el token/instancia lo permitan. Filtrar `PushEvent`, actor ID y repositorios; no usar eventos «received» de personas seguidas como si fueran pushes del usuario.
5. La Events API tiene retraso y límites de historial; un webhook puede llegar tarde o fuera de orden. No ordenar por UUID de entrega, ni presentar la hora de recepción como prueba de hora real del push. Si dos observaciones no tienen orden fiable, queda ambiguo.
6. Tras conectar por primera vez, sin historial verificable, mostrar «No tengo una última subida confirmada; selecciona una rama o realiza un nuevo push». No inventar una a partir de la fecha del commit.
7. La UI muestra alcance observado desde la conexión y última sincronización. Ante huecos, entrega desordenada o evidencia desactualizada, el comando entra en `awaiting_selection`/`freshness_unknown` y solicita rama/proyecto. No prometer que el registro sabe la última acción global en tiempo real.

En operación normal, seleccionar el último push confirmado dentro del alcance conectado, excluyendo tags, ramas borradas y la propia rama base configurada. Esa exclusión es visible en configuración. Si el propietario pide explícitamente una rama, validar ese ref y su autorización en vez de usar el índice. Si un ref fue borrado o cambió de SHA después de seleccionar, invalidar el snapshot.

BC-07a es requisito de la observación automática G03, distinto de notificaciones. Guardar observación durable antes de responder satisfactoriamente al webhook. GitHub no reentrega automáticamente entregas fallidas: una caída/rechazo/pérdida del journal marca cobertura incompleta y exige reconciliación/redelivery autorizada o selección manual. Un 503 no garantiza nueva entrega. No elevar permisos de App para consultar entregas sin decisión explícita. awaiting_selection es stage de waiting_for_input; freshness_unknown es cause/result_code, no estado nuevo de job.

### Selección de la última PR para merge

Enumerar repositorios permitidos, listar PR abiertas ordenadas por creación descendente y filtrar con la regla de propiedad. Agregar y comparar fechas de creación de candidatos propios; las PR creadas para el usuario con recibo válido participan en el mismo conjunto. Un índice/search puede acelerar el descubrimiento, pero la PR final se revalida con su endpoint y IDs; search no es prueba suficiente de exhaustividad o autoría.

No omitir silenciosamente repositorios inaccesibles ni sustituir «última global» por «primera página de un repo». Si faltan páginas/permisos para decidir, pedir proyecto o informar alcance incompleto. Empates sin orden fiable se aclaran. Un borrador puede resultar ser la última PR: seleccionarlo y explicar que aún no está listo, sin fusionar en su lugar una PR más antigua.

Primero descubrir la PR propia más reciente, después comprobar soporte. Si usa fork/stacked PR/cola o está draft, informar su restricción; no excluirla para fusionar silenciosamente una anterior. Ninguna PR nueva durante la espera reemplaza el objetivo fijado.

## 4. Conexión, permisos y capacidades

GitHub App con instalaciones limitadas a repositorios elegidos y autorización individual de usuario. Las escrituras del piloto usan user access token para respetar la intersección usuario/App/repositorio. El token de instalación no se usa como alternativa para elevar privilegios cuando el usuario no puede operar. Modo bot solo si está configurado explícitamente con trazabilidad y política por usuario.

| Operación | Permiso GitHub a solicitar | Capacidad AlexIA independiente |
|---|---|---|
| Resolver candidatos sin proyecto conocido | Lecturas ya concedidas a la conexión | `github.targets.resolve`; no permite efectos |
| Leer ramas, commits, archivos y comparación | Contents: read | `github.branches.read`, `github.code.read` |
| Leer/crear PR y cambiar base | Pull requests: read/write según operación | `github.pull_requests.read`, `github.pull_requests.create`, `github.pull_requests.retarget` |
| Squash merge por REST | Contents: write según contrato del endpoint de merge | `github.pull_requests.squash_merge` |
| Leer checks | Checks: read | `github.checks.read` |
| Leer statuses | Commit statuses: read | `github.statuses.read` |
| Leer runs/jobs de Actions | Actions: read | `github-actions.runs.read` |
| Recibir push | Suscripción push y permisos de su contrato | `github.pushes.observe` |
| Reconciliar eventos privados del usuario | Events de usuario: read si la modalidad lo requiere | `github.user_events.read` |

Metadata y permisos implícitos se verifican en el contrato de la App. El permiso Contents: write necesario para merge es más amplio que esa operación: el núcleo restringe sus capacidades; Antigravity no recibe el token. No pedir Administration: write, Workflows: write o Actions: write para simular que todo pase. Consultar reglas activas mediante el endpoint soportado y reconocer que reglas de rulesets y protección clásica pueden necesitar consultas distintas. Si faltan datos para verificar un requisito, bloquear esa transición y explicar la limitación.

Al conectar:

1. Registrar App/OAuth, callback exacto y mecanismo local BYO o broker confidencial. Nunca distribuir la clave privada común o client secret en frontend/binario público.
2. Validar state/código y obtener identidad humana con user access token; asociar instalación y repositorios.
3. Seleccionar `creation_base_branch`, ramas destino permitidas, aliases y contrato de tests de cada repositorio.
4. Conceder por separado crear PR, retarget y squash merge. Mostrar efectos y alcance una vez; no pedir autorizaciones repetidas si el grant vigente ya cubre la orden concreta.
5. Configurar recepción verificada de pushes o modo manual con limitación de historial claramente visible.
6. Comprobar lectura, permisos y revocación en repositorio fixture. La configuración guardada no implica conexión verificada.

La orden de merge dentro de una identidad vinculada y una política preautorizada autoriza esa acción sobre el candidato/destino resueltos. Resolver ambigüedades de PR/rama antes de actuar; no añadir una confirmación universal extra si no existe esa ambigüedad o una exigencia real del repositorio. Esta revisión del plan no autoriza fusionar una PR real durante su redacción.

## 5. Flujo A: crear PR desde la última rama subida

### A1 — Resolver usuario y rama en job selection

Validar petición Alexa/principal/conexión antes de encolar; resolver permisos/cobertura fuera de la petición. Registrar o recuperar ProjectRef remote_repository por ID estable y owner, sin exigir checkout local. Tomar snapshot `repository_id + head_ref + head_sha + push_provenance + connection_id`, fijar base configurada y SHA, volver a autorizar efecto y crear hijo execution. No elegir repo por tener una PR pendiente: puede no existir ninguna todavía.

### A2 — Comprobar si ya tiene PR

Consultar PR abiertas para el head y mismo repositorio; procesar todas las coincidencias relevantes. Si existe una PR abierta de ese head, propia o ajena, devolver `already_exists` con enlace/autor y no crear ni modificar. Si hay varias bases/PR para el mismo head, pedir selección, no abrir otra automáticamente. Una PR previa cerrada/merged se muestra como antecedente y se exige diferencia nueva frente a base antes de considerar nueva creación.

Si head coincide con base, no hay commits nuevos o la comparación carece de cambios, responder `nothing_to_open`. No pasar a otra rama anterior para forzar una PR.

### A3 — Preparar snapshot de código para Antigravity

Consultar comparación base...head, commits y archivos relevantes a los SHAs fijados. Descargar texto permitido con límites y señalar omisiones. No ejecutar npm scripts, hooks Git, código del repositorio o tests para redactar título/descripción. No hacer checkout del proyecto personal activo ni alterar sus archivos.

Preparar carpeta/snapshot de trabajo de solo lectura con contexto mínimo. Excluir credenciales, `.env`, binarios y archivos fuera de alcance. Un directorio separado y el modo plan del CLI no son aislamiento suficiente; verificar controles de filesystem/red y herramientas disponibles del perfil antes de habilitarlo. La conexión de IA conserva únicamente su autenticación propia; el proceso no hereda tokens GitHub, credenciales cloud, SSH agent ni el vault.

### A4 — Generar título y descripción

Perfil técnico independiente de la voz. Entrada estructurada: repo, refs/SHAs, resumen de cambios, fragmentos y plantilla de PR seleccionada. La plantilla es contenido, no autorización para usar herramientas. No truncar silenciosamente una revisión para fingir que se leyó todo.

Salida propuesta:

```json
{
  "title": "Describe el cambio verificado",
  "body_markdown": "## Resumen\nCambios observados.\n\n## Pruebas\nNo ejecutadas por este flujo.",
  "analyzed_head_sha": "sha-head-demo",
  "analyzed_base_sha": "sha-base-demo",
  "coverage": "complete",
  "warnings": []
}
```

Validar JSON/esquema y límites de producto, por ejemplo título de 1 a 160 caracteres y body de hasta 32 KiB. Son límites propios propuestos, no una afirmación de límites GitHub. En cobertura insuficiente para redactar con sentido, marcar `needs_review` y no publicar automáticamente. Si la cobertura parcial es deliberada y autorizada, describirla en la PR. Nunca escribir «tests pasados» sin evidencia real.

Antigravity redacta; Rust valida y hace POST. El proveedor actual `generate(&str)` y su límite de 4.000 caracteres no representan una revisión de repositorio completa. Crear un contrato técnico separado o un constructor de contexto acotado con referencias/snapshot, sin aumentar indiscriminadamente el prompt de voz ni reutilizar conversaciones entre usuarios.

### A5 — Revalidar y publicar

Antes del POST, volver a leer head y base, comprobar que el usuario sigue autorizado y repetir búsqueda de PR existente bajo bloqueo local por `repo_id/head_ref`. Si cambia SHA, descartar el borrador. Esta primera entrega de creación usa grant snapshot: terminar stale_snapshot sin POST y permitir una nueva orden para analizar la versión nueva; no reanalizar y publicar bajo un grant ligado al hash anterior. Un modo futuro follow_ref para creación necesita contrato explícito de rama/base/efectos, no reutiliza por omisión follow_head_updates del merge. No publicar una descripción del commit anterior como si fuera actual.

Crear una PR normal, no draft, porque el usuario solicita una PR subida. El endpoint recibe refs, no una condición atómica de SHA del head: el bloqueo local no evita pushes de otras personas entre comprobación y POST. Tras respuesta, leer PR y comprobar head/base con los del análisis. Si hubo carrera, marcar metadata_stale e informar; no afirmar análisis actualizado ni habilitar merge usando ese borrador. Crear no concede por sí solo un grant de PATCH para reparar título/body: esa reparación futura requiere capacidad/efecto explícito y precondiciones de concurrencia. No cerrar una PR creada concurrentemente por otra persona ni editarla por atribuirse el intento.

```json
{
  "title": "Título validado por AlexIA",
  "head": "feature/demo",
  "base": "develop",
  "body": "Descripción validada con evidencia y estado real de pruebas.",
  "draft": false
}
```

Persistir `creation_receipt` saneado y devolver URL/número, autor real, refs/SHAs y cobertura. Un timeout del POST produce `unknown_outcome`; buscar PR por repo/head/base y evidencia del job antes de repetir. Una respuesta 422 no se interpreta automáticamente como duplicado: leer su causa y reconciliar.

Persistir EffectIntent antes del POST y recibo después de confirmar, conforme BC-02b. Reiniciar entre ambos obliga a reconciliar, no repetir. Misma orden Alexa devuelve el mismo padre/candidato/hijos/efecto aunque haya un push posterior; una nueva orden deliberada usa otro requestId.

## 6. Flujo B: squash merge de la última PR a una rama indicada

### B1 — Seleccionar y fijar la PR

Resolver candidatos según sección 3 en selection, registrar ProjectRef remoto real y fijar PR node ID/número, repo, connection/principal, autor/procedencia y head_sha. Reautorizar efecto/destino antes del hijo execution. Comunicar el objetivo; no sustituirlo por una PR posterior. Si no hay PR propia/trazada, terminar sin cambios.

Comprobar disponibilidad del guard C1 para ese repo/destino antes de enviar cualquier retarget o merge. Si sigue unverified, permitir diagnóstico de solo lectura y terminar blocked/destination_guard_unverified, sin PATCH/PUT. No cambiar primero la base de la PR para descubrir después que el merge está deshabilitado.

### B2 — Resolver la rama y alinear la base

Resolver {Rama} en el repositorio de la PR seleccionada. La rama debe existir, pertenecer al repositorio base, estar autorizada y ser distinta de head. Guardar ref exacto, no un alias mutable. El comando no crea ramas ni apunta a una rama homónima de otro repositorio.

El endpoint de merge fusiona a la **base actual de la PR**, no admite una rama de destino arbitraria en el body. Por tanto:

- Si `pull_request.base.ref == requested_base_ref`, seguir sin retarget.
- Si son diferentes y existe grant de retarget a ese destino, actualizar `base` de esa PR propia mediante PATCH y verificar la respuesta.
- Al cambiar base, invalidar el conjunto anterior de evidencias CI y revisiones dependientes del diff. Releer cambios, reglas y contexto; no suponer que los checks viejos sirven para el nuevo destino.
- Si no hay grant o la nueva base produce cambios inesperados, entrar en `awaiting_authorization`/`needs_review` con la diferencia concreta, no fusionar a la base antigua.

Cambiar base puede volver comentarios/revisiones obsoletos. Registrar old_base/new_base y no restaurar automáticamente la base si los tests fallan: una reversión también es un efecto y podría pisar trabajo concurrente. Mostrar que el retarget ya ocurrió y qué queda pendiente.

### B3 — Definir qué significa que todos los tests pasen

No basta `combined_status=success`, un color verde de pantalla o que Antigravity lo afirme. Construir una política verificable por repositorio/destino:

1. Requisitos GitHub vigentes: checks/statuses requeridos, revisiones, conversaciones y restricciones aplicables. El servidor sigue siendo la autoridad final para permitir merge.
2. Contrato de tests configurado: workflows/jobs/contextos esperados y emisores/app IDs permitidos. La configuración se define antes de automatizar; no descubrir un subconjunto conveniente después del fallo.
3. Evidencia de la versión correcta: head SHA, base ref/estado y commit de integración de prueba cuando lo use CI, junto con workflow/run/job IDs e intento vigente.
4. Todos los tests esperados terminan con `success`. `pending`, `queued`, `in_progress` o `mergeable=null` esperan dentro de plazo; `failure`, `cancelled`, `timed_out`, `action_required`, ausencia o cobertura desconocida bloquean.
5. GitHub puede aceptar `neutral`/`skipped` para algunos requisitos. Para cumplir la intención «todos los tests», AlexIA no los cuenta como pruebas ejecutadas con éxito. Un job deliberadamente no aplicable necesita regla explícita en el contrato; un job esperado omitido no pasa.
6. Tras push, reintento relevante, movimiento de base o retarget, volver a resolver qué evidencia corresponde a esa combinación; no reutilizar un run exitoso de otra base o intento antiguo que ahora falló.

Un repositorio sin tests definidos/configurados no cumple por conjunto vacío: devolver `tests_not_configured`. Un check desconocido no sustituye uno esperado por tener el mismo nombre. GitHub Actions no es el único emisor posible; incluir checks de terceros y commit statuses cuando proceda.

Si la política permite `follow_head_updates`, un nuevo push sobre la PR fijada mantiene el mismo objetivo PR/destino, reemplaza el snapshot y reinicia el gate dentro del plazo original. No prolongar indefinidamente el deadline ni reutilizar éxitos del SHA anterior. Si la política solo autoriza el snapshot inicial, terminar como `stale_head` sin fusionar la versión nueva. El hecho de seleccionar la última PR al inicio nunca permite pasar a otra PR.

Si CI no se lanza para ese evento/retarget, informar `tests_not_triggered`. No editar workflows, fabricar checks verdes ni dar permisos Actions: write silenciosamente. Dispatch/re-run solo si se añade una capacidad específica ya autorizada; el flujo básico observa las pruebas existentes.

### B4 — Espera y precondiciones de merge

Esperar en job independiente del request Alexa, mediante polling limitado y/o eventos verificados. Presupuesto propuesto: hasta 20 minutos por operación, polling inicial cada 10 segundos con backoff. Son valores de diseño a medir/configurar, no una garantía de tiempo. La autorización para esperar caduca; no dejar auto-merge indefinido tras timeout o cancelación.

Antes de fusionar:

- PR sigue abierta, no draft, propia o creada para el usuario con recibo confiable.
- Head coincide con el SHA que se comprobó; base coincide con la rama pedida y su contexto validado.
- Squash está habilitado en el repositorio; no cambiar settings para habilitarlo.
- Sin conflictos; mergeabilidad conocida; requisitos/revisiones y contrato de tests satisfechos.
- Usuario/App conservan permisos; ninguna revocación, cancelación o cambio relevante ocurrió.
- GitHub no exige un mecanismo distinto incompatible con este flujo, como una merge queue no soportada.

La primera implementación usa merge directo de PR normal. Si una rama exige merge queue o una PR es stacked, respetar la restricción y devolver `merge_mechanism_not_supported` hasta implementar su contrato oficial. No saltar la cola ni convertir el método a rebase/merge commit. Una cola futura necesita comprobar que su estrategia efectiva cumple squash.

**Condición de habilitación C1:** merge directo es mecanismo candidato. Mantener `destination_guard_status=unverified` y merge automático deshabilitado hasta comprobar un mecanismo server-side o restricción operativa efectivamente verificable que cumpla el destino frente a retarget concurrente. Relectura, lock local, casilla UI o auditoría posterior no demuestran esa condición. Si no puede garantizarse, devolver destination_guard_unverified sin enviar PUT; no debilitar el criterio ni sustituir squash por otra estrategia. Esta condición pendiente no bloquea selección, creación o diagnóstico CI.

### B5 — Ejecutar squash y comprobar efecto

```json
{
  "sha": "head-sha-validado",
  "merge_method": "squash"
}
```

Enviar a `PUT /repos/{owner}/{repo}/pulls/{number}/merge` desde el backend autorizado. `sha` protege contra cambio del head. HTTP 200 solo es éxito si la respuesta confirma `merged: true`; guardar el SHA devuelto y releer PR para verificar destino/estado. No asumir que el nuevo commit squash es el SHA del head: son objetos diferentes.

No enviar `base` a ese endpoint para intentar escoger el destino. El API documentado no ofrece una precondición `expected_base_sha/ref` equivalente al parámetro head `sha`. El lock del job tampoco bloquea retargets humanos en GitHub. Mitigar con exclusión local, relectura inmediatamente anterior, registro de cambios y coordinación del repositorio; documentar la carrera residual entre relectura y escritura. Si se exige garantía atómica del destino frente a actores concurrentes, no declarar cumplido ese criterio hasta contar con un mecanismo server-side/política verificada que lo haga posible. Una comprobación posterior detecta un efecto inesperado, pero no lo deshace.

C1 exige esa verificación para habilitar el flujo pedido, no la considera opcional por documentar la carrera. Antes de cualquier PATCH/PUT, revalidar grant, versión de política, expiración y cancelación bajo la misma exclusión que registra EffectIntent. Un efecto enviado se reconcilia incluso si después el usuario cancela; no presentar como cancelled un merge que ya ocurrió.

Manejar 403/405/409/422 por causa: denegar, esperar condiciones pendientes dentro del contrato, invalidar SHA o solicitar intervención. No reintentar el merge a ciegas ni elegir otra PR. Si se pierde la respuesta, consultar estado/merge commit: éxito previo del mismo objetivo se reconcilia, no se repite. El resultado incluye rama final y SHA squash. CI que corra después del merge se muestra aparte: no se prometen resultados futuros al decir que pasaron las pruebas previas.

## 7. Contratos, modelos y cambios de arquitectura

### Superficie GitHub propuesta

| Operación | Método/ruta | Verificación clave |
|---|---|---|
| Identidad humana | `GET /user` con user access token | ID estable, no instalación/bot |
| PR de repositorio | `GET /repos/{owner}/{repo}/pulls?state=open&sort=created&direction=desc` | Paginación y filtro author ID local |
| Detalle PR | `GET /repos/{owner}/{repo}/pulls/{number}` | Head/base/autor/node ID actuales |
| PR por head | Listado con filtro `head={owner}:{branch}` y estado | No confundir head de fork ni ignorar otras bases |
| Rama | `GET /repos/{owner}/{repo}/branches/{branch}` | Existencia real y ref codificado correctamente |
| Comparación | `GET /repos/{owner}/{repo}/compare/{base}...{head}` | SHAs fijados y cobertura parcial |
| Crear PR | `POST /repos/{owner}/{repo}/pulls` | Título/body/head/base/draft validados |
| Retarget | `PATCH /repos/{owner}/{repo}/pulls/{number}` con base | PR propia y evidencia anterior invalidada |
| Rulesets activos | `GET /repos/{owner}/{repo}/rules/branches/{branch}` | No sustituye por sí solo lectura de protección clásica |
| Checks/statuses | `GET /repos/{owner}/{repo}/commits/{ref}/check-runs` y `/commits/{ref}/status` | Emisor, intento y versión correctos |
| Revisiones | `GET /repos/{owner}/{repo}/pulls/{number}/reviews` | No contar todas las APPROVED históricas como vigentes |
| CI Actions | Endpoints de [PLAN05](PLAN05-GITHUB-ACTIONS.md) | Job/run/SHA/base e intento |
| Squash | `PUT /repos/{owner}/{repo}/pulls/{number}/merge` | `sha`, `merge_method=squash`, merged y commit |
| Pushes | Webhook firmado; Events API como reconciliación limitada | Actor, repo/ref, procedencia y cobertura |

Usar TLS, versión REST explícita, paginación y límites. GitHub por API es el mecanismo de operación; «abrir GitHub» significa consultar la cuenta/recurso autorizado y ofrecer el enlace, no automatizar clicks/login en su web. El navegador sirve para UI y evidencia de pruebas, no para eludir OAuth o cambiar el método de merge.

### Modelos internos propuestos

```text
ConnectedIdentity: principal_id, alexa_user_id, connection_id, github_user_id
PushObservation: provenance, delivery/event_id, actor_id, repository_id,
                 ref, before_sha, after_sha, received_at, source_time?, coverage
SelectionSnapshot: principal, connection, repository_id, ref/PR_id,
                   head_sha, base_ref, base_sha, selected_at, selection_policy
CreationReceipt: principal_id, connection_id, repository_id, pr_node_id,
                 pr_number, job_id, authenticated_actor_id, created_at
TestGate: policy_version, head_sha, base_ref/base_sha, integration_sha?,
          expected_tests, check_sources, workflow_runs, attempt, coverage
MergeReceipt: job_id, pr_node_id, expected_head_sha, requested_base_ref,
              observed_base_ref, method, merged_commit_sha, reconciled_at
```

Índices, EffectIntent y recibos siguen persistencia/retención diferenciada de PLAN00, no el TTL único del reporte. No introducir base de datos remota por conveniencia ni guardar tokens/código/cuerpos completos en recibos. Si se pierde procedencia de bot, no inventarla. Revocar conexión invalida grants/jobs. credential_refs_by_role separa user_access, user_refresh, app_private_key y webhook_signing; ningún token GitHub se transfiere a Antigravity.

### Jobs y estado

`POST /api/jobs` sigue las variantes de PLAN00. Selección autenticada sin proyecto conocido ni capacidad de escritura:

```json
{
  "kind": "selection",
  "integration_id": "github",
  "connection_id": "github-connection-demo",
  "capability": "github.targets.resolve",
  "input": {"operation": "create_pr", "selector": "latest_verified_user_push"},
  "idempotency_key": "verified-alexa-request-id"
}
```

Para seleccionar merge, usar la misma capability de lectura con operation=squash_merge, selector=latest_owned_or_tracked_open_pr y target_branch_alias=desarrollo. El servidor deriva usuario y repos autorizados; no acepta github_user_id del modelo. Tras resolver y reautorizar, el scheduler crea el hijo de efecto:

```json
{
  "kind": "execution",
  "integration_id": "github",
  "connection_id": "github-connection-demo",
  "project_id": "remote-project-demo",
  "parent_job_id": "selection-job-demo",
  "capability": "github.pull_requests.create",
  "input": {"selection_snapshot_id": "selected-snapshot-demo"},
  "idempotency_key": "selection-job-demo:create:1"
}
```

Los IDs ilustran registros reales requeridos; no se admiten sin comprobar. El hijo de merge usa `github.pull_requests.squash_merge` y destino/snapshot fijados. No se puede adjuntar un padre ajeno ni ampliar el permiso original. La misma clave/entrada devuelve padre/hijos existentes; distinto digest con la misma clave devuelve 409, no una nueva búsqueda.

Usar estados canónicos de PLAN00. stage resolving_target/analyzing/publishing/ready_to_merge/merging acompaña status running; awaiting_selection/awaiting_authorization usa waiting_for_input con continuation_id; waiting_for_checks usa waiting_for_external; reconciling es status. already_exists/nothing_to_open son result_code de completed/no_change. freshness_unknown/needs_review explica una espera o resultado blocked/partial, no crea permisos. Cancelación/revocación detiene nuevos hijos/efectos y reconcilia los ya enviados.

### Archivos previstos, no existentes todavía

| Archivo/módulo | Responsabilidad |
|---|---|
| `src/integrations/github/identity.rs` | Usuario OAuth, instalación y vínculo |
| `src/integrations/github/push_index.rs` | Evidencia de pushes y cobertura |
| `src/integrations/github/selection.rs` | Rama/PR propia y snapshot estable |
| `src/integrations/github/client.rs` | APIs HTTP y renovación |
| `src/integrations/github/pull_requests.rs` | Crear, retarget y reconciliar |
| `src/integrations/github/merge.rs` | Precondiciones, squash y recibo |
| `src/integrations/github/test_gate.rs` | Tests y políticas de destino |
| `src/integrations/github/webhooks.rs` | Validación y deduplicación |
| `src/providers/antigravity/review.rs` | Perfil técnico aislado para metadatos |
| `src/alexa/github_intents.rs` | Contrato de voz y slots |
| `src/alexa/service.rs` | Despacho de intents, conservando PreguntarIntent |
| `ui/src/integrations/github.js` | Conexión, ramas, permisos y trabajos |
| `tests/integrations_github.rs` | Contratos e identidad |
| `tests/github_pr_flows.rs` | Creación, selección y merge con fixtures |
| `tests/alexa_github.rs` | Voz, aislamiento y acuse |

Evitar meter todos los módulos en una sola tarea. La interfaz actual no ejecuta estas operaciones y no se habilita un botón hasta tener recorrido verificable. Actualizar documentación afectada al implementar.

## 8. Implementación por entregas y dependencias

### 01.1 — Identidad y selección antes de escribir

Dependencias: BC-01a/BC-02a/BC-03/BC-05a para tipos, jobs, identidad y catálogo fixture, sin conector real como prerrequisito. BC-02b durable antes de escrituras. BC-06a implementa ingreso/acuse con fixtures, G07/G10 handlers reales y BC-06b/G11 verificación posterior; no exigir estos últimos a G01. BC-04 técnico aplica a G05; BC-07a es requisito de G03.

| Tarea | Trabajo concreto | Archivos probables; tamaño | Aceptación y verificación |
|---|---|---|---|
| G01 | Autorizar humano y vincular Alexa | identity.rs, auth/linking.rs, tests; M | GET /user e IDs estables; otro Alexa user no accede a conexión |
| G02 | Registrar base/aliases/política de tests | settings de integración, UI y test; M | Destino inexistente/ambiguo rechazado; no secretos en API |
| G03 | Ingerir pushes verificables; requiere G01 + BC-07a | webhooks.rs, push_index.rs y tests; M | Journal, firma/delivery/actor y cobertura; frío/desorden/pérdida no inventan última rama |
| G04 | Seleccionar PR propia o trazada | selection.rs y tests; S/M | Terceros/bots ajenos excluidos; candidato fijado y exhaustividad controlada |

Checkpoint: no se permite POST/PATCH/merge hasta que identidad y selección pasen fixtures de dos usuarios y varios repositorios. Mantener aceptación pendiente en el tracker de implementación:

- [ ] G01–G04 completos, con permisos ajenos y frescura desconocida probados.
- [ ] Registro de decisiones de selección legible y sin datos sensibles.

### 01.2 — Crear una PR desde rama con título y descripción

| Tarea | Trabajo concreto | Archivos probables; tamaño | Aceptación y verificación |
|---|---|---|---|
| G05 | Snapshot y perfil Antigravity técnico | review.rs, context.rs y tests; M | Código leído sin credenciales GitHub, JSON válido y pruebas no inventadas |
| G06 | Crear/reconciliar sin duplicar | pull_requests.rs, client.rs y tests; M | Rama con PR no duplica; timeout y carrera de head se detectan |
| G07 | Exponer intent de creación y resultado | github_intents.rs, service.rs, fixture interaction model y tests; M | «Crea la última PR subida» encola al usuario correcto dentro del deadline |

Dependencias según grafo canónico: G05 consume G02/G03 + BC-04t y disponibilidad del perfil IA; G06 exige G01..G05 + BC-02b; G07 consume G06 + BC-06a. Los checkpoints de fase son aceptación, no un requisito de disponer de un conector ya terminado. BC-05b/BC-06b verifican después el recorrido real. Antes de integrar service.rs, definir el contrato. Tamaño objetivo de hasta cinco archivos por subtarea; dividir si se excede.

- [ ] PR real creada en repo fixture con autor, head/base y contenido esperado.
- [ ] Repetir orden o encontrar PR existente no duplica ni modifica PR ajena.

### 01.3 — Esperar tests y ejecutar squash al destino

| Tarea | Trabajo concreto | Archivos probables; tamaño | Aceptación y verificación |
|---|---|---|---|
| G08 | Retarget y gate de tests | test_gate.rs, pull_requests.rs, tests; M | Base nueva invalida evidencia; falta/fallo/skipped esperado bloquea |
| G09 | Squash condicionado y reconciliación | merge.rs, client.rs y tests; M | Destino verificado o deshabilitado; squash/SHA; 409/timeout no duplican ni cambian objetivo |
| G10 | Intent merge, espera y cancelación | github_intents.rs, jobs/github.rs, UI y tests; M | Rama exacta, acuse breve y job recuperable sin auto-merge infinito |
| G11 | Recorrido real y evidencia UI/voz | fixtures, tests y docs; M | Simulador Alexa y GitHub fixture acreditan cada efecto; CLI comprueba UI |

Dependencias: G04 y cliente de lectura de G01; G08 añade PLAN05 y no requiere disponibilidad runtime de IA. G09 exige G08 + BC-02b + guard de destino C1. G10 añade BC-06a; G11/BC-05b/BC-06b verifican el conjunto. Construir creación antes no convierte Antigravity en dependencia runtime del merge.

- [ ] Todos los escenarios de tests/rama/autoría críticos fallan cerrados.
- [ ] Squash real en rama desechable y recibo verificado.
- [ ] Mecanismos no soportados no hacen bypass ni cambian de método.
- [ ] Ningún test documental o de UI se presenta como integración funcional completa.

## 9. Configuración, límites, errores y recuperación

| Campo lógico propuesto | Uso |
|---|---|
| `github_user_id`, `connection_id`, `installation_id` | Identidad vinculada y permisos |
| `repository_ids` | Alcance explícito de búsqueda/efectos |
| `creation_base_branch` por repo | Destino del comando de creación sin slot |
| `merge_target_allowlist`, `branch_aliases` | Ramas que el usuario puede pedir por voz |
| `push_observation_mode`, `coverage_since`, cursor | Frescura y límites de «última rama» |
| `test_policy_version`, `expected_test_sources` | Definición previa de todos los tests |
| `allow_retarget`, `allow_create`, `allow_squash_merge` | Grants independientes |
| `review_context_budget`, `analysis_deadline` | Presupuesto técnico distinto del de voz |
| `checks_wait_deadline`, `authorization_expires_at` | Espera acotada y autorización vigente |
| `follow_head_updates` | version_policy=follow_ref solo con configuración explícita; default false/snapshot, siempre gate nuevo |
| `destination_guard_status` | unverified hasta demostrar la precondición; sin ella no habilitar merge automático |
| `user_token_ref`, `refresh_token_ref`, `webhook_secret_ref` | Referencias al vault, nunca valores en UI |

Propuestas iniciales a medir: 10 s por llamada GitHub, 120 s para redacción técnica, 64 KiB de salida técnica y una operación de escritura por repo/PR o head_ref. Mantener el presupuesto actual de Alexa para validar y acusar; nunca esperar allí el análisis/CI. Los límites actuales del adaptador de voz no equivalen a estos presupuestos futuros.

| Condición | Respuesta/estado esperado | Efecto permitido |
|---|---|---|
| Sin cuenta o vínculo | Requiere conexión/vinculación en panel | Ninguno |
| Última subida desconocida | Elegir rama o sincronizar, explicar alcance | Ninguno |
| Rama ya tiene PR | already_exists con enlace y autor | Ninguno; no reescribir |
| Sin cambios frente a base | nothing_to_open | Ninguno |
| Redacción inválida o incompleta | needs_review / provider_error | No publicar metadatos inventados |
| Tests pendientes | waiting_for_checks hasta deadline | Lectura; no merge |
| Tests fallidos/ausentes | blocked_checks / tests_not_configured | No merge |
| Base modificada | Invalidar gate y reevaluar | Solo retarget autorizado ya registrado |
| Head modificado | Invalidar selección/evidencia y reconciliar | No fusionar SHA no validado |
| Guard de destino sin verificar | destination_guard_unverified | No enviar PUT; creación/lectura siguen disponibles |
| Revocación/cancelación | Parar nuevas escrituras | Reconciliar si efecto ya fue enviado |
| Timeout tras POST/PUT | unknown_outcome | Consultar estado antes de repetir |

Cancelación detiene espera y trabajo futuro. Si GitHub ya recibió el merge, cancelar no lo revierte; comprobar resultado y comunicarlo. No deshacer un squash mediante reset/force push. Una reversión real requiere un nuevo trabajo explícito, normalmente un revert PR.

## 10. Seguridad, efectos y trazabilidad

| Riesgo | Control y evidencia requeridos |
|---|---|
| PR ajena seleccionada | Comparar author ID o recibo por usuario, no repo owner/assignee |
| Bot Antigravity de otro usuario | Recibo local verificable por conexión/PR exacta; no confiar en labels |
| Rama reciente inferida mal | Evidencia de push y cobertura/frescura; pedir elección si no alcanza |
| Código modifica instrucciones | Snapshot tratado como datos, capacidades fuera del modelo |
| Credenciales en proceso IA | Env mínimo, vault inaccesible y pruebas con secreto centinela |
| Tests antiguos o de otra base | Gate por snapshot/destino y fuente/attempt; invalidación en cambios |
| Método diferente a squash | Valor fijo squash y verificación de método permitido |
| Privilegios de administrador eluden políticas | No usar bypass; núcleo exige tests/revisiones aunque token pueda más |
| Carrera entre lectura y efecto | Guardas de SHA cuando existen, locks propios y límites atómicos declarados |
| Duplicado por reintento | Idempotencia lógica, consultas de reconciliación y resultado desconocido |

Auditar IDs, referencias de versión, efectos y tiempos. No registrar tokens, preguntas completas, cuerpos de webhook o código privado por defecto. En PR públicas, el título/body pueden divulgar información: el usuario selecciona ese repositorio y su visibilidad al configurar la política; no copiar secretos ni detalles de repositorios distintos.

No simular checks exitosos para probar en repositorios reales. Las fixtures de tests green/red pertenecen a un repositorio desechable y no sustituyen protecciones en el producto.

## 11. Matriz de pruebas y aceptación

### Casos de contrato y comportamiento

| Caso | Resultado exigido |
|---|---|
| Último push de A, commit con email de B | Selección por actor A verificado, no email |
| Push más reciente de B en repo de A | No pasa a ser última rama de A |
| Login de A renombrado | Conserva autoría por ID |
| Cuenta recién conectada sin historial | Pide selección; no ordena por fechas de commit |
| Pushes fuera de orden o mismo tiempo sin orden fiable | Ambigüedad/frescura desconocida, sin escritura |
| Rama elegida tiene PR propia | already_exists, cero POST |
| Rama elegida tiene PR ajena | No crea copia ni edita esa PR |
| PR ajena más reciente en repo propio | Se excluye del merge |
| PR bot con recibo de A frente a recibo de B | Solo A puede seleccionarla con su conexión |
| Nueva PR aparece mientras se espera CI | Continúa objetivo fijado, no cambia silenciosamente |
| Draft es la última PR | Informa no lista, no fusiona una anterior |
| Base actual difiere de Rama pedida | Retarget solo autorizado; invalidar tests anteriores |
| Tests verdes de head viejo/otra base | Bloquea merge |
| Un test esperado falla, falta, se cancela o se omite | No fusiona; resultado explícito |
| No hay tests configurados | Bloquea por configuración ausente |
| Emisor no autorizado imita nombre de check | No satisface gate |
| Conflicto, review requerida o merge queue | Respeta bloqueo; no bypass |
| SHA cambia antes de PUT | No fusionar versión no validada; manejar 409 |
| Cambio concurrente de base | Detectar límites del API; no afirmar garantía atómica inexistente |
| POST/PUT tuvo éxito pero respuesta se perdió | Reconciliar sin duplicar |
| Usuario cancela/revoca durante espera | No enviar nueva escritura |
| Reporte de 24 h expiró pero PR bot sigue abierta | Recibo de procedencia sigue vigente |
| Reinicio entre envío y respuesta | Reconciliar EffectIntent, no repetir POST/PUT |
| Orden duplicada después de otro push | Mismo padre/objetivo, sin nueva selección |
| Antigravity no disponible | Creación no disponible; merge/lecturas conservan dependencias propias |
| Búsqueda supera plazo Alexa | Acuse previo, selection fuera de la petición |
| Antigravity produce JSON inválido o afirma tests inventados | No publicar; validación/fallo explícitos |

### Prueba real de creación

Usar repositorio y ramas desechables autorizados. Conectar usuario A; generar push rastreable a feature/demo; un colaborador B también tiene una PR. Invocar en simulador Alexa con usuario vinculado. Verificar en GitHub PR nueva con head/base correctos, autor real y título/body basados en el snapshot. Repetir el comando: no aparece otra PR. Comprobar que la de B no cambia. Esto se ejecuta solo durante implementación autorizada, no en la redacción.

### Prueba real de merge

Workflow fixture con test controlado que primero falla y luego pasa sobre el snapshot correcto. Pedir «Haz merge a la rama desarrollo». Comprobar que no hay merge con fallo; cuando exista nueva evidencia válida dentro del alcance autorizado, efectuar squash solo sobre SHA reevaluado. Si se empujó nuevo código, invalidar el gate y aplicar la política de seguimiento al mismo objetivo: continuar solo si el grant lo incluye, o terminar stale_head si no lo incluye. Un fallo no crea permiso permanente. Verificar PR merged, destino y SHA squash, sin borrar rama.

Probar además destino distinto de la base original, retarget registrado y CI nuevo. La prueba de carrera de base debe dejar constancia de la ausencia de precondición atómica del endpoint y del tratamiento elegido. No realizar estas pruebas en producción.

### Alexa, UI y Playwright CLI

1. Probar el modelo de interacción en el locale real: «Abre mi agente», creación, rama con alias, nombre ambiguo y consulta de resultado.
2. Verificar firma/certificado/timestamp/skill ID existentes y vínculo de Alexa. Un request inválido no llega al selector ni a Antigravity.
3. Medir acuse dentro del deadline y job largo fuera de la petición. Probar cancelación y cierre de sesión.
4. Usar la skill Playwright CLI instalada para conectar la UI local de prueba y revisar identidad/repo/base/aliases/grants.
5. Comprobar estados selecting/analyzing/publishing/waiting/merged y enlaces/SHAs; pruebas UI con mocks se rotulan como simulación.
6. Comprobar que ya existe, PR ajena, tests fallidos, espera vencida y revocación no habilitan merge por error.
7. Abrir evidencia de PR real fixture y contrastar efecto remoto. Playwright sobre UI no demuestra por sí solo firma Alexa, actor GitHub, lectura del CLI o un merge real.

Tests futuros: `tests/integrations_github.rs`, `tests/github_pr_flows.rs` y `tests/alexa_github.rs`, más contratos ampliados de PLAN05. Ejecutar Cargo checks/test pertinentes y build de UI cuando se implemente. No afirmar que esos archivos o recorridos existen actualmente.

## 12. Entrega, reversión y decisiones pendientes

Activar primero la lectura/vinculación, después creación y finalmente merge para repositorio desechable. La separación de fases no elimina las dos funciones solicitadas del objetivo final; evita declarar terminado el plan con solo una tarjeta o búsqueda.

Definición de completado: usuario vinculado puede crear una única PR desde su última rama confirmada sin duplicar; se excluyen PR ajenas; Antigravity redacta con evidencia; el comando de merge resuelve destino, exige todos los tests definidos y ejecuta squash bajo permisos/restricciones. Errores, revocación, falta de evidencia y efectos desconocidos quedan comprobados. Medir límites de tiempo, compatibilidad y los límites de concurrencia real del API.

Desactivar capacidades impide nuevas escrituras. Cancelar jobs propios y revocar tokens. PR ya creadas/retargets/merges son efectos externos: no se deshacen al desconectar ni al volver al binario anterior. Conservar recibos saneados durante el plazo acordado y ofrecer recuperación explícita.

Pendientes de implementación/configuración, sin impedir definir este plan:

- Confirmar que «mi agente» funciona como nombre de invocación en el locale del dispositivo; no configurar Amazon en esta entrega.
- Elegir custodia OAuth BYO/broker y recepción de webhook verificable; no provisionar infraestructura por redactar el plan.
- Elegir bases de creación, aliases y destinos por repo; por defecto ofrecer su default branch, sin activar merge a todas las ramas.
- Fijar tests esperados y fuentes, requisitos de reviews y deadline por repo.
- Resolver la garantía necesaria frente a retarget concurrente y soportes de queue/stacked PR antes de prometerlos.

La garantía de destino es condición de habilitación C1 de G09, no una nota satisfecha con relectura local. Hasta comprobarla, no declarar merge automático terminado. La auditoría distingue contratos documentales corregidos de esa condición abierta.

Historial: R1 proponía solo lectura y dejaba creación/merge fuera de la primera entrega. R2 incorpora la solicitud del propietario y su aclaración de crear desde rama sin PR. R1 queda sustituida en ese alcance; no se borran las guardas de identidad, permisos o seguridad del endpoint.

Validación de R2: revisión documental, enlaces, ejemplos y coherencia con PLAN00/PLAN05. La evidencia anterior de 41 comprobaciones corresponde a la UI/API existente; no acredita estos nuevos flujos. Esta revisión no llama APIs GitHub, no cambia la skill Amazon y no modifica código ejecutable del producto.

Resultado mecánico de esta revisión: [GITHUB-R2-DOCUMENT-CHECK.json](evidence/GITHUB-R2-DOCUMENT-CHECK.json). Comprueba numeración, estructura, JSON, capacidades declaradas, enlaces locales, ausencia de patrones de credenciales y conservación de la configuración original.

## 13. Fuentes oficiales consultadas

Consulta: 2026-11-03. Los contratos y límites externos se revalidan al implementar. El comportamiento de selección, identidad y jobs es diseño propio de AlexIA. No reproducir tokens o datos privados de las fuentes en fixtures.

- [Identidad mediante user access token de GitHub App](https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/authenticating-with-a-github-app-on-behalf-of-a-user).
- [Usuario autenticado](https://docs.github.com/en/rest/users/users#get-the-authenticated-user).
- [Permisos de GitHub Apps](https://docs.github.com/en/apps/creating-github-apps/registering-a-github-app/choosing-permissions-for-a-github-app).
- [PR: listar, crear, actualizar y merge](https://docs.github.com/en/rest/pulls/pulls).
- [Cambio de base y efectos sobre revisiones](https://docs.github.com/en/pull-requests/how-tos/create-pull-requests/changing-the-base-branch-of-a-pull-request).
- [Comparación de commits](https://docs.github.com/en/rest/commits/commits#compare-two-commits).
- [Events API y límites de actualización](https://docs.github.com/en/rest/activity/events).
- [Payloads push](https://docs.github.com/en/webhooks/webhook-events-and-payloads#push).
- [Validación de webhooks](https://docs.github.com/en/webhooks/using-webhooks/validating-webhook-deliveries).
- [Reglas activas de rama](https://docs.github.com/en/rest/repos/rules#get-rules-for-a-branch).
- [Protección de ramas](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches).
- [Checks por referencia](https://docs.github.com/en/rest/checks/runs#list-check-runs-for-a-git-reference).
- [Commit statuses](https://docs.github.com/en/rest/commits/statuses#list-commit-statuses-for-a-reference).
- [Reviews de PR](https://docs.github.com/en/rest/pulls/reviews#list-reviews-for-a-pull-request).
- [Intents, utterances y slots Alexa](https://developer.amazon.com/en-US/docs/alexa/custom-skills/create-intents-utterances-and-slots.html).
