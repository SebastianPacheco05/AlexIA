# AlexIA — Arquitectura y decisiones

Fecha: 2026-10-23. Estado: decisiones aprobadas; implementación parcial.

## Lectura y alcance

Este documento define alcance y evolución. [BACKEND.md](BACKEND.md) especifica contratos y voz; [INFRASTRUCTURE.md](INFRASTRUCTURE.md), recursos y operación; [DEPLOY.md](DEPLOY.md), validación y despliegue. Los cuatro viven en `.agents/.docs/`, con estos nombres exactos. Los documentos futuros deben enlazarse desde los existentes cuando corresponda.

AlexIA permite conversar por voz mediante una Skill privada Alexa y un backend propio. No sustituye Alexa ni implica afiliación con Amazon o Google. La privacidad de la Skill no elimina la autenticación del endpoint. La disponibilidad de idioma, cuenta, región y dispositivo se comprobará con la cuenta del propietario antes de declarar la experiencia operativa.

## Estado observado

El repositorio conserva el nombre técnico histórico `gemilexa`: paquete Cargo, importaciones, binario, usuario Docker y README. AlexIA es el nombre del proyecto; esta documentación no renombra esos elementos. El README aún contiene rutas `C:\workSpace\GemiLexa` y la preferencia anterior por una API con cuota gratuita. La ruta actual es `C:\workSpace\AlexIA`; el requisito de acceso vigente es el descrito abajo. Esas referencias históricas no prueban una integración.

- Rust edition 2024, MSRV declarada 1.85; Axum, Tokio, Serde y pulldown-cmark en Cargo.toml.
- `GET /health` devuelve 200 con `status=ok`, `alexa=blocked`, `gemini=not_configured`.
- `/alexa` bloquea todos los métodos con 503 y `alexa_verification_not_implemented`, sin procesar el cuerpo ni llamar a un proveedor.
- `LanguageModel` existe; Gemini devuelve `NotConfigured`. Existe normalizador local para voz.
- Solo se lee `BIND_ADDR`. No hay sesiones, métricas operativas, caché de respuestas ni cliente HTTP saliente.
- Hay Dockerfile multistage y pruebas de contratos. Su existencia no acredita compilación, ejecución ni despliegue. No se encontró Cargo.lock.

## Decisiones y consecuencias

| Decisión aprobada | Motivo y consecuencia |
|---|---|
| Rust + Axum/Tokio/Serde | Un servicio pequeño, asíncrono y con tipos explícitos; mantener límites de concurrencia y evitar trabajo bloqueante en el runtime. |
| Reqwest + Rustls cuando proceda | Cliente HTTP reutilizable con TLS para proveedores y certificados. Aún no son dependencias; verificar compatibilidad antes de añadirlas. Rustls no implementa por sí solo toda la validación de mensajes Alexa. |
| Skill privada con firma y timestamp | Separar acceso de voz y proveedor; mantener el bloqueo hasta completar y probar la cadena de confianza. |
| Gemini como proveedor V1 | Es la preferencia funcional; la viabilidad del acceso solicitado es una puerta obligatoria de V1. |
| Interfaz multimodelo | Aislar autenticación, transporte, errores y capacidades de cada proveedor; preparar Hermes/OmniRoute y otros sin comprometer compatibilidad aún no comprobada. |
| Metaprompt de voz y normalización final | El prompt guía estilo; el normalizador garantiza salida apta para PlainText aunque el modelo ignore instrucciones. |
| Sesiones en RAM con TTL | Evitar base de datos inicial y reducir complejidad; aceptar pérdida de contexto al reiniciar. |
| Sin Supabase inicialmente | No hay necesidad validada de persistencia ni autenticación adicional; cualquier adopción exige nueva decisión. |
| Docker multistage y alojamiento económico 24/7 | Mantener artefacto portable; elegir Render sin suspensión o VPS tras medir y aprobar coste. |

## Puerta de viabilidad Gemini

El requisito solicitado es usar Gemini desde una cuenta personal sin API key y sin facturación. **Está pendiente de validar y no se afirma que exista una integración oficial que lo satisfaga.** Iniciar sesión en la web no demuestra acceso programático ni permiso para automatizarlo. Una cuota gratuita de API tampoco equivale al requisito de no usar API key.

Antes de implementar, registrar mecanismo y documentación verificable, condiciones de uso, permisos, tratamiento de datos, región, cuotas, renovación/revocación de sesión y comportamiento sin facturación. Probar acceso y recuperación tras expiración sin guardar secretos en Git. No extraer cookies, eludir controles ni adoptar endpoints privados por defecto. Si no se puede validar una vía admisible, V1 queda bloqueada en esta dependencia; conservar `NotConfigured` y presentar alternativas al propietario. Cambiar a API key, habilitar cobros o elegir otro proveedor requiere una decisión explícita, nunca una sustitución silenciosa.

La [documentación de autenticación Gemini API](https://ai.google.dev/gemini-api/docs/api-key), consultada el 2026-10-23, describe acceso mediante claves; no acredita el flujo personal sin clave solicitado.

## Flujo objetivo V1

Alexa → HTTPS → límites de entrada → verificación de origen y frescura → aplicación/conversación → sesión RAM → metaprompt + historial acotado → `LanguageModel` → normalizador → respuesta Alexa PlainText.

Las solicitudes inválidas terminan antes del proveedor. Cada petición tiene un presupuesto total que incluye certificados, generación y serialización. Health es independiente del proveedor. Sesiones y cachés se consultan solo después de autenticar; ninguna respuesta o excepción registra preguntas, historial o credenciales por defecto.

## Evolución y aceptación

La siguiente distribución concreta el orden de trabajo; no declara capacidades implementadas ni convierte objetivos provisionales de rendimiento en compromisos medidos.

### V1 — conversación privada segura

Incluye un proveedor Gemini validado, firma/timestamp/skill ID, entrada de consulta, metaprompt, PlainText, seguimiento dentro de una sesión RAM con TTL, modos rápido/normal/detallado, timeouts, conexiones reutilizadas, métricas mínimas, health y contenedor preparado para 24/7.

Criterios de aceptación:

1. Resolver y documentar la puerta de acceso Gemini; una implementación simulada no cumple este criterio.
2. Solicitudes falsificadas, caducadas, futuras fuera de tolerancia, de otra Skill o con certificados inválidos no llaman al proveedor. No existe bypass.
3. Probar pregunta inicial, seguimiento contextual, cambio de modo, ayuda, cancelación, expiración y cierre en simulador y dispositivo autorizado.
4. Todas las respuestas habladas usan PlainText normalizado y longitud acotada; un error produce una frase breve sin detalles internos.
5. Medir latencia total y por etapa, con p50/p95, errores y timeouts; comprobar que se responde antes del límite Alexa confirmado para el flujo elegido, con margen. El presupuesto numérico se fija antes de habilitarlo.
6. Verificar aislamiento de sesiones, límites de RAM, concurrencia acotada y pérdida de contexto previsible tras reinicio.
7. Validar build/pruebas, contenedor sin root, HTTPS y operación continua en el alojamiento elegido. Objetivo inicial de observación: 24 horas; no equivale a garantía SLA.

### V1.1 — eficiencia y robustez

Ajustar presupuestos con mediciones, TTL, límites y prompts. Añadir caché opcional, desactivada por defecto, solo si resulta útil; reforzar pruebas de carga, deduplicación y recuperación. Aceptación: comparar latencia y errores antes/después con la misma muestra, demostrar expiración y aislamiento de caché, y no debilitar seguridad ni privacidad. Ninguna mejora desbloquea por sí sola el acceso Gemini.

### V2 — expansión multimodelo

Implementar adaptadores Hermes/OmniRoute u otros tras validar sus contratos, autenticación y costes. Selección explícita de proveedor/modelo; fallback únicamente autorizado y observable. Persistencia o varias réplicas son opciones sujetas a necesidad demostrada, no compromisos de V2 ni autorización para añadir Supabase. Aceptación: pruebas comunes de adaptadores, errores normalizados, límites propios y ausencia de cambio silencioso de proveedor, coste o destino de datos.

## Riesgos y respuesta

| Riesgo | Tratamiento y evidencia requerida |
|---|---|
| Acceso personal Gemini inviable o inestable | Puerta de viabilidad antes de prometer V1; registrar evidencia y alternativa aprobada. |
| Latencia de red/modelo o arranque en frío | Medir por etapa, limitar contexto y salida, mantener proceso activo, cancelar antes del deadline. |
| Endpoint falsificado o URL de certificado maliciosa | Verificación completa y pruebas negativas; límites antes de descargas. |
| Fuga o mezcla de conversaciones | Sesiones aisladas, TTL, cuotas y registros sin contenido; pruebas de concurrencia. |
| Pérdida de RAM al reiniciar | Reiniciar conversación con mensaje claro; no prometer historial persistente. |
| Costes/cuotas o cambios externos | Revisar condiciones al desplegar, límites de uso, sin activación automática de pago. |
| Nombre histórico e instrucciones divergentes | Conservar identificadores funcionales; documentar discrepancias y exigir sincronización. |

## Registro inicial de decisiones

2026-10-23: se documentan las decisiones aprobadas por el propietario y el estado del esqueleto. Acceso Gemini sin clave y facturación: pendiente de validación. Hermes/OmniRoute: preparación de interfaz aprobada, integración pendiente. Hosting concreto y presupuesto económico: pendientes. Cambios posteriores registrarán fecha, motivo, impacto, documentos afectados y pruebas; conservar el historial de decisiones sustituidas.
