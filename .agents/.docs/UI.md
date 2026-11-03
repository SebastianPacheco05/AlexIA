# AlexIA — Configuración local

Fecha: 2026-10-25. Estado: implementado y validado localmente.

## Objetivo y alcance

Editar metaprompt, variables de `.env` y proveedor desde una interfaz local,
sin editar Rust. Solo Antigravity está disponible. Un `cargo run` inicia Axum
y Vite con JavaScript puro y Tailwind en `ui/`. Sin despliegue externo ni base
de datos. Conservar seguridad Alexa, respuesta directa y cierre de sesión.

## Contrato y plan

1. Configuración: lectura explícita de `.env` y `prompts/meta-prompt.txt`,
   validación, secretos redactados, revisión optimista y escritura con reemplazo
   por archivo. Variables desconocidas se conservan. El archivo prevalece sobre
   valores heredados de la terminal para que la UI sea la fuente persistente.
2. Control: listener administrativo en loopback con puerto interno dinámico,
   accesible a Vite mediante proxy `/api`; nunca montar estas rutas en Alexa.
   Validar Host, Origin y encabezado de petición local; sin CORS abierto.
   `GET /api/settings` lee; `PUT /api/settings` valida, prepara proveedor,
   persiste y sustituye el servicio sin recompilar. Puertos requieren reinicio.
   Fallos de preparación no sustituyen la configuración activa.
3. UI: Vite vanilla + Tailwind, navegación Proveedor / Metaprompt / Entorno,
   estado real, errores, cambios sin guardar y guardado explícito. Valores
   secretos vacíos significan conservar el valor; nunca devolver su contenido.
   Interfaz responsive y navegable por teclado; no usar HTML de datos externos.

`BIND_ADDR` controla Alexa (predeterminado `127.0.0.1:3000`); `UI_PORT` controla
Vite (predeterminado 3001), `UI_HOST` solo admite loopback. No pueden coincidir.
El túnel solo apunta a Alexa. El arranque comprueba dependencias de `ui/` y
ejecuta Vite como hijo, con cierre junto a Rust. La instalación inicial de npm
se documenta y se realiza para esta entrega.
UI_MODE=dev es el predeterminado; static sirve `ui/dist` con Axum en el mismo
puerto local administrativo, sin Vite. Cambiar UI_MODE requiere reinicio.

Ampliación autorizada 2026-10-25: sección **Integraciones**, con una única
tarjeta GitHub. Estado «Sin conectar / Próximamente», botón deshabilitado;
no incluye OAuth, credenciales, llamadas a GitHub, automatizaciones ni base de
datos. Marca pública oficial de GitHub servida como asset local. Se validó la
sección en Chrome sin warnings ni errores; se versiona como módulo separado.

Revisión 2026-10-25: se sustituye la tarjeta grande por un botón compacto con
icono y nombre. Abre un modal nativo `dialog` con la preparación de conexión;
autenticación sigue pendiente y el botón de conectar permanece deshabilitado.
Cierre con botón, Escape y fondo; foco contenido por el diálogo y restaurado
al botón de origen. Motivo: reducir espacio ocupado por cada integración.
Validación: build Vite correcto, apertura y cierre del modal (botón y Escape),
retorno de foco comprobado en Chrome y consola sin warnings/errores. Antes del
commit se repitió la consulta real de Alexa: «Tres más tres es seis», cierre
de sesión true y estado de proveedor ok (2445 ms), sin errores del servicio.

## Diseño

2026-10-25, ajustes solicitados: eliminar las tarjetas «Mi espacio», «Tu equipo.
Tus reglas» y el bloque de perfil del pie. Se conserva Entorno como sección
independiente; el nuevo botón Configuración ocupa el pie del lateral y muestra
el botón de configuración guiada de Alexa por petición del propietario. `cargo run` pasa
a la cabecera y se compactan las tarjetas de inicio para reducir desplazamiento.

La barra de guardar permanece oculta hasta que existan diferencias con la
configuración cargada; volver a los valores originales la oculta. Permanece
visible durante el guardado y se oculta al confirmarse o descartar cambios.
La comparación incluye metaprompt, valores protegidos redactados y claves
nuevas. La consulta de túnel no recarga ni sobrescribe ediciones pendientes.

La tarjeta de conexiones muestra el endpoint HTTPS completo con `/alexa` y un
botón para copiarlo. `GET /api/tunnel`, exclusivo de administración local,
responde `{endpoint: string|null}`. Se consulta cada 5 s con la pestaña visible
y al recuperar foco. El script `start-tunnel.ps1` registra URL/puerto/latido en
`target/tunnels/*.json`, cada 2 s; el backend rechaza registros de más de 10 s,
puertos distintos y URL ajenas a HTTPS de quick tunnels de trycloudflare.com.
No se escribe la URL en `.env` ni se publica la API administrativa. Si el
lanzador termina, elimina su registro; cierre abrupto caduca por el latido.
El registro acredita que el lanzador sigue ejecutándose, no disponibilidad
extremo a extremo. No se configura automáticamente la consola de Amazon.

Ampliación de Configuración: botón compacto Alexa, con el asset
del propietario `ui/public/alexa.png`. Integraciones conserva únicamente GitHub.
Alexa abre un diálogo de configuración guiada:
enlace a la consola oficial de Amazon, copia del endpoint vigente y explicación
de ID de skill, certificado HTTPS y prueba en Development. «Abrir Entorno» muestra
todas las variables. Entrar a Entorno limpia la búsqueda anterior para evitar
que un filtro persistente haga parecer que faltan variables; el buscador sigue
disponible para búsquedas manuales. El modo estático sirve ambos logos con guardas locales.
La guía es manual: no solicita usuario/contraseña, no inicia sesión en Amazon,
no almacena tokens ni crea/modifica skills. OAuth de Login with Amazon y SMAPI
queda como alternativa futura, no implementada, con permisos explícitos.
Fuentes: [SMAPI](https://developer.amazon.com/en-US/docs/alexa/smapi/smapi-overview.html)
y [endpoint HTTPS](https://developer.amazon.com/en-US/docs/alexa/custom-skills/host-a-custom-skill-as-a-web-service.html).

2026-10-25, implementado: marca del lateral sustituida por el SVG del propietario
`ui/public/AlexIA.svg`, con texto alternativo. Se ajustó el viewBox para retirar
el margen vacío del lienzo, conservando sus trazados y colores. El servidor de
assets estáticos también sirve `/AlexIA.svg` con las mismas guardas locales.
Revisión de proporciones del lateral solicitada por el propietario: tras evaluar
la UI en Chrome, se conserva el SVG a 84 px, centrado por petición expresa y con
margin-bottom de 12 px sobre el subtítulo. Sustituye el ajuste intermedio de
100 px centrado. Lateral de 264 px en escritorio y 240 px en ancho intermedio,
navegación de 15 px con iconos de 20 px y filas de al menos 50 px. Se retiran
los números decorativos de navegación. Tarjeta de espacio y pie con más aire;
el lateral puede desplazarse verticalmente en ventanas bajas. En móvil se
conserva navegación horizontal con filas de 42 px e iconos de 16 px.
Validación: build Vite, prueba de ruta
local/rechazo de Host externo y revisión en Chrome sin errores de consola.

Rediseño total autorizado 2026-10-25, implementado: consola oscura neutra
con acento lima, lateral compacto, cabecera de estado, formularios de dos
columnas y estado contextual integrado. Sustituye la superficie clara y acento
terracota anteriores. Navegación con iconos Lucide, selección de razonamiento
visual, editor de prompt amplio, tabla de entorno filtrable y modales nativos
consistentes. Anime.js gestiona transiciones breves respetando reduced-motion.
Referencias: composición de Dialog/Sidebar de shadcn, controles de RareUI y
movimiento de React Bits/Animate UI; no se introduce React ni se afirma usar
sus componentes. Se usan componentes propios sobre Vite vanilla y Tailwind.
Conservar contratos de API, privacidad, guardado explícito, GitHub pendiente,
botón compacto y navegación por teclado. Verificar cada vista, guardado,
modal, build, consola y Alexa antes de versionar. No mostrar métricas ficticias.

Diseño anterior (sustituido): lateral oscuro, superficie clara, acento terracota,
formularios con etiquetas persistentes, tipografía del sistema y jerarquía
compacta. Guardar es la acción principal. Estados de conexión/aplicación y
errores son texto además de color. En móvil, navegación horizontal y una columna.

## Aceptación y entrega

Pruebas Rust de lectura/escritura y concurrencia, preservación de secretos,
validación de puertos/proveedor y rechazo de peticiones externas; build de UI,
verificación en navegador a ancho escritorio y móvil, guardado real reversible,
una consulta real en el simulador Alexa y ausencia de errores de consola.
Después, commits separados por módulos, sin secretos ni artefactos de `target/`.

Riesgos: preparación del CLI puede tardar o fallar; el guardado muestra su
resultado. Dos archivos se sustituyen por separado, con rollback en errores de
E/S; no se promete transacción ante pérdida de energía. Puertos se aplican al
reiniciar. Vite es un servidor local de desarrollo, no hosting de producción.

Fuentes: [Vite](https://vite.dev/config/server-options),
[Tailwind con Vite](https://tailwindcss.com/docs/installation/using-vite),
[middleware Axum](https://docs.rs/axum/0.8.9/axum/middleware/index.html).

## Evidencia de esta entrega

28 pruebas Rust y 6 pruebas del verificador Node completadas. Vite 8.3.5 y
Tailwind 4.3.3 compilan; npm audit informa 0 vulnerabilidades. La interfaz cargó
en Chrome, guardó un cambio reversible del metaprompt y lo restauró sin
recompilar. No se observaron warnings ni errores en su consola; navegación de
teclado comprobada. El simulador Alexa respondió «Dos más dos es cuatro»,
con `shouldEndSession: true`, y el servicio registró estado `ok` en 2435 ms
de proveedor. `cargo run` inició ambos puertos. La comprobación de viewport
móvil no pudo validarse: la herramienta mantuvo el viewport de escritorio.
Se conserva CSS responsive, sin afirmar verificación visual móvil completada.
Docker no pudo arrancarse: falta el motor Linux de Docker Desktop.

Rediseño completo: Anime.js 4.5.0 y Lucide 1.61.0 fijados en el lockfile.
Iconos y movimiento viven en `ui/src/components.js`; estado y persistencia en
`ui/src/main.js`. Las recetas añaden instrucciones al editor sin guardar por
su cuenta. Entorno filtra por nombre/descripción, mantiene ocultos los secretos
y añade variables mediante un diálogo con validación. Descartar ediciones pide
confirmación; los tres diálogos usan foco modal nativo, Escape y retorno de foco.
Sin cambio de contratos, puertos ni exposición del panel administrativo.

Verificación del rediseño: build Vite correcto (JS 51,78 kB, gzip 20,69 kB),
npm audit completo sin vulnerabilidades, navegación de las cuatro vistas,
receta y guardado real del metaprompt con restauración del original, filtro
GEMINI (2/12), validación y creación de una variable temporal sin persistir,
descarte de ediciones y modal GitHub con cierre por Escape y retorno de foco.
La selección Equilibrado refleja `medium` en Entorno sin guardar. Consola de
Chrome sin warnings ni errores en la carga final. Alexa respondió «Cuatro más
cuatro es ocho», `shouldEndSession: true`, proveedor ok en 2132 ms y sin errores
del servicio durante esa prueba. CSS contempla móvil, pero el override de la
herramienta sigue devolviendo 1920 px; verificación visual móvil pendiente.
No se repiten pruebas Rust al no cambiar backend ni sus contratos.

Fuentes del rediseño: [Anime.js](https://animejs.com/documentation/animation),
[Lucide](https://lucide.dev/guide/packages/lucide),
[Dialog de shadcn](https://ui.shadcn.com/docs/components/dialog).

Verificación de la ampliación de túnel/Alexa y limpieza del lateral: 29 pruebas
Rust de la suite completa correctas; comprobaciones focalizadas posteriores de
aislamiento de `/api/tunnel` y logos estáticos correctas. Seis pruebas Node del
verificador con ALEXA_OPENSSL_BIN explícito y una prueba Node de cambios netos
correctas. La primera ejecución de escritura de settings dentro del sandbox
falló por acceso al archivo temporal; fuera del sandbox pasó sin cambiar código.
Build Vite correcto. El script real publicó el endpoint y renovó su latido;
Chrome mostró la URL con `/alexa` y confirmó copia en ambos controles. Guardado
real reversible de metaprompt completado y restaurado al original. Barra oculta
al cargar, visible al editar y oculta al restaurar el valor original. Alexa en
Configuración abre la guía; Abrir Entorno muestra todas las variables. Consola sin
warnings/errores. Simulador real: «Cinco más cinco es diez», cierre true,
proveedor ok en 2156 ms y sin errores del servicio. OAuth/SMAPI no implementado.
