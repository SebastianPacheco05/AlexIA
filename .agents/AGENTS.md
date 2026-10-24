# AlexIA — Instrucciones para agentes

## Consulta obligatoria y alcance

Antes de planificar, programar, revisar o desplegar cambios en `C:\workSpace\AlexIA`, consultar `.agents/.docs/` y leer los documentos relevantes, incluidos los futuros. La ubicación de este archivo es `.agents/AGENTS.md`: conservarla. Sus instrucciones describen el flujo de trabajo de todo el proyecto; si una herramienta no descubre automáticamente instrucciones dentro de `.agents/`, cargar este archivo explícitamente al iniciar.

Documentos base, con nombres exactos en mayúsculas:

- `.agents/.docs/ARCHITECTURE.md`: decisiones, versiones, riesgos y aceptación.
- `.agents/.docs/BACKEND.md`: contratos, seguridad Alexa, proveedores, voz y sesiones.
- `.agents/.docs/DEPLOY.md`: validación, despliegue y rollback.
- `.agents/.docs/INFRASTRUCTURE.md`: recursos, red, alojamiento y operación.

No crear variantes de estos nombres ni una segunda fuente de verdad. Inspeccionar siempre contenido, instrucciones adicionales y cambios locales antes de editar; preservar instrucciones útiles y trabajo ajeno. La solicitud explícita vigente del propietario prevalece sobre decisiones históricas; ante contradicciones sustanciales no resueltas, señalar el conflicto antes de implementar la parte dependiente.

## Mantener documentación y código sincronizados

Todo cambio de decisión, comportamiento, contrato, dependencia, configuración, seguridad, pruebas o despliegue debe actualizar los documentos afectados en el mismo cambio. Revisar también documentos futuros de `.agents/.docs/`, sus enlaces y referencias cruzadas. Añadir documentos futuros al índice del documento pertinente y mantener nombres coherentes; no exigir que todo tema futuro encaje en los cuatro iniciales.

Registrar fecha, estado (aprobado/propuesto/pendiente/implementado/sustituido), motivo, consecuencias, riesgos y criterios de aceptación verificables. Conservar contexto de decisiones sustituidas. Distinguir siempre código observado, diseño objetivo y evidencia de validación. No declarar pruebas, builds, integraciones ni despliegues exitosos sin ejecutarlos y comprobarlos. Al terminar, informar archivos creados/editados, validaciones y pendientes; si un cambio no afecta documentación, explicar brevemente por qué.

## Guardas del proyecto

- Rust con Axum/Tokio/Serde; Reqwest/Rustls cuando se implemente transporte y proceda. AlexIA conserva por ahora los identificadores históricos `gemilexa`; no renombrarlos accidentalmente.
- Skill privada Alexa: firma, certificado, timestamp e ID esperado obligatorios. Mantener bloqueo hasta implementar y probar todos los controles; no añadir bypass.
- Gemini es la preferencia V1. Cuenta personal sin API key ni facturación sigue pendiente de validar: no afirmar integración oficial, usar endpoints privados por suposición, extraer cookies ni sustituir por un servicio con claves/cobros sin decisión explícita.
- Mantener proveedor desacoplado, preparado para Hermes/OmniRoute y otros; no asumir compatibilidad ni fallback automático.
- Voz con metaprompt y normalizador final hacia PlainText; contexto/salida acotados, conexiones reutilizables, timeouts, RAM con TTL y métricas sin contenido sensible.
- Sin Supabase inicialmente. Caché de respuestas opcional y apagada por defecto; nuevas persistencias requieren decisión documentada.
- No introducir secretos, conversaciones, credenciales ni material de autenticación en código, Git, documentos, logs o imágenes. No provisionar ni modificar infraestructura externa sin solicitud explícita.
- Respetar el alcance V1/V1.1/V2 y no presentar preparación de interfaz como integración terminada. Revalidar fuentes oficiales y condiciones externas antes de implementarlas o desplegarlas.

## Verificación final

Comprobar rutas, mayúsculas exactas, enlaces relativos, correspondencia con el código y ausencia de secretos. Para cambios de código ejecutar las pruebas pertinentes y registrar límites reales del entorno; para edición solo documental basta revisión de contenido, enlaces y diff. No debilitar pruebas o seguridad para cerrar un criterio pendiente.
