# AlexIA — Infraestructura

Fecha: 2026-10-23. Diseño objetivo; no se provisionan recursos con este documento. Ver [ARCHITECTURE.md](ARCHITECTURE.md), [BACKEND.md](BACKEND.md) y [DEPLOY.md](DEPLOY.md).

## Topología mínima aprobada

Una Skill privada Alexa conecta por HTTPS público con una instancia del backend Rust en contenedor. La instancia realiza conexiones salientes al proveedor validado y al servicio de certificados permitido. El almacenamiento de conversación es RAM con TTL; no se requiere volumen persistente, Supabase, PostgreSQL ni Redis en V1.

Una sola réplica evita repartir el contexto RAM entre procesos. Un reinicio, redeploy o caída elimina sesiones. No prometer continuidad entre invocaciones que no pertenezcan a la sesión vigente. Varias réplicas requieren una decisión posterior sobre afinidad o estado compartido, aislamiento y coste; no habilitar escalado horizontal antes de resolverlo.

## Render o VPS económico

| Opción | Ventaja | Coste operativo y límite |
|---|---|---|
| Render con instancia sin suspensión | Menor administración del servicio y despliegue de contenedor | Confirmar plan, región, recursos, costes y políticas vigentes; el servicio debe permanecer activo. |
| VPS pequeño | Control de proceso, red y recursos | Administrar actualizaciones, firewall, TLS, reinicios, monitorización y recuperación. |
| Render Free | Útil para prueba temporal | No cumple el objetivo de disponibilidad 24/7 por suspensión; arranque en frío afecta voz. |

La [documentación Render Free](https://render.com/docs/free), consultada el 2026-10-23, indica suspensión tras inactividad. Elegir un servicio siempre activo para V1 operativa. No usar pings artificiales como sustituto del requisito 24/7. El objetivo es bajo coste, no una promesa de gratuidad ni un SLA garantizado. No se fija proveedor, tarifa, región ni tamaño sin medir y aprobar el gasto.

Antes de elegir: estimar RAM del proceso y del máximo de sesiones, CPU para verificación y normalización, concurrencia esperada, latencia a proveedor/Alexa y tráfico saliente. Registrar coste mensual completo (cómputo, dominio si aplica, almacenamiento técnico y transferencia). Las condiciones del proveedor de IA son independientes del coste de hosting; no activar facturación automáticamente.

## Red y confianza

HTTPS con certificado válido es obligatorio para la ruta Alexa publicada. Terminar TLS en la plataforma o proxy del VPS y mantener backend interno en un puerto no expuesto innecesariamente. El único puerto/configuración actual de la aplicación es `BIND_ADDR`; no lee `PORT` ni convierte un valor automáticamente. En contenedor escucha en `0.0.0.0:3000`; al desplegar, alinear el puerto real y el esperado por el alojamiento.

En VPS publicar HTTPS por el proxy y limitar administración al propietario; el backend permanece en red interna/loopback según la topología. Definir y verificar salida a destinos necesarios. El certificado del endpoint y la firma de cada mensaje son controles diferentes, ambos obligatorios. No desactivar validación TLS para resolver errores.

Sincronizar reloj del host para timestamp y vigencia de certificados; monitorizar desviaciones. Acotar descargas de certificados contra SSRF como indica BACKEND.md. No permitir que contenido de una pregunta elija URLs de infraestructura, credenciales o destinos de proveedor.

## Recursos y disponibilidad

Configurar memoria, CPU, concurrencia, tamaño de cuerpo, límite de respuesta upstream y TTL antes de la entrega V1. Medir consumo bajo carga y dejar margen; no inventar una capacidad sin ensayo. Rechazar carga excedente de forma controlada en vez de crecer sin límite.

Conexiones HTTP reutilizadas, timeouts por etapa y deadline global reducen trabajo retenido. Ajustar pools y concurrencia conjuntamente: un pool no es un límite de peticiones. En shutdown dejar de aceptar tráfico y acotar drenaje. El código actual maneja Ctrl+C; añadir y probar SIGTERM para operación Linux antes de declarar cierre de contenedor correcto.

Política de reinicio del proceso, arranque al reiniciar host y health check deben configurarse en el alojamiento elegido. `/health` comprueba vida local y reporta integraciones bloqueadas/no configuradas en el estado actual. No consultar al modelo periódicamente desde health: añadiría dependencia, latencia y posible consumo. Readiness diferenciada, si se añade, debe tener contrato explícito.

## Datos, secretos y observabilidad

No incluir claves, cookies, tokens, contraseñas, certificados privados ni conversaciones en Git, imágenes o documentos. Configurar material autorizado en el gestor de secretos del alojamiento o variables protegidas; no pasarlo en comandos visibles ni build args. No crear credenciales para una integración aún no validada. Revisar rotación/revocación cuando se decida autenticación Gemini.

Sesiones RAM expiran y se acotan; no se respaldan. No loguear prompts ni respuestas por defecto. Logs estructurados y métricas agregadas describen duración, resultado, errores y recursos con retención limitada. Cualquier diagnóstico con contenido requiere decisión específica de privacidad y borrado. Métricas aún no implementadas; restringir su exposición y alertar por errores sostenidos, timeouts, RAM y caída del proceso.

Respaldar código y configuración no secreta con Git; guardar por separado únicamente configuración operativa autorizada. Un rollback restaura imagen/configuración compatible y pierde sesiones, no recupera RAM.

## Imagen y cadena de entrega

Ya existe Docker multistage: builder Rust y runtime Debian slim con usuario `gemilexa` sin root. No se ha probado la imagen en esta tarea documental. Antes de V1, fijar versiones/digests, generar y versionar Cargo.lock, construir con `--locked`, verificar certificados CA en runtime para HTTPS saliente y señal de parada. El Dockerfile actual aún usa etiquetas mutables y no copia lockfile.

Mantener `.dockerignore` y `.gitignore` sin secretos ni artefactos. Etiquetar imágenes por commit para rollback; no confiar únicamente en `latest`. No añadir infraestructura externa ni servicios de datos por conveniencia.

## Versiones, riesgos y aceptación

- V1: una instancia activa, HTTPS válido, proveedor validado, RAM/TTL acotados, métricas mínimas, health, reinicio y rollback ensayados. Observar inicialmente 24 horas y registrar reinicios, errores, latencia y consumo; pendiente de ejecutar.
- V1.1: optimizar capacidad y caché opcional con comparación medible. Aceptar solo si no mezcla datos ni supera presupuesto.
- V2: adaptadores nuevos y eventual escalado se evalúan separadamente; no asumir persistencia ni añadir Supabase por defecto.

Riesgos principales: suspensión del alojamiento, expiración de acceso Gemini, saturación, pérdida de sesiones, deriva del reloj y costes externos. Su aceptación requiere prueba de recuperación, límites medidos, alertas útiles y coste aprobado. Un health verde por sí solo no acredita conversación funcional ni disponibilidad 24/7.
