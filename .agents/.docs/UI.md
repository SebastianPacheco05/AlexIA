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

Panel de ajustes en español: lateral oscuro, superficie clara, acento terracota,
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
