# Guía de uso de AlexIA

Este archivo es para la persona que instala y usa AlexIA. Consulta el
[README](README.md) para los detalles técnicos del proyecto.

## Qué funciona actualmente

- Consultar Gemini desde la terminal mediante Rust y Antigravity CLI, con una
  cuenta de Google y sin API key.
- Recibir respuestas en español y texto plano; el prompt pide una sola frase de máximo 25 palabras, con el dato esencial y sin introducciones.
- Iniciar el servidor local y comprobar su estado.
- Abrir la skill personal en el simulador de Alexa, enviar una pregunta y
  escuchar su respuesta automáticamente mediante el endpoint HTTPS local.

El servidor dispone de un endpoint verificado para una skill personal en
desarrollo. Crear la skill en Amazon no la conecta automáticamente con Gemini:
debes configurar el endpoint HTTPS siguiendo el apartado 6. El despliegue en
Render y la conservación de la sesión de Google allí también están pendientes.
Por ahora, las consultas se ejecutan en tu equipo: debe estar encendido y conectado.

El metaprompt de voz vive en `prompts/meta-prompt.txt`. Edítalo desde
[la interfaz local](http://localhost:3001) y pulsa **Guardar y aplicar**;
no requiere recompilar. La interfaz también modifica `.env`, modelo, nivel y
proveedor. Actualmente solo está disponible Antigravity.
Antigravity arranca y procesa esas instrucciones antes de que el servidor
escuche peticiones, de modo que al decir «abre mi asistente» ya hay una sesión
preparada. Cada pregunta usa una conversación nueva con solo esas instrucciones
previas; nunca se reutilizan preguntas de otro usuario. Se prepara otra sesión
de reserva mientras se atiende la consulta actual. Si preguntas muy seguido,
la espera de preparación también cuenta dentro del plazo de Alexa.

## 1. Preparar el equipo en Windows

Necesitas Rust 1.85 o superior, Cargo y Antigravity CLI. La conexión local se
probó con Antigravity CLI 1.4.7.

Abre **PowerShell** y comprueba las herramientas:

```powershell
cargo --version
rustc --version
Get-Command agy.exe
```

Si falta Rust, puedes instalarlo con:

```powershell
winget install --id Rustlang.Rustup -e
```

Instala también [Build Tools para Visual Studio](https://visualstudio.microsoft.com/es/visual-cpp-build-tools/)
con la opción **Desarrollo para el escritorio con C++**, incluyendo MSVC y Windows
SDK. Esto proporciona las herramientas que Rust necesita para compilar en Windows.

Si falta Antigravity CLI, sigue su
[guía oficial de instalación](https://github.com/google-antigravity/antigravity-cli#installation).
Al terminar las instalaciones, cierra y vuelve a abrir PowerShell. Si usas la
terminal integrada de VS Code, reinicia VS Code.

En los comandos siguientes se supone que el proyecto está en
`C:\workSpace\AlexIA`. Cambia esa ruta si lo guardaste en otra carpeta.
Cuando copies un comando, no incluyas la numeración del paso ni el texto
`PS C:\...>` de la terminal.

## 2. Iniciar sesión con Google

```powershell
cd C:\workSpace\AlexIA
agy
```

Completa el inicio de sesión en el navegador con la cuenta de Google que deseas
usar. Para comprobarla, escribe dentro de Antigravity:

```text
Responde en español, en una sola frase y sin usar herramientas: ¿qué es una estrella?
```

Cuando responda, escribe `/exit` para volver a PowerShell. No necesitas mantener
esta ventana de Antigravity abierta para consultar desde Rust.

No copies contraseñas, códigos de autorización ni tokens al proyecto o al chat.
Antigravity administra la sesión y sus propios registros e historial. Las cuotas
y el consumo dependen de tu cuenta y sus ajustes; tener un plan Pro no implica
consultas ilimitadas.

## 3. Consultar Gemini desde AlexIA

Desde PowerShell:

```powershell
cd C:\workSpace\AlexIA
$env:AGY_BIN = (Get-Command agy.exe -ErrorAction Stop).Source
cargo run --example ask -- "¿Qué es una estrella?"
```

La primera ejecución puede tardar mientras descarga dependencias y compila.
Cuando termine, aparecerá la respuesta en la terminal. Cambia el texto entre
comillas para hacer otra pregunta.

Cada consulta es independiente: no recuerda la pregunta anterior. No necesitas
iniciar el servidor con `cargo run` para usar este ejemplo. Tampoco necesitas
una API key ni crear un archivo `.env`.

La pregunta admite hasta 4000 caracteres y no puede estar vacía. La respuesta
para voz se limita a 800 caracteres. Si la consulta tarda demasiado, el programa
devolverá un error en lugar de esperar indefinidamente.

Esta versión se usa para preguntas locales introducidas por el propietario.
Antigravity se ejecuta en modo plan, que no deshabilita todas sus herramientas;
el ejemplo `ask` es independiente del endpoint de Alexa.

## 4. Iniciar el servidor local

El servidor carga `.env` al arrancar. Para usar Flash con razonamiento bajo:

```dotenv
GEMINI_MODEL=gemini-3.8-flash
GEMINI_LEVEL=low
```

Esto selecciona `gemini-3.8-flash-low` en Antigravity. Los niveles admitidos son
`low`, `medium` y `high`; la disponibilidad del modelo depende de tu cuenta.
Reinicia el servidor después de editar `.env`. Las variables exportadas en la
terminal tienen prioridad sobre el archivo. Esta selección se aplica al servidor
de Alexa; el ejemplo `ask` conserva el modelo predeterminado de Antigravity.

Este paso comprueba el servidor; es distinto de hacer una consulta a Gemini.

```powershell
cd C:\workSpace\AlexIA
cargo run
```

Deja esa terminal abierta. En otra terminal ejecuta:

```powershell
Invoke-RestMethod http://127.0.0.1:3000/health
```

El estado esperado es:

```json
{"status":"ok","alexa":"blocked","gemini":"not_configured"}
```

Este es el estado cuando no configuras `ALEXA_SKILL_ID`. `gemini: not_configured` se refiere
a la integración del servidor HTTP; las consultas locales del ejemplo `ask`
pueden funcionar al mismo tiempo. Sin el ID configurado, `/alexa` devuelve HTTP 503.
El servidor carga automáticamente `.env` desde la carpeta donde lo inicias;
las variables de PowerShell tienen prioridad sobre las del archivo.

Para detener el servidor, presiona **Ctrl+C** en la terminal donde lo iniciaste.

## 5. Crear la skill de Alexa

Abre la [consola de Alexa Developer](https://developer.amazon.com/alexa/console/ask)
con la misma cuenta de Amazon que usa tu dispositivo Alexa.

Pulsa **Create Skill** y selecciona:

| Campo | Opción |
| --- | --- |
| Skill name | `AlexIA` |
| Primary locale | El idioma y la variante que tiene configurados tu Echo |
| Experience | `Other` |
| Model | `Custom` |
| Hosting services | `Provision your own` |
| Templates | `Start from Scratch` |

Por ejemplo, elige **Spanish (MX)** si tu Echo utiliza Español de México.
Avanza con **Next**, revisa las opciones y pulsa **Create Skill**.
Espera a que termine de crearla.

En **Build → Invocation → Skill Invocation Name**, escribe:

```text
mi asistente
```

Guarda el cambio. La frase prevista para abrirla será **«Alexa, abre mi
asistente»**, cuando terminemos la configuración y la conexión.

En **Interaction Model → Intents**, crea `PreguntarIntent`, con un slot llamado
`pregunta` de tipo `AMAZON.SearchQuery`. Agrega cada frase por separado pulsando **+**:

```text
pregunta {pregunta}
consulta {pregunta}
quiero saber {pregunta}
```

No pegues las tres frases juntas en el campo de una sola frase. Pulsa **Save →
Build skill** y espera a que termine. La dirección
`http://127.0.0.1:3000` solo es accesible desde tu equipo y no sirve como endpoint
público para Amazon. Por ahora, conserva la skill en desarrollo.

Los pasos de creación se basan en la
[guía oficial de Amazon](https://developer.amazon.com/en-US/docs/alexa/devconsole/create-a-skill-and-choose-the-interaction-model.html).

## 6. Conectar la skill con el servidor local mediante HTTPS

Necesitas Node.js 22 o superior, OpenSSL 3 y cloudflared. El verificador usa la
criptografía nativa de Node y OpenSSL, sin paquetes de npm. En Windows, el script
de arranque puede utilizar el OpenSSL que viene con Git para Windows.

Crea `.env` en la raíz del proyecto y agrega tu ID de la consola de Alexa:

```dotenv
ALEXA_SKILL_ID=amzn1.ask.skill.REEMPLAZAR_CON_TU_ID
```

En PowerShell inicia el servidor:

```powershell
cd C:\workSpace\AlexIA
.\scripts\start-alexa.ps1
```

`cargo run` inicia los dos servidores: Alexa en `BIND_ADDR` (por defecto 3000)
y la web Vite en `UI_PORT` (por defecto 3001). El script es un ayudante opcional
para localizar las herramientas. Puedes configurar sus rutas desde la interfaz.
Los valores guardados en `.env` tienen prioridad sobre el shell. Si cambias
puertos o `UI_MODE`, reinicia AlexIA. El metaprompt y el modelo se aplican al guardar.

En otra terminal inicia el túnel:

El túnel apunta al puerto de `BIND_ADDR`; no publiques el puerto de la interfaz.

```powershell
cd C:\workSpace\AlexIA
.\scripts\start-tunnel.ps1
```

Si falta cloudflared, descárgalo desde la [página oficial](https://developers.cloudflare.com/tunnel/downloads/).
Agrega su ejecutable al PATH o guarda la versión de Windows de 64 bits como
`target/tools/cloudflared.exe`, creando esa carpeta si hace falta.

El túnel imprime una dirección `https://...trycloudflare.com`. En la consola de
Alexa abre **Build → Endpoint**, selecciona **HTTPS** y pega esa dirección con
`/alexa` al final en **Default Region**. Por ejemplo:

```text
https://NOMBRE-DEL-TUNEL.trycloudflare.com/alexa
```

Para el certificado wildcard de TryCloudflare, selecciona **My development
endpoint is a sub-domain of a domain that has a wildcard certificate from a
certificate authority**, y pulsa **Save**.

En **Test** selecciona **Development** y usa el simulador en **Spanish (US)**,
o el idioma que configuraste en tu skill. Para probar desde el Echo, utiliza la
misma cuenta de Amazon y la misma variante de idioma.

Prueba en este orden:

1. «Alexa, abre mi asistente». Debe responder que AlexIA está conectada.
2. «Pregunta qué es una estrella». Alexa espera y dice la respuesta de Gemini.
3. Al terminar la respuesta, Alexa cierra la sesión y deja de escuchar.
   Para otra consulta, di otra vez «Alexa, abre mi asistente».

Si la sesión se cerró, di «Alexa, abre mi asistente» para comenzar otra vez.
La consulta se responde en la misma petición, sin ejecución en segundo plano.
Si Gemini supera el presupuesto local de 7 segundos (incluida la verificación),
Alexa indica que la consulta tardó demasiado; puedes intentarlo de nuevo.
La respuesta pendiente se conserva en memoria hasta dos minutos, vinculada al
usuario de Alexa que la solicitó, y se elimina al entregarla. Después de leer
la respuesta, la skill cierra la sesión y deja de escuchar. Solo se ejecuta
una consulta a Gemini a la vez. Reiniciar el servidor elimina las respuestas pendientes.

Deja abiertas las dos terminales. Detener el túnel o apagar el PC interrumpe esta
conexión local. Cada nuevo túnel genera una URL diferente: debes actualizarla en
Amazon. El túnel es para desarrollo y hace accesible el servidor desde Internet;
`/alexa` solo procesa mensajes con firma, certificado, fecha e ID de skill válidos.
No publiques la skill: sigue siendo una prueba personal de desarrollo.

## 7. Resolver problemas frecuentes

| Problema | Qué hacer |
| --- | --- |
| `cargo` no se reconoce | Instala Rust y vuelve a abrir PowerShell o VS Code. |
| `link.exe not found` | Instala Build Tools con Desarrollo para el escritorio con C++, MSVC y Windows SDK. |
| `agy.exe` no se encuentra | Comprueba la instalación de Antigravity y vuelve a abrir la terminal. |
| Antigravity falla por sesión o cuota | Ejecuta `agy` directamente para ver el motivo y comprobar tu cuenta. |
| La consulta supera el tiempo permitido | Prueba con una pregunta más breve; si persiste, comprueba `agy` directamente. |
| `/health` no responde | Comprueba que `cargo run` sigue abierto y que no terminó con un error. |
| Windows bloquea `AlexIA.exe` al compilar | Detén el servidor con Ctrl+C o usa la carpeta de compilación alternativa que se muestra abajo. |
| El simulador muestra `Invalid AuthToken Error` | Cierra sesión en la consola de Amazon y vuelve a entrar. Después abre Test y selecciona Development. |
| El simulador deniega el micrófono | Puedes escribir las frases en el campo de texto para probar sin micrófono. |
| `/alexa` devuelve HTTP 400 con un navegador o PowerShell | Es lo esperado para solicitudes sin una firma válida de Amazon. Prueba desde Alexa. |
| La skill falla después de reiniciar el túnel | Copia la URL nueva con `/alexa` al final en Build → Endpoint y guarda. |

Para consultar sin reutilizar la carpeta de compilación del servidor:

```powershell
cargo run --target-dir target/connection-check --example ask -- "¿Qué es una estrella?"
```

## 8. Usarlo en Render más adelante

El objetivo es que Alexa pueda consultar Gemini aunque tu PC esté apagado.
Para lograrlo, tanto AlexIA como su conexión autenticada con Gemini deberán
funcionar en Render. Aún no se ha validado ese despliegue ni la persistencia y
renovación de la sesión de Google en el servidor. Primero completaremos la
conexión y las pruebas locales.
