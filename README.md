# AlexIA

2026-11-03: [PLAN00/PLAN01 — implementación parcial y guía de GitHub App](docs/integrations_plan/evidence/IMPLEMENTACION-PLAN00-PLAN01.md).
Núcleo de jobs y conexión/selección GitHub disponibles para piloto local; crear PR y squash merge permanecen bloqueados por las guardas pendientes. GitHub Desktop no es necesario.

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

La evolución hacia automatización para una comunidad de desarrolladores se detalla
en [el índice y la base común de integraciones](docs/integrations_plan/PLAN00-BASE-COMUN.md).
Incluye veinte planes individuales, dependencias, permisos, contratos propuestos,
pruebas y reversión. Son propuestas pendientes de implementación; el estado actual
de GitHub continúa siendo «Sin conectar». La [validación con Playwright CLI](docs/integrations_plan/evidence/VALIDACION-2026-11-03.md)
registra 41 comprobaciones correctas y una observación abierta sobre saltos CRLF
del metaprompt. No se conectaron cuentas ni se modificó infraestructura.

La [revisión R2 del plan GitHub](docs/integrations_plan/PLAN01-GITHUB.md) incorpora
los comandos Alexa para crear una PR desde la última rama subida sin PR y hacer
squash merge a una rama indicada cuando pasen los tests definidos. Los flujos
siguen pendientes de implementación; la prueba de UI anterior no los acredita.

- `GET /health`: HTTP 200, `{"status":"ok","alexa":"blocked","gemini":"not_configured"}`.
  Indica que el servidor esta vivo, no que las integraciones funcionen.
- `/alexa` sin `ALEXA_SKILL_ID`: cualquier metodo devuelve HTTP 503 y el error
  `alexa_verification_not_implemented`. Todas las solicitudes se bloquean,
  con o sin cabeceras de firma. No se interpreta el cuerpo ni se llama al proveedor.
- `LanguageModel`: interfaz asincrona desacoplada; `GeminiProvider` sigue
  deshabilitado para HTTP. `AntigravityProvider` permite consultas locales reales
  mediante la sesion de Google de Antigravity CLI, sin API key.
- Normalizador local de Markdown a texto plano: conserva texto y etiquetas de
  enlaces, descarta destinos de enlaces, imagenes, bloques de codigo y HTML crudo;
  compacta espacios y limita longitud por caracteres Unicode. Destinado a
  `outputSpeech.type = PlainText`, nunca a insertar directamente en SSML.
- Configuracion: `BIND_ADDR`, por defecto `127.0.0.1:3000`; IP y puerto
  obligatoriamente validos y puerto distinto de cero.
- Sin Supabase. El endpoint de Alexa conserva hasta ocho respuestas pendientes
  en memoria durante dos minutos, separadas por usuario. Antigravity CLI administra
  su propia sesion, historial, registros y politica de datos.

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
Set-Location C:\workSpace\AlexIA
$env:BIND_ADDR = '127.0.0.1:3000'
cargo run
```

En otra terminal:

```powershell
Invoke-RestMethod http://127.0.0.1:3000/health
# La siguiente solicitud debe fallar con HTTP 503.
Invoke-WebRequest http://127.0.0.1:3000/alexa -Method Post -Body '{}'
```

El servidor carga `.env` desde el directorio actual antes de iniciar Tokio. Las
variables ya exportadas en la terminal tienen prioridad. No publicar archivos
con secretos. Consulta [INSTRUCTIONS.md](INSTRUCTIONS.md) para la guia de usuario.

## Consultar Gemini localmente, sin API key

Requiere Antigravity CLI instalado y una sesion de Google iniciada con `agy`.
La integracion local se verifico con Antigravity CLI 1.4.7 en Windows.
En PowerShell, desde la carpeta del proyecto:

```powershell
$env:AGY_BIN = (Get-Command agy.exe -ErrorAction Stop).Source
cargo run --example ask -- "Que es una estrella?"
```

El ejemplo llama a `AntigravityProvider`, obtiene JSON y muestra texto plano
limitado a 800 caracteres para voz. Cada pregunta es independiente. No hace falta
arrancar el servidor HTTP ni introducir contrasenas o tokens en el proyecto.
Si `agy.exe` esta en PATH, `AGY_BIN` es opcional; tambien permite indicar su ruta
absoluta. Usar el ejecutable nativo, no un script `.cmd` o `.bat`.

La pregunta admite de 1 a 4000 caracteres. El CLI recibe un limite de 30 segundos;
Rust impone un limite total de 35 segundos y 64 KiB de salida. El adaptador inicia
el ejecutable directamente, sin shell, no continua conversaciones anteriores y
no imprime los mensajes internos del proveedor. Si falla la sesion o la cuota,
abrir `agy` directamente para diagnosticarlo. No hay reintentos automaticos ni
cambio automatico a una API de pago; el consumo depende del plan y los ajustes
existentes de tu cuenta de Antigravity.

El CLI trabaja en `target/antigravity-local`, separado del contexto habitual del
repositorio, con expansion de comandos desactivada y en modo plan. **El modo plan
no es un aislamiento completo ni deshabilita todas las herramientas.** En las
pruebas, `tools: []` en un agente personalizado tampoco elimino las herramientas
anunciadas por el CLI. El ejemplo `ask` es para preguntas locales del propietario;
el endpoint de Alexa se configura por separado.

Las credenciales permanecen bajo la administracion del CLI. No copiarlas al
repositorio, a una imagen Docker ni a este chat. `/health` sigue informando
`gemini: not_configured` cuando no se configura el endpoint de Alexa.
Render y la persistencia de la sesion de Google en Linux quedan pendientes de
validacion; ejecutar localmente no confirma que la conexion funcione en la nube.

Si Windows informa que `target/debug/AlexIA.exe` esta bloqueado porque el servidor
esta abierto, detenerlo con Ctrl+C o compilar en otra carpeta:

```powershell
cargo run --target-dir target/connection-check --example ask -- "Que es una estrella?"
```

Referencias de la integracion:
- [Autenticacion de Antigravity](https://antigravity.google/docs/install?tab=cli)
- [Modo no interactivo y formato JSON](https://antigravity.google/docs/cli/headless)
- [Procesos asincronos y cancelacion en Tokio](https://docs.rs/tokio/latest/tokio/process/index.html)

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

Validacion local actual: 28 pruebas Rust pasaron en Windows, incluido guardado
de configuracion, secretos redactados, control de origen y UI compilada.
La UI compila con Vite y npm audit reporta 0 vulnerabilidades.
Se comprobo una consulta real desde Rust y el flujo completo en el simulador de
Alexa: abrir la skill, preguntar y recuperar una respuesta real de Gemini mediante
el endpoint HTTPS local. Tambien se comprobo HTTP 400 para solicitudes sin firma
a traves del tunel. La prueba fisica en un Echo queda a cargo del usuario.
Docker Desktop esta instalado, pero su motor Linux no esta activo; la imagen
no se ha construido ni ejecutado en esta entrega.

`Cargo.lock` y `ui/package-lock.json` fijan las dependencias. Usar `--locked`
en Cargo y `npm ci` para reproducir la instalacion de la interfaz.

## Docker

```powershell
docker build -t AlexIA:local .
docker run --rm -p 127.0.0.1:3000:3000 AlexIA:local
```

Si se usa `--env-file .env`, establecer `BIND_ADDR=0.0.0.0:3000` para escuchar
dentro del contenedor. La imagen final corre como usuario sin privilegios y
no incluye archivos `.env`, llaves ni herramientas de compilacion.
Usar `UI_MODE=static` para servir el resultado de Vite sin servidor de desarrollo.
La administracion sigue en loopback; no se publica al mapear el puerto 3000.
Antigravity Linux y su autenticacion se provisionan aparte. Consulta
[DOCKER.md](.agents/.docs/DOCKER.md) para requisitos y limites reales.

## Interfaz de configuracion

```powershell
cargo run
```

Un comando inicia Alexa en `BIND_ADDR` (por defecto `127.0.0.1:3000`) y Vite
en `UI_HOST` / `UI_PORT` (por defecto `localhost:3001`). El frontend esta en
`ui/`, con JavaScript puro, Vite y Tailwind. Si faltan sus dependencias y Node
incluye npm, el arranque ejecuta `npm ci`; tambien puedes usar `npm --prefix ui ci`.

Abre [la interfaz local](http://localhost:3001). Permite editar el metaprompt,
el proveedor (actualmente Antigravity), modelo, razonamiento y variables de `.env`.
Guardar prepara el proveedor y aplica los ajustes sin recompilar. Los puertos y
`UI_MODE` se guardan para el siguiente arranque. Los valores de `.env` prevalecen
sobre los heredados del shell; el proceso no modifica variables globales en runtime.
Los secretos no se devuelven al navegador: un campo protegido vacio conserva
su valor. La API administrativa no existe en el servidor publico de Alexa.
Contrato, seguridad y pruebas en [UI.md](.agents/.docs/UI.md).
La sección Integraciones muestra únicamente GitHub como opción futura; todavía
no conecta cuentas, no solicita permisos ni consulta repositorios.

## Endpoint personal de Alexa y HTTPS local

Con `ALEXA_SKILL_ID` configurado, `app_with_alexa` habilita exclusivamente
`POST /alexa`. Sin ese ID se conserva el bloqueo original. El servidor valida
el cuerpo original con `tools/alexa-verifier/verify.cjs`, Node 22+ y OpenSSL 3.
El auxiliar verifica firma RSA-SHA256, cadena de confianza contra las CA de Node,
vigencia, restricciones de CA y SAN exacto `echo-api.amazon.com`; solo descarga
certificados de `https://s3.amazonaws.com/echo.api/`, sin redirects, con limites
de tamano y tiempo. Rechaza timestamps invalidos o fuera de 150 segundos en ambas
direcciones y exige el ID configurado tanto en context como en session, si existe.

No usa `ask-sdk-express-adapter`: al evaluar 2.14.0, npm audit encontro la alerta
sin parche GHSA-86w9-cpqp-85rv en node-forge. La implementacion nativa evita esa
dependencia. Las pruebas generan certificados temporales para comprobar cadena,
identidad y firmas alteradas.

Limites: cuerpo 64 KiB, dos verificaciones concurrentes, timeout de verificacion
5 segundos y una generacion concurrente. Los errores no imprimen cuerpos,
credenciales ni detalles de certificados. Alexa espera a Gemini y habla la
respuesta en la misma peticion, sin pedir `consulta resultado`. El presupuesto
local total es de 7 segundos desde el inicio del handler, incluida la verificacion;
si se agota, se cancela la consulta y se devuelve un aviso breve de timeout.
La sesion se cierra al terminar la respuesta, sin reprompt; no se guardan resultados pendientes.
El metaprompt se edita desde la interfaz y se guarda en `prompts/meta-prompt.txt`.
El servidor prepara un CLI con esas instrucciones antes de escuchar peticiones.
Cada pregunta consume una conversacion independiente y prepara otra de reserva,
sin historial de preguntas compartido. Hay una consulta de usuario simultanea
y, como maximo, una preparacion adicional del CLI en segundo plano.

```powershell
# Requiere ALEXA_SKILL_ID en .env y una sesion de agy ya iniciada.
.\scripts\start-alexa.ps1
# En otra terminal, con cloudflared instalado:
.\scripts\start-tunnel.ps1
```

Alexa escucha en `BIND_ADDR`, por defecto loopback:3000. Configura en Amazon la URL HTTPS del tunel
terminada en `/alexa`. `/health` informa `alexa: verification_enabled` y
`gemini: local_cli`; esto indica configuracion, no prueba una sesion o cuota activa.
Las instrucciones del certificado, reinicio y pruebas estan en INSTRUCTIONS.md.

El verificador no tiene dependencias de npm. Para probarlo en Windows:

```powershell
$env:ALEXA_OPENSSL_BIN = 'C:\Program Files\Git\usr\bin\openssl.exe'
node --test tools/alexa-verifier/verify.test.cjs tools/alexa-verifier/certificate.test.cjs
```

Esta conexion es para la skill personal en desarrollo. El CLI conserva las
herramientas y permisos descritos en la seccion local. Antes de publicar deben
resolverse el aislamiento del proveedor y la autorizacion de usuarios; la sesion
de Google en Render tambien requiere validacion independiente.

## Controles de Alexa

No hay variable para omitir la verificacion. El endpoint habilitado aplica:

1. Verificacion criptografica de la firma sobre los bytes originales del cuerpo.
2. Validacion de URL de certificado, cadena de confianza, vigencia e identidad
   esperada segun Amazon; descargas restringidas para evitar SSRF y redirects inseguros.
3. Validacion de timestamp con tolerancia de 150 segundos segun la guia de Amazon,
   rechazando solicitudes antiguas, futuras fuera de ventana o malformadas.
4. Validacion del skill/application ID configurado, limites del cuerpo,
   tiempos de espera y pruebas negativas de todos los controles.
5. HTTPS publico valido para la skill y configuracion conjunta con el usuario.

El flujo asincrono devuelve una confirmacion y permite recoger despues la
respuesta de Antigravity para evitar esperar a la generacion dentro del request.
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
