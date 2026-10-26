# AlexIA — Imagen Docker

Fecha: 2026-10-25. Estado: preparación para despliegue; no despliegue remoto.

Tres etapas: Vite/Tailwind con Node 24, Rust con Cargo.lock y runtime Node 24
+ OpenSSL 3 + CA + tini. Axum sirve la UI compilada con `UI_MODE=static`,
sin Vite ni instalación npm en runtime. UID/GID 10001 sin privilegios,
health check local y cierre por SIGTERM.

```powershell
docker build -t alexia:local .
docker run --rm -p 127.0.0.1:3000:3000 alexia:local
```

Sin ID de skill y proveedor disponible, `/alexa` permanece bloqueado. Health
solo confirma vida local, no autenticación ni cuota. La administración escucha
en loopback dentro del contenedor y no se publica. Para administración local
en Docker Linux puede usarse `--network host` en lugar de `-p`: con
`UI_HOST=localhost` la UI sigue en `localhost:3001` del host. El modo de red host
de Docker Desktop depende de su configuración y no se habilita aquí.
No publicar el panel por el túnel de Alexa ni cambiar UI_HOST a 0.0.0.0.

## Configuración y dependencia pendiente

Para conservar cambios de la UI, montar un directorio de trabajo writable en
`/app`, propiedad del UID 10001, con `.env`, `prompts/meta-prompt.txt`,
`tools/alexa-verifier/verify.cjs` y `ui/dist/`. El montaje reemplaza los recursos
de la imagen: preparar todos antes del arranque. Para pruebas puede usarse la
capa writable del contenedor, aceptando su pérdida al borrarlo. No montar solo
`.env` como archivo si se quiere editarlo: el reemplazo atómico requiere poder
renombrar archivos en su directorio. Usar `UI_MODE=static` y rutas Linux.

La imagen no incluye Antigravity ni copia la sesión personal del equipo.
Las consultas reales requieren instalar una versión Linux compatible de `agy`
y autenticación admitida, provisionadas fuera de la imagen; `AGY_BIN` apunta
al ejecutable. La integración dentro del contenedor queda pendiente: no extraer
cookies ni incluir credenciales en Git o en capas de imagen.

## Validación

Registrar el resultado real del build y health antes de declarar probada la
imagen. Rust/UI compilados localmente no equivalen al contenedor ejecutado.
No se despliega infraestructura remota en esta tarea.

Comprobación 2026-10-25: `docker info` no conectó con
`dockerDesktopLinuxEngine` (motor no iniciado). No se ejecutó un build de
imagen. El build local de Vite y las pruebas Rust del modo static pasaron;
esto no sustituye la validación del contenedor.
