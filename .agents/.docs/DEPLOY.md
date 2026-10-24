# AlexIA — Validación y despliegue

Fecha: 2026-10-23. Procedimiento propuesto; no implica despliegue realizado. Dependencias: [ARCHITECTURE.md](ARCHITECTURE.md), [BACKEND.md](BACKEND.md), [INFRASTRUCTURE.md](INFRASTRUCTURE.md).

## Estado y condiciones previas

El repositorio contiene un esqueleto que bloquea Alexa y un proveedor Gemini sin configurar. No es V1 operativa. El README registra que no se confirmó compilación ni construcción Docker en el entorno anterior; esta edición documental tampoco acredita esas verificaciones.

Antes de habilitar V1:

1. Resolver el requisito de cuenta personal Gemini sin API key ni facturación con evidencia reproducible y condiciones admisibles. Si no es viable, conservar bloqueo y solicitar decisión sobre alternativa; no configurar una API de pago o con clave por sustitución.
2. Completar todos los controles Alexa y sus pruebas negativas; no existe bypass de desarrollo.
3. Definir y medir deadline total, límites de salida/contexto, TTL, tamaño de sesión y concurrencia; confirmar ventana de respuesta Alexa para el flujo elegido.
4. Elegir Render sin suspensión o VPS, presupuesto y región; cualquier creación/configuración externa debe corresponder a una solicitud expresa. Este procedimiento no la ejecuta.
5. Confirmar cuenta, dispositivo y locale para la Skill privada; guardar secretos únicamente fuera del repositorio.

## Validación local

Los comandos siguientes usan el nombre técnico actual `gemilexa`. Requieren Rust/Cargo compatibles con Cargo.toml y herramientas del sistema. Ejecutar en la raíz correcta:

```powershell
Set-Location C:\workSpace\AlexIA
cargo fmt --all -- --check
cargo check --all-targets
cargo test
```

Revisar resultados; no declarar éxito si falta una herramienta. Primera compilación genera Cargo.lock: revisarlo y versionarlo como aplicación, actualizar Dockerfile para copiarlo y usar `cargo build --release --locked`. A partir de ahí, usar `--locked` también en check/test para reproducibilidad. No actualizar dependencias automáticamente durante el despliegue.

```powershell
$env:BIND_ADDR = '127.0.0.1:3000'
cargo run
```

En otra terminal:

```powershell
Invoke-RestMethod http://127.0.0.1:3000/health
Invoke-WebRequest http://127.0.0.1:3000/alexa -Method Post -Body '{}'
```

Hoy health debe devolver `status=ok`, `alexa=blocked`, `gemini=not_configured`. El segundo comando debe fallar con HTTP 503 y `alexa_verification_not_implemented`; esto acredita bloqueo, no integración. Solo `BIND_ADDR` está implementada. `.env` no se carga automáticamente; las variables comentadas en `.env.example` no habilitan capacidades.

## Construcción y prueba del contenedor

```powershell
Set-Location C:\workSpace\AlexIA
docker build -t alexia:local .
docker run --rm --name alexia-local -p 127.0.0.1:3000:3000 alexia:local
```

La etiqueta `alexia:local` es libre; el ejecutable interno sigue siendo `gemilexa`. Repetir los smoke checks anteriores. No hay instrucción HEALTHCHECK en el Dockerfile actual: configurar la sonda en el alojamiento o añadirla con herramientas disponibles en runtime cuando se implemente. Verificar usuario sin root, HTTPS saliente con CA válidas, consumo bajo carga y salida limpia al detenerlo. Ctrl+C actual no demuestra manejo de SIGTERM; incorporarlo y probarlo antes de producción.

No incorporar `.env`, cookies ni tokens a la imagen. Antes de una entrega reproducible, fijar imagen base/digest, incluir lockfile, construir con `--locked` y etiquetar imagen con commit. Conservar una imagen anterior probada para reversión.

## Render: procedimiento pendiente

Crear/configurar el servicio solo cuando esté autorizado. Elegir servicio Docker que permanezca activo, revisar precio vigente y alinear puerto con `BIND_ADDR=0.0.0.0:3000` o la dirección explícita elegida. La aplicación actual no lee `PORT`; no asumir esa integración. Configurar health en `/health`, reinicio, registros y secretos autorizados, luego comprobar HTTPS público.

El [plan Free de Render](https://render.com/docs/free), consultado el 2026-10-23, tiene suspensión por inactividad y no satisface el objetivo 24/7. No afirmar que Alexa funcionará con baja latencia tras un arranque en frío ni crear tráfico artificial para evitarlo. Precio y plan concretos se revalidan al desplegar.

## VPS: procedimiento pendiente

Seleccionar tamaño después de medir y aprobar gasto. Preparar runtime de contenedores y proxy HTTPS, firewall mínimo, DNS/certificado si procede, configuración protegida y política de reinicio/arranque con el host. Publicar el backend solo a través de la ruta prevista; evitar exponer directamente el puerto interno. Configurar sincronización horaria, actualización del sistema y monitorización.

Levantar imagen por identificador inmutable, comprobar health local y HTTPS público, verificar logs sin contenido sensible y ensayar parada/reinicio. No usar un túnel temporal como despliegue final 24/7. Este documento no selecciona proveedor ni modifica servidores.

## Habilitar y comprobar la Skill privada

Configurar modelo de interacción, nombre de invocación, locale, permisos de prueba y endpoint HTTPS con el propietario. Verificar skill ID esperado. No publicar en el catálogo público ni afirmar distribución privada ya completada; comprobar cómo accede la cuenta y dispositivo autorizados.

Primero ensayar firmas y certificados válidos e inválidos, timestamp, cuerpo alterado y aplicación equivocada. Registrar que los inválidos nunca llegan al proveedor. Después probar lanzamiento, consulta, respuesta breve, aclaración, seguimiento con contexto, modos rápido/normal/detallado, cancelar y sesión vencida. Probar también timeout, cuota y expiración de autenticación Gemini. La prueba de conversación con Gemini exige un adaptador real validado, no un mock.

Medir tiempo completo en dispositivo y etapas del backend con muestra y concurrencia declaradas. Acordar objetivos numéricos antes de liberar y comprobar p95 con margen frente al deadline; no usar un valor supuesto como límite oficial Alexa. Toda respuesta hablada pasa por normalizador y usa PlainText.

## Entrega, observación y rollback

Registrar commit/imagen, configuración no secreta, proveedor/método de acceso validado, resultados de pruebas, latencia, límites y fecha. Comprobar ausencia de credenciales y de contenido personal en artefactos y logs. Observar inicialmente 24 horas, incluyendo un reinicio controlado y pérdida esperada de contexto RAM; no confundir esta muestra con garantía de disponibilidad.

Si falla seguridad, mantener o restaurar bloqueo de Alexa. Si falla proveedor, responder indisponibilidad controlada sin activar pago ni cambiar proveedor automáticamente. Si empeoran latencia/errores o recursos, volver a imagen y configuración anteriores verificadas, comprobar health y una conversación autorizada. Avisar de reinicio de sesión cuando corresponda; no existe backup de conversaciones.

## Criterios por versión

| Versión | Evidencia necesaria para entrega |
|---|---|
| V1 | Puerta Gemini resuelta; controles Alexa completos; conversación/seguimiento/modos reales; sesiones TTL aisladas; PlainText; métricas/health; build y contenedor probados; alojamiento activo y rollback ensayado. |
| V1.1 | Comparación de latencia y recursos, resiliencia y caché opcional probada con TTL/aislamiento; misma seguridad y presupuesto. |
| V2 | Adaptadores Hermes/OmniRoute u otros validados y pruebas de contrato comunes; selección explícita y costes/destinos aprobados; decidir escalado/persistencia solo si hacen falta. |

Toda capacidad incompleta debe figurar como pendiente en estos documentos y en el informe de entrega. Mantener sincronizadas instrucciones, documentación, configuración de ejemplo y pruebas cuando cambie el código.
