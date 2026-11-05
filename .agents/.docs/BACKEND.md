# AlexIA — Backend

2026-11-03, implementado: desconexión registra revoked y
last_error=disconnect_cleanup_pending antes de cancelar trabajos/borrar la
credencial. Solo limpia last_error tras completar ambas operaciones y persistir
el resultado. Permite reintentar sin reabrir acceso; begin_device rechaza nueva
autorización mientras quede limpieza pendiente. El catálogo conserva revoked
aunque la fecha del token haya caducado. Registros anteriores siguen siendo
compatibles, sin nuevo campo de almacenamiento. Motivo: deshabilitar la acción
de desconexión terminada sin ocultar fallos recuperables del vault/journal.
Prueba de regresión: fallo sintético de borrado, acceso/vínculo revocados,
reautorización bloqueada, reintento exitoso y estado conservado al reabrir journal.

2026-11-03: el catálogo GitHub incluye authorization_flow_version=2. La UI detecta
un backend anterior aún activo tras HMR y requiere reiniciar el lanzador antes
de permitir otra autorización. Evita repetir OAuth contra el binario antiguo que
descartaba el token sin instalación. No existe migración por lectura de secretos.

2026-11-03, corregido GitHub: autorización humana e instalación se validan por
separado. OAuth válido sin instalación conserva el user token en el vault y la
identidad con estado configured_unverified/last_error=app_installation_required;
no habilita repositorios/jobs. Verify comprueba también GET /user/installations
para la App exacta y conecta tras instalarla. Errores previos se conservan en
last_error (campo opcional compatible con registros anteriores). Sin identidad
guardada, verificar devuelve github_authorization_incomplete o el requisito de
instalación conocido antes de intentar leer un token inexistente. No se registran
tokens ni respuestas OAuth. Pruebas HTTP con identidad/instalación sintéticas.

2026-11-03, implementado parcialmente: `src/integrations/routes.rs` añade sesión
bootstrap/reanudación, catálogo/conexión/verificación/desconexión GitHub,
repositorios/vínculo Alexa y jobs/listado/detalle/cancel/input al listener local.
Las rutas nuevas de acciones exigen cookie HttpOnly/SameSite Strict y CSRF,
además de las guardas Host/Origin/marcador/forwarded existentes. Sesiones RAM
8 h, bootstrap de un uso/10 min; las rutas de settings conservan su contrato.
El router público no expone administración. No hay endpoint de artefactos.

CommandService encola sin búsqueda remota dentro de Alexa; intents de creación,
merge y resultado solo admiten skill/user explícitamente vinculados. Consulta
directa/firma/certificado/timestamp conservados. Readiness real: selección y
diagnóstico; sin POST de PR/PATCH de base/PUT de merge mientras sus guardas no
estén demostradas. Modelo de interacción entregado, no instalado en Amazon.
Gate puro y CI de lectura no acreditan todavía todos los requisitos del merge.
Contratos observados, límites, recuperación y pendientes en
[el tracker](../../docs/integrations_plan/evidence/IMPLEMENTACION-PLAN00-PLAN01.md).

2026-10-25, administración local ampliada: `GET /api/tunnel` devuelve
`{endpoint: string|null}` con la URL quick tunnel validada y `/alexa`, sin
mutar ajustes. Usa las mismas guardas de Host, Origin, marcador local y rechazo
de proxy que `/api/settings`; no existe en el router público. Lee latidos
efímeros de `target/tunnels/` con caducidad y puerto del listener activo.
El detalle del protocolo con el lanzador vive en [UI.md](UI.md#diseño).

Fecha: 2026-10-23. Diseño aprobado; diferenciar siempre objetivo y código actual. Alcance y versiones: [ARCHITECTURE.md](ARCHITECTURE.md). Operación: [INFRASTRUCTURE.md](INFRASTRUCTURE.md). Entrega: [DEPLOY.md](DEPLOY.md).

## Base y límites actuales

Actualización implementada 2026-10-25: `main` lee `settings::Repository` sin
mutar el entorno del proceso. `.env` prevalece sobre shell; metaprompt runtime
en `prompts/meta-prompt.txt`. `control` prepara el proveedor antes de persistir
un cambio y sustituye el servicio mediante RwLock; un fallo conserva el anterior.
Administración separada del listener Alexa, con Host/Origin locales, rechazo de
forwarded y cabecera propia contra CSRF; secretos redactados y revisión optimista.
Contratos actuales y límites en [UI.md](UI.md). El estado del esqueleto que sigue
se conserva como contexto histórico y queda sustituido en estas capacidades.

`src/main.rs` arranca Tokio y Axum, con cierre por Ctrl+C. `src/config.rs` valida `BIND_ADDR` como IP:puerto no nulo; predeterminado `127.0.0.1:3000`. Serde serializa respuestas. `src/lib.rs` sirve health y bloqueo Alexa. `src/providers/mod.rs` define `LanguageModel: Send + Sync`, `generate(&str)` y un futuro que devuelve `Result<String, ProviderError>`; el único error actual es `NotConfigured`. No hay autenticación Gemini, Reqwest/Rustls, sesiones ni métricas implementadas.

El diseño objetivo mantiene módulos para protocolo Alexa, aplicación/conversación, proveedores y voz. No crear servicios separados ni base de datos para V1. Reutilizar un cliente Reqwest por configuración de transporte, con Rustls cuando proceda, verificación TLS activa y límites de conexiones, respuesta y tiempo. No construir un cliente por consulta.

## Contratos HTTP

| Ruta | Hoy | Objetivo |
|---|---|---|
| `GET /health` | 200 y `{"status":"ok","alexa":"blocked","gemini":"not_configured"}` | Liveness local, sin llamada remota ni secretos; si cambia el esquema, actualizar pruebas y documentos. |
| `/alexa` | Todos los métodos: 503 y `alexa_verification_not_implemented` | `POST` autenticado que produce un envelope Alexa; rechazar otros métodos. No habilitar parcialmente. |
| Métricas | Sin endpoint | Instrumentación mínima en V1; si se expone endpoint, restringir acceso y documentar ruta antes de publicarla. |

Liveness no demuestra readiness del proveedor. Un proveedor sin configurar debe declararse indisponible y producir respuesta controlada; no simular éxito. Las solicitudes Alexa válidas requieren JSON compatible con el protocolo; un rechazo de seguridad usa el estado HTTP apropiado antes de generar voz.

## Seguridad Alexa obligatoria

Seguir la [guía oficial de Amazon](https://developer.amazon.com/en-US/docs/alexa/custom-skills/host-a-custom-skill-as-a-web-service.html), consultada el 2026-10-23. Verificar `Signature-256` sobre los bytes originales; no reserializar JSON antes de verificar. Restringir `SignatureCertChainUrl` a HTTPS, host `s3.amazonaws.com`, puerto 443 y ruta normalizada `/echo.api/`. Validar cadena X.509, vigencia y SAN `echo-api.amazon.com`. Rechazar redirecciones o destinos no admitidos; limitar descarga y caché de certificados a su vigencia.

Validar timestamp firmado con diferencia absoluta máxima de 150 segundos respecto a reloj UTC sincronizado. Rechazar faltantes, inválidos, antiguos y futuros fuera de ventana; firma o frescura fallida produce HTTP 400. Verificar además el application/skill ID esperado. Ningún fallo habilita fallback inseguro.

Añadir límites de cuerpo y concurrencia antes de trabajo costoso. Un verificador TLS de transporte no reemplaza al verificador de mensajes. Pruebas: cuerpo alterado, firma ausente/falsa, URL maliciosa, certificado inválido/caducado, ID distinto y bordes temporales. Mantener el bloqueo actual hasta aprobarlas conjuntamente.

## Proveedores y puerta Gemini

Gemini es el proveedor preferido V1, condicionado al acceso personal sin API key ni facturación. No se ha validado una vía oficial que cumpla esto; revisar la puerta completa en ARCHITECTURE.md. Las variables Gemini comentadas de `.env.example` son históricas y no se leen. No convertirlas en obligación ni afirmar acceso funcional.

Evolución propuesta del contrato interno: petición con mensajes, modo, locale, límite de salida, deadline y cancelación; respuesta con texto, proveedor/modelo y uso si el proveedor lo informa. Errores tipados para no configurado, autenticación expirada, cuota, timeout, no disponible y respuesta inválida. Aún no son tipos implementados: incorporarlos con pruebas de contrato cuando se implemente la capacidad.

El adaptador decide transporte y autenticación. El núcleo no depende del formato Gemini ni de uno OpenAI-compatible. Para Hermes/OmniRoute comprobar primero qué producto/endpoint se usará y sus capacidades; no asumir un protocolo común, credenciales compartidas ni compatibilidad de herramientas/streaming. V2 añade implementaciones del contrato sin reescribir Alexa ni el normalizador. No redirigir automáticamente consultas ni gasto a otro proveedor.

## Voz, modos y conversación

Metaprompt objetivo: «Responde en el idioma de la consulta, para escuchar por voz. Usa frases claras y naturales. Evita Markdown, tablas, código y URLs largas. Ajusta la extensión al modo indicado. Si falta información esencial, formula una pregunta breve. Reconoce incertidumbre y no inventes resultados». Especificar modo y límites desde configuración de confianza, separados de la pregunta del usuario. Este prompt no autentica solicitudes ni garantiza salida segura.

| Modo | Comportamiento objetivo |
|---|---|
| Rápido | Respuesta directa, breve y poco contexto; preferir baja latencia. |
| Normal | Explicación breve suficiente; predeterminado. |
| Detallado | Más contexto dentro del mismo deadline y límite de voz; ofrecer continuación si no cabe. |

Longitudes/tokens y presupuestos exactos se calibran con muestras reales antes de liberar V1. Detallado no permite respuesta ilimitada ni exceder el plazo Alexa. Un cambio de modo afecta la sesión y puede revertirse por voz. Para ambigüedad esencial, preguntar una sola aclaración y conservar contexto acotado; no generar preguntas de seguimiento innecesarias.

`normalize_for_voice` ya analiza Markdown con pulldown-cmark: conserva texto y etiquetas de enlaces, elimina destinos, imágenes, bloques de código y eventos HTML, compacta espacios y limita caracteres Unicode. El código inline queda como texto. No garantiza corte por oración ni tratamiento hablado de todas las tablas. La salida objetivo se coloca exclusivamente en `outputSpeech.type = PlainText`; no interpolarla en SSML. Si queda vacía, usar una frase breve de recuperación. Probar español, emoji, entidades, enlaces, listas, HTML, código, entrada vacía y límites de longitud.

La Skill objetivo gestiona lanzamiento, consulta, ayuda, cancelar/detener y fin de sesión. Mantener conversación mediante `shouldEndSession` y un reprompt útil; finalizar al cancelar. Los nombres de intents y el modelo de interacción aún deben diseñarse y probarse en el locale elegido. Un reprompt no amplía el plazo de la petición HTTP ya recibida.

## Sesiones RAM con TTL

Introducir en V1 un almacén en memoria compartido mediante estado Axum. Identidad compuesta por aplicación, usuario y sesión verificadas; no confiar en un ID proporcionado sin autenticar. Guardar solo turnos necesarios, modo y última actividad; sin persistencia ni historial global. El estado se pierde al reiniciar.

TTL por inactividad renovado tras interacción válida; verificar caducidad al leer y limpiar periódicamente. Configurar máximo de sesiones, turnos y bytes; expulsar entradas vencidas y aplicar política acotada al alcanzar capacidad. Evitar bloqueos durante llamadas de red y resolver peticiones concurrentes de la misma sesión con serialización o versionado. Cancelación/fin elimina contexto; una sesión vencida comienza limpia. TTL y límites aún no tienen valores implementados: fijarlos y probarlos antes de V1.

## Latencia, timeouts y errores

Actualización 2026-10-25. Estado: implementado por solicitud del propietario.
El endpoint verificado espera `LanguageModel::generate` en la misma petición y
devuelve la respuesta normalizada en PlainText, sin avisos ni comandos para
consultar resultados. Sustituye el flujo anterior de tareas en RAM y recuperación
con «consulta resultado»: se elimina ese almacén y no se retienen respuestas.
Una generación concurrente, sin cola. Actualización 2026-10-25, implementada por
petición del propietario: al terminar la respuesta de la consulta, la sesión se
cierra (`shouldEndSession: true`), sin reprompt. Sustituye la decisión anterior
de seguir escuchando. Motivo: dejar de escuchar después de contestar; otra
consulta requiere reabrir la skill. Aceptación: texto final, cierre y ausencia
de reprompt comprobados juntos en la prueba de respuesta directa.
El handler dispone de 7 segundos totales, incluida la verificación (máximo 5 s),
dejando margen frente a los aproximadamente 8 segundos de Alexa. Al vencer el
presupuesto se cancela el futuro del proveedor y su CLI mediante `kill_on_drop`,
se libera la concurrencia y se devuelve voz breve de timeout, sin reintento.
Motivo: respuesta automática sin un segundo comando. Riesgo: el CLI y la red
deben terminar dentro del presupuesto; no se promete éxito para toda consulta.
Aceptación: respuesta directa normalizada, timeout con liberación de permiso,
rechazo de concurrencia y solicitudes sin firma sin invocar al proveedor.
Referencia: [plazo de Alexa](https://developer.amazon.com/en-US/docs/alexa/custom-skills/progressive-response-api-reference.html),
consultada el 2026-10-25. La medición real se registra separadamente de las pruebas.

Optimización adicional 2026-10-25, implementada: metaprompt único en
`prompts/gemini-voice.txt`, incorporado con `include_str!`. El servidor espera
el evento `init` y un turno de confirmación del metaprompt antes de escuchar
peticiones; cada pregunta recibe una conversación recién preparada y la consume
una sola vez. Se prepara una conversación de reserva sin preguntas de usuarios
mientras se atiende la actual. Máximo dos CLI: uno para la pregunta y otro de
reserva/preparación; un solo usuario en generación. Solo se acepta el `result`
con `SUCCESS`, salida total por turno 64 KiB; EOF, JSON inválido o exceso fallan.
Timeout/cancelación elimina el proceso de la consulta; el reemplazo no recibe
esa pregunta. La espera de reserva consume el presupuesto HTTP restante.
La preparación agrega un turno mínimo de modelo por proceso y puede hacer
fallar el arranque si el CLI o la cuenta no están disponibles. No cambia modelo,
autenticación ni permisos. La herramienta CLI conserva su propia política de datos.
Fuente: [streaming oficial de Antigravity](https://antigravity.google/docs/cli/headless#stream-prompts-from-stdin).

Medir verificación, espera de concurrencia, lectura de sesión, proveedor, normalización y tiempo total. Un deadline global domina los timeouts de conexión, certificados y generación; reservar margen de serialización y red. Cancelar trabajo al vencerlo. Contexto y salida acotados, pool de conexiones y proceso siempre activo son prioridades V1. Evitar llamadas remotas dentro de health.

No reintentar indiscriminadamente generación: puede duplicar uso o gasto. Un retry solo cabe dentro del presupuesto y con semántica conocida. Mensajes hablados para timeout, cuota o autenticación son breves y sin stack traces. Métricas agregadas: conteos, duración p50/p95, errores por categoría, sesiones activas y expulsiones; etiquetas de cardinalidad baja, nunca texto o IDs de usuario. No existe telemetría de consultas actualmente.

## Caché y validación

Caché de respuestas: opcional V1.1, apagada por defecto. Omitir consultas personales, contextuales o sensibles; incluir proveedor/modelo, versión del prompt, idioma y modo en la clave, además de la consulta elegible. TTL, tamaño y expulsión obligatorios. No compartir respuestas de una conversación entre usuarios ni almacenar errores como respuestas válidas. Medir hit rate y beneficio antes de activarla. Esta caché es distinta de reutilización de conexiones, sesiones y certificados.

Aceptación backend V1: pruebas negativas Alexa sin llamada al proveedor; contrato real Gemini tras validar acceso; follow-up y modos; expiración/aislamiento/concurrencia de sesiones; normalización final en todos los caminos; timeout con cancelación; health local y métricas sin contenido. V1.1 demuestra mejora y aislamiento de caché; V2 pasa la misma batería de contratos para cada adaptador. Las pruebas actuales solo cubren el esqueleto descrito, no estos criterios futuros.


## 2026-11-03 — Contrato propuesto de trabajos GitHub por voz, R2

Estado: **propuesto; no implementado**. Fuente de requisitos y detalle técnico:
[PLAN01 R2](../../docs/integrations_plan/PLAN01-GITHUB.md). El propietario confirmó
creación desde última rama subida sin PR y solicitó squash merge a rama indicada
cuando se cumpla el contrato de tests. Esta solicitud sustituye la exclusión
inicial de merge para ese conector, sin habilitar infraestructura ni cuentas.

Intents nuevos propuestos: CrearUltimaPrIntent, MergeUltimaPrIntent con slot Rama
y ResultadoGithubIntent de soporte. LaunchRequest ofrece las dos operaciones
dentro de la skill con nombre solicitado «mi agente»; el nombre y locale se
verifican en Amazon. No cambiar la palabra de activación del dispositivo desde
el backend. PreguntarIntent conserva su respuesta directa vigente.

Cada orden GitHub verifica la petición Alexa y el vínculo de usuario antes de
resolver recursos. El handler acusa y cierra sesión dentro del presupuesto actual;
análisis y espera de CI se ejecutan como trabajos. No mantener una petición abierta
durante CI ni prometer voz espontánea al terminar. Cuenta GitHub humana por ID
estable y user access token; la instalación de App por sí sola no identifica al
hablante. PR bot requiere recibo verificable para ese principal.

Antigravity lee snapshot autorizado y devuelve título/body estructurados; no hereda
tokens GitHub ni decide permisos. El contrato técnico futuro debe separarse del
prompt breve y límite de 4.000 caracteres de la consulta de voz actual. Rust valida
y publica. Checks/statuses/Actions alimentan un gate por head, base y fuentes de
tests esperadas. Cambios de snapshot invalidan evidencia previa; seguimiento de
head solo bajo grant explícito para la misma PR/destino y plazo acotado.

No se modificó src/alexa/service.rs, el proveedor, routers ni modelo Amazon en esta
revisión. Las pruebas futuras, efectos, límites y recuperación viven en PLAN01 y
[PLAN05](../../docs/integrations_plan/PLAN05-GITHUB-ACTIONS.md); no interpretar estos
contratos como endpoints/intents existentes.


## 2026-11-03 — Contratos comunes corregidos por auditoría C1

Estado: **propuesto; no implementado**. PLAN00 es fuente canónica de estados/API;
PLAN01 concreta las operaciones GitHub. Se incorpora una variante selection de job,
acotada a conexión/usuario y sin efectos, que permite acusar la orden Alexa antes
de enumerar repositorios. El CommandService recibe principal verificado internamente,
sin simular cookie/cabecera de la UI. Después de seleccionar se registra ProjectRef
real, se revalida grant y se crea hijo execution, consultable por el padre.

Idempotencia: digest es dato de comparación y no componente de la clave única.
Una orden repetida conserva selección/hijos/recibos. Estados waiting_for_input y
waiting_for_external, stage y result_code se separan; continuation_id se liga a
principal/revisión/TTL. Snapshot es default; follow_ref solo por configuración explícita.

Antes de escritura externa se requiere journal durable EffectIntent y reconciliación
tras caída/cancelación. Procedencia de PR bot no expira con el reporte de 24 h.
Las credenciales se guardan por rol y la desconexión de un usuario no revoca a otros.
No habilitar retarget/merge automático con destination_guard_status unverified;
los controles server-side/operativos suficientes se deben demostrar, sin confundir
sha de head con precondición de base. Ver
[hallazgos y condiciones de salida](../../docs/integrations_plan/evidence/PLAN00-GITHUB-CONSISTENCIA.md).

La revisión comprobó documentos y grafo, no ejecutó operaciones remotas ni cambió
los handlers existentes. La validación Playwright histórica no prueba estos flujos.
