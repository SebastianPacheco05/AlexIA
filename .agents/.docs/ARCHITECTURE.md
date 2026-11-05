# AlexIA — Arquitectura y decisiones

2026-11-03, **implementación parcial** de PLAN00/PLAN01: módulos de proyecto,
sesión local, vault Windows, journal/jobs y selección GitHub. Catálogo y tareas
funcionan sin ALEXA_SKILL_ID. BYO GitHub App/device flow elegido para el piloto,
sin broker ni client secret distribuido. Creación y merge permanecen bloqueados
por aislamiento técnico y precondición de destino sin verificar. No se declara
completo ninguno de los planes. Decisiones concretas, límites y aceptación en
[el tracker](../../docs/integrations_plan/evidence/IMPLEMENTACION-PLAN00-PLAN01.md).
Las descripciones anteriores de esqueleto se conservan como historia.

Fecha: 2026-10-23. Estado: decisiones aprobadas; implementación parcial.

## Lectura y alcance

Configuración local: [UI.md](UI.md). Imagen y límites de despliegue:
[DOCKER.md](DOCKER.md). Ambas ampliaciones fueron solicitadas el 2026-10-25.

Este documento define alcance y evolución. [BACKEND.md](BACKEND.md) especifica contratos y voz; [INFRASTRUCTURE.md](INFRASTRUCTURE.md), recursos y operación; [DEPLOY.md](DEPLOY.md), validación y despliegue. Los cuatro viven en `.agents/.docs/`, con estos nombres exactos. Los documentos futuros deben enlazarse desde los existentes cuando corresponda.

AlexIA permite conversar por voz mediante una Skill privada Alexa y un backend propio. No sustituye Alexa ni implica afiliación con Amazon o Google. La privacidad de la Skill no elimina la autenticación del endpoint. La disponibilidad de idioma, cuenta, región y dispositivo se comprobará con la cuenta del propietario antes de declarar la experiencia operativa.

## Estado observado

El repositorio conserva el nombre técnico histórico `alexia`: paquete Cargo, importaciones, binario, usuario Docker y README. AlexIA es el nombre del proyecto; esta documentación no renombra esos elementos. El README aún contiene rutas `C:\workSpace\AlexIA` y la preferencia anterior por una API con cuota gratuita. La ruta actual es `C:\workSpace\AlexIA`; el requisito de acceso vigente es el descrito abajo. Esas referencias históricas no prueban una integración.

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

2026-10-25, aprobado e incorporado al frontend: Anime.js 4.5.0 para
transiciones y Lucide 1.61.0 para iconos, con versiones exactas en npm.
Motivo: el propietario solicita renovar todos los componentes visuales.
Se conserva Vite vanilla y Tailwind; no se añade React ni dependencias del
backend. npm audit completo: cero vulnerabilidades. Movimiento opcional según
preferencia del sistema. Implementación y validación visual en [UI.md](UI.md).

2026-10-25, implementado: interfaz Vite vanilla + Tailwind en `ui/`, iniciada
junto con Rust mediante `cargo run`. UI_PORT separado de BIND_ADDR; API de
administración exclusivamente local. `.env` y metaprompt se leen en runtime,
con guardado y sustitución del proveedor; puertos requieren reinicio. Docker
multietapa sirve assets compilados en modo static y conserva UI en loopback.
Sustituye el metaprompt incorporado exclusivamente al binario. Riesgos y
aceptación en UI.md y DOCKER.md; validación de contenedor pendiente de motor.

2026-10-25, implementado: el propietario sustituye la recuperación manual de
resultados por respuesta directa en la misma petición Alexa. Presupuesto local
7 s incluida verificación; timeout cancela la generación. El CLI se prepara
con el metaprompt antes de recibir preguntas, sin compartir conversaciones de
usuarios. Motivo: eliminar el segundo comando y el coste de arranque durante la
consulta. Riesgo: latencia/cuota y coste de memoria de dos procesos CLI; detalles,
fuentes y aceptación en [BACKEND.md](BACKEND.md#latencia-timeouts-y-errores).

2026-10-23: se documentan las decisiones aprobadas por el propietario y el estado del esqueleto. Acceso Gemini sin clave y facturación: pendiente de validación. Hermes/OmniRoute: preparación de interfaz aprobada, integración pendiente. Hosting concreto y presupuesto económico: pendientes. Cambios posteriores registrarán fecha, motivo, impacto, documentos afectados y pruebas; conservar el historial de decisiones sustituidas.


## 2026-11-03 — Planes de integraciones para comunidad técnica

Estado: **planes propuestos; conectores pendientes de implementación**. El propietario
solicita orientar AlexIA a una comunidad técnica, automatizar tareas de desarrollo y
conectar tanto herramientas locales como servicios externos. Se crean veinte planes
en la ubicación solicitada, con contratos transversales centralizados en
[PLAN00 — Base común e índice](../../docs/integrations_plan/PLAN00-BASE-COMUN.md).
Los archivos individuales siguen `PLAN##-APLICACION.md`; separan PowerShell/Bash y
Linear/Jira porque sus contratos y permisos son diferentes.

La propuesta conserva Rust y la consulta directa de Alexa. Plantea proyectos,
identidad, credenciales del sistema, capacidades, trabajos y ejecutores controlados;
no declara aprobada ni implementada infraestructura nueva, persistencia de jobs,
brokers OAuth o servicio SaaS multiusuario. Prioridad sugerida: validación local,
Gitleaks/OSV y lectura GitHub/Actions, antes de escritura remota o producción.

Motivo: convertir consultas técnicas en flujos verificables con evidencia. Consecuencias
y riesgos: nueva autorización por usuario/recurso, custodia de secretos, aislamiento
de procesos y efectos externos. Cada plan detalla prerequisitos, fases, aceptación,
pruebas y reversión; los presupuestos de recursos son propuestas sin medición.

La skill oficial Playwright CLI se instaló antes de crear los Markdown. La UI/API
actual se comprobó con backend real y configuración temporal sin credenciales:
41 comprobaciones correctas y una observación abierta por normalización CRLF/LF.
[Informe y evidencias](../../docs/integrations_plan/evidence/VALIDACION-2026-11-03.md).
No se implementó ningún conector, se conectó ninguna cuenta ni se modificó
infraestructura externa. `.env` y metaprompt originales permanecieron intactos.


## 2026-11-03 — GitHub R2: creación de PR y squash merge por Alexa

Estado: **requisitos solicitados; plan actualizado; implementación pendiente**.
El propietario solicita invocar «Alexa, abre mi agente», crear una PR desde la
última rama subida por el usuario conectado cuando no tenga PR y hacer squash
merge de su última PR —o una creada para él por AlexIA/Antigravity— a una rama
indicada, después de que pasen todos los tests definidos.

Se aclaró expresamente que crear significa abrir una PR nueva desde rama sin PR,
no reescribir una PR existente ni duplicar una ajena. La revisión
[PLAN01 R2](../../docs/integrations_plan/PLAN01-GITHUB.md) sustituye el alcance inicial
solo de lectura para GitHub; PLAN00 y PLAN05 quedaron sincronizados. Otros
conectores mantienen sus alcances. No se implementó código ni se hizo una
operación de escritura GitHub al revisar estos documentos.

Decisiones de diseño propuestas: identidad humana mediante user access token e ID
estable, vínculo con Alexa, índice verificable de pushes con frescura/cobertura,
recibos por usuario para PR de bot, redacción técnica Antigravity separada del
perfil de voz y publicación por backend. No inferir la última rama por fecha de
commit, ni que una PR es propia por ser dueño del repositorio. Sin evidencia de
último push, solicitar selección de rama.

El merge fija una PR y resuelve el destino; retarget requiere capacidad específica
y gate nuevo. Comprobar tests por snapshot/base/intento y respetar requisitos de
GitHub, sin bypass ni estrategia distinta de squash. El endpoint REST condiciona
head mediante sha pero no ofrece precondición equivalente de base: la revisión
documenta esa carrera residual y no promete atomicidad inexistente.

Aceptación y pruebas futuras se detallan en PLAN01: autoría, duplicados, PR ajenas,
CI ausente/antiguo/fallido, cambios concurrentes y reconciliación. Esta revisión es
documental. La prueba de 41 comprobaciones anterior acredita UI/API actual, no
estos nuevos recorridos ni configuración de la skill Amazon.


## 2026-11-03 — Consistencia C1 de PLAN00 y GitHub R2

Estado: **contratos corregidos; implementación pendiente; guard de destino aún no verificado**.
La auditoría cruzada identificó doce conflictos de diseño. Once se corrigieron en
PLAN00/PLAN01 y la dependencia PLAN05: selección asíncrona sin proyecto conocido,
proyectos remotos, idempotencia estable, retención de procedencia, estados y
continuaciones, canal Alexa interno/credenciales por rol, ingreso durable de eventos,
grants snapshot/follow_ref, dependencias acíclicas, readiness por capacidad y journal
de efectos con reconciliación. Detalle y criterios en
[la auditoría C1](../../docs/integrations_plan/evidence/PLAN00-GITHUB-CONSISTENCIA.md).

El grafo canónico vive en PLAN00; el cliente GitHub no depende de un conector ya
terminado, y lectura CI no exige la creación/merge completos ni Antigravity disponible.
BC-02b se completa antes de efectos reales, BC-07a antes de afirmar observación
automática de pushes y BC-04t antes de redacción técnica aislada.

Condición abierta C12: no se ha demostrado un mecanismo suficiente para garantizar
destino frente a retarget concurrente. destination_guard_status permanece unverified;
el flujo automático no envía retarget ni merge hasta verificarlo. Una relectura o lock
local no lo acredita. Esta condición no bloquea núcleo, selección, creación o CI.
No se declara resuelta por documentarla ni se cambia el requisito del propietario.

Validación C1: grafo de 25 nodos y 46 aristas acíclico, esquemas de ejemplos y enlaces
locales correctos. Es evidencia documental, no prueba runtime de integraciones.
No se modificó producto, configuración original ni infraestructura externa.
