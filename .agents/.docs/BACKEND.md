# AlexIA — Backend

Fecha: 2026-10-23. Diseño aprobado; diferenciar siempre objetivo y código actual. Alcance y versiones: [ARCHITECTURE.md](ARCHITECTURE.md). Operación: [INFRASTRUCTURE.md](INFRASTRUCTURE.md). Entrega: [DEPLOY.md](DEPLOY.md).

## Base y límites actuales

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

Medir verificación, espera de concurrencia, lectura de sesión, proveedor, normalización y tiempo total. Un deadline global domina los timeouts de conexión, certificados y generación; reservar margen de serialización y red. Cancelar trabajo al vencerlo. Contexto y salida acotados, pool de conexiones y proceso siempre activo son prioridades V1. Evitar llamadas remotas dentro de health.

No reintentar indiscriminadamente generación: puede duplicar uso o gasto. Un retry solo cabe dentro del presupuesto y con semántica conocida. Mensajes hablados para timeout, cuota o autenticación son breves y sin stack traces. Métricas agregadas: conteos, duración p50/p95, errores por categoría, sesiones activas y expulsiones; etiquetas de cardinalidad baja, nunca texto o IDs de usuario. No existe telemetría de consultas actualmente.

## Caché y validación

Caché de respuestas: opcional V1.1, apagada por defecto. Omitir consultas personales, contextuales o sensibles; incluir proveedor/modelo, versión del prompt, idioma y modo en la clave, además de la consulta elegible. TTL, tamaño y expulsión obligatorios. No compartir respuestas de una conversación entre usuarios ni almacenar errores como respuestas válidas. Medir hit rate y beneficio antes de activarla. Esta caché es distinta de reutilización de conexiones, sesiones y certificados.

Aceptación backend V1: pruebas negativas Alexa sin llamada al proveedor; contrato real Gemini tras validar acceso; follow-up y modos; expiración/aislamiento/concurrencia de sesiones; normalización final en todos los caminos; timeout con cancelación; health local y métricas sin contenido. V1.1 demuestra mejora y aislamiento de caché; V2 pasa la misma batería de contratos para cada adaptador. Las pruebas actuales solo cubren el esqueleto descrito, no estos criterios futuros.
