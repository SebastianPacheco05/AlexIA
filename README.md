# GemiLexa

Base en Rust de una skill comunitaria para consultar Gemini desde Alexa.
Software gratuito y de codigo abierto, licencia MIT, sin suscripciones,
cobros propios ni base de datos. Proyecto independiente, sin afiliacion a
Amazon o Google. No reemplaza el sistema de Alexa ni reproduce todas las
funciones de Alexa Plus: la interaccion futura sera mediante una skill.

El objetivo es facilitar consultas por voz, incluyendo usuarios de Colombia.
El alojamiento y la API de Gemini son servicios externos: sus cuotas gratuitas,
modelos elegibles y condiciones pueden cambiar. No se garantiza operacion sin
costos externos. No hay facturacion ni seleccion automatica de un plan de pago.

## Estado y contrato inicial

- `GET /health`: HTTP 200, `{"status":"ok","alexa":"blocked","gemini":"not_configured"}`.
  Indica que el servidor esta vivo, no que las integraciones funcionen.
- `/alexa`: cualquier metodo devuelve HTTP 503 y el error
  `alexa_verification_not_implemented`. Todas las solicitudes se bloquean,
  con o sin cabeceras de firma. No se interpreta el cuerpo ni se llama al proveedor.
- `LanguageModel`: interfaz asincrona desacoplada; `GeminiProvider` devuelve
  `ProviderError::NotConfigured`. Sin cliente HTTP, credenciales ni llamadas reales.
- Normalizador local de Markdown a texto plano: conserva texto y etiquetas de
  enlaces, descarta destinos de enlaces, imagenes, bloques de codigo y HTML crudo;
  compacta espacios y limita longitud por caracteres Unicode. Destinado a
  `outputSpeech.type = PlainText`, nunca a insertar directamente en SSML.
- Configuracion: solo `BIND_ADDR`, por defecto `127.0.0.1:3000`; IP y puerto
  obligatoriamente validos y puerto distinto de cero.
- Sin Supabase, almacenamiento, historial de conversaciones ni telemetria de consultas.

## Estructura

```text
Cargo.toml
src/
  main.rs                 Arranque y cierre con Ctrl+C
  lib.rs                  Rutas y health
  config.rs               Configuracion desde el entorno
  alexa/mod.rs            Bloqueo explicito de solicitudes
  providers/mod.rs        Contrato LanguageModel y error
  providers/gemini.rs     Proveedor aun no implementado
  voice.rs                Normalizador para voz
tests/contracts.rs       Pruebas de contratos y seguridad
Dockerfile                Compilacion multistage, ejecucion sin root
.env.example              Ejemplos publicos, sin secretos
LICENSE                   MIT
```

## Ejecutar en Windows

Requiere Rust estable con soporte de edition 2024 (1.85 o superior; se recomienda
el estable actual), Cargo y las herramientas de compilacion de C++ requeridas
por Rust MSVC. Instalacion oficial: https://www.rust-lang.org/tools/install

```powershell
Set-Location C:\workSpace\GemiLexa
$env:BIND_ADDR = '127.0.0.1:3000'
cargo run
```

En otra terminal:

```powershell
Invoke-RestMethod http://127.0.0.1:3000/health
# La siguiente solicitud debe fallar con HTTP 503.
Invoke-WebRequest http://127.0.0.1:3000/alexa -Method Post -Body '{}'
```

No hace falta crear `.env` para ejecutar el esqueleto. `.env.example` documenta
las variables; copiarlo a `.env` NO las carga automaticamente. Exportarlas en
la terminal o, para Docker, usar `--env-file`. No publicar archivos con secretos.
Las variables de Alexa se decidiran y configuraran despues con el usuario.

## Validacion

```powershell
cargo fmt --all
cargo fmt --all -- --check
cargo check --all-targets
cargo test
```

Las pruebas cubren health, bloqueo de Alexa incluso con cabeceras de firma
falsas y timestamps antiguos, proveedor sin configurar, normalizacion y
configuracion invalida. No necesitan credenciales, servicios externos ni red
al ejecutarse; Cargo si necesita descargar dependencias en el primer build.

Validacion durante la creacion: los comandos fmt/check/test se intentaron,
pero Cargo/Rust no estan disponibles en el entorno de trabajo. No se ha
confirmado compilacion, formato ni paso de las pruebas. Docker tampoco esta
disponible aqui, asi que la imagen no se ha construido.

`Cargo.lock` se generara en la primera compilacion. Debe incluirse en el control
de versiones de esta aplicacion. Despues, usar `--locked` en check/test/build,
copiarlo tambien en el Dockerfile y fijar versiones/digests de imagenes antes
de publicar una version reproducible.

## Docker

```powershell
docker build -t gemilexa:local .
docker run --rm -p 127.0.0.1:3000:3000 gemilexa:local
```

Si se usa `--env-file .env`, establecer `BIND_ADDR=0.0.0.0:3000` para escuchar
dentro del contenedor. La imagen final corre como usuario sin privilegios y
no incluye archivos `.env`, llaves ni herramientas de compilacion.

## Requisitos antes de habilitar Alexa

No hay bypass de desarrollo ni variable para desactivar seguridad. Antes de
reemplazar el bloqueo se deben implementar y probar conjuntamente:

1. Verificacion criptografica de la firma sobre los bytes originales del cuerpo.
2. Validacion de URL de certificado, cadena de confianza, vigencia e identidad
   esperada segun Amazon; descargas restringidas para evitar SSRF y redirects inseguros.
3. Validacion de timestamp con tolerancia de 150 segundos segun la guia de Amazon,
   rechazando solicitudes antiguas, futuras fuera de ventana o malformadas.
4. Validacion del skill/application ID configurado, limites del cuerpo,
   tiempos de espera y pruebas negativas de todos los controles.
5. HTTPS publico valido para la skill y configuracion conjunta con el usuario.

Luego se implementara Gemini sin acoplarlo a Alexa, priorizando un modelo
compatible con la cuota gratuita y sin cambios automaticos a servicios de pago.
Un prompt de respuesta breve puede complementar al normalizador: no sustituye
los controles de seguridad ni la conversion final a PlainText.

## Referencias

- Axum: https://docs.rs/axum/0.8/axum/
- Markdown: https://docs.rs/pulldown-cmark/0.13/pulldown_cmark/
- Seguridad del endpoint Alexa: https://developer.amazon.com/en-US/docs/alexa/custom-skills/host-a-custom-skill-as-a-web-service.html
- Cuotas y precios externos de Gemini: https://ai.google.dev/gemini-api/docs/pricing

Se aceptan contribuciones bajo licencia MIT. No incluir secretos en cambios,
issues ni ejemplos. No publicar como skill funcional hasta completar la
verificacion de solicitudes y la integracion real.
