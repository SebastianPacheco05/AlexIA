# Implementación local de PLAN00 y PLAN01

Fecha: 2026-11-03 (America/Bogota). Estado: **parcial comprobado; creación y merge no habilitados**.

La sesión de revisión terminó antes de editar código. Se tomó como contrato C1 de [PLAN00](../PLAN00-BASE-COMUN.md), [GitHub R2](../PLAN01-GITHUB.md) y [la auditoría cruzada](PLAN00-GITHUB-CONSISTENCIA.md). Este documento registra implementación y evidencia, sin sustituir esos contratos.

## Código entregado y límites

| Área | Implementado | Límite real |
|---|---|---|
| Proyectos | Fuentes locales/remotas, owner/repo ID; lectura explícita acotada con exclusión de traversal/secretos | No es snapshot inmutable ni sandbox contra cambios concurrentes de filesystem |
| Identidad | Bootstrap de un uso/10 min/10 intentos; cookie HttpOnly/SameSite Strict; CSRF estable entre pestañas; sesión de 8 h | HTTP loopback, cookie host-only sin Secure; pairing probado como primitiva, sin extensión distribuida |
| Credenciales | SecretStore y Windows Credential Manager; referencias opacas sin fallback `.env`/JSON | Otros SO devuelven indisponibilidad; refresh automático pendiente |
| Jobs | Cola de 32 activos, 2 workers, idempotencia/conflicto 409, continuación por propietario/revisión/plazo, cancelación y recuperación | Sin árbol de hijos con grants de escritura; ningún job escribe en repositorios |
| Journal | ACL privada, archivo exclusivo, frames versionados/digest/sync, tail incompleto descartado, fallo de E/S bloquea posteriores, compactación | Una instancia por directorio; digest no protege frente al propietario del SO |
| Retención | Deadline 20 min; borrado de metadatos/resultados 24 h tras terminar; conserva efectos no reconciliados | Mantenimiento al abrir, cada 60 s y al encolar/listar/consultar; recibos de PR reales pendientes |
| GitHub | App propia/device flow, identidad humana GET /user, instalación por app ID, selección/validación de repositorios | App/cuenta real aún no configurada ni autorizada; sin transferencia de tokens a IA |
| Selección | PR por author ID o recibo exacto en selector, exhaustividad controlada; draft/fork no sustituyen al candidato por otro | Última subida automática pendiente; se pide selección manual de rama |
| CI | Lector checks/statuses/Actions; gate puro por snapshot/fuente/intento | Diagnóstico parcial, sin acreditar requisitos/reviews/base ni anunciar todos los tests aprobados |
| Voz | Intents nuevos y vínculo skill/user → conexión; CommandService interno/acuse sin búsqueda remota; consulta directa conservada | Modelo Amazon como archivo, sin subir; simulador/Echo pendientes. Listener aún depende de preparación existente del proveedor |
| UI | Desbloqueo, BYO App, repositorio del piloto, vínculo Alexa, catálogo/jobs/selección/cancelación; OBS-01 CRLF/LF corregida | Sin botones de publicar/fusionar, endpoint de artefactos, paginación de lista de jobs o instalador |

Módulos: `src/auth`, `src/projects`, `src/jobs`, `src/storage.rs`, `src/secrets`, `src/integrations`, `src/alexa/github_intents.rs`, `ui/src/integrations.js`. Host/Origin/marcador/rechazo forwarded siguen envolviendo la administración. El router público no monta las rutas nuevas.

## Configurar la App del piloto

GitHub en la web es suficiente. **GitHub App** es una integración registrada, no GitHub Desktop.

1. Registrar una App privada propia en [Settings → Developer settings → GitHub Apps](https://github.com/settings/apps/new). Activar **Device flow**. AlexIA no necesita recibir client secret ni private key. Si GitHub pide generar una private key para preparar/instalar la App, conservar el archivo fuera del repositorio; ese requisito del registro no añade autenticación por clave al conector. Dejar webhook desactivado para este piloto manual.
2. Conceder **Contents: read** y **Pull requests: read**. Para diagnóstico CI, añadir **Checks: read**, **Commit statuses: read** y **Actions: read**. La capacidad actual no necesita escritura.
3. Instalarla solo en los repositorios del piloto. Anotar los **App ID** y **Client ID** públicos, distintos de un token.
4. Ejecutar `cargo run --locked`; abrir Integraciones → GitHub y pegar el código efímero de la terminal. Si caducó o se perdió la cookie tras consumirlo, reiniciar el proceso para obtener otro.
5. Introducir los IDs públicos y pulsar Autorizar en GitHub. Abrir el enlace oficial e introducir allí el código de dispositivo. El device_code secreto permanece en RAM; la UI solo muestra el user_code.
6. Cargar repositorios y guardar repositorio/base/destinos exactos. La UI guarda uno; HTTP admite hasta diez. Aliases/tests esperados están en RepositoryPolicy; su editor avanzado sigue pendiente.
7. Vincular explícitamente los IDs de skill/usuario Alexa. Adaptar [el modelo de interacción](../alexa-interaction-model.es.json) al locale real; no se modifica Amazon automáticamente.

Al expirar autorización, desconectar/reconectar. Desconectar revoca acceso/jobs locales y borra el token del vault; la revocación remota se hace en ajustes de GitHub. No desinstala una App compartida. La limpieza local puede reintentarse aun con estado revoked mientras last_error=disconnect_cleanup_pending; la UI ofrece «Reintentar limpieza». Al completarse, «Desconectar» queda deshabilitado. No se permite nueva autorización con limpieza pendiente.

Corrección posterior al piloto: un OAuth válido sin instalación conserva el token
en el vault y la identidad en configured_unverified con last_error=app_installation_required.
Después de instalar se puede pulsar Verificar conexión, que comprueba usuario y App
instalada antes de habilitar repositorios. La implementación anterior descartaba
ese token y una verificación posterior acababa en vault_unavailable. Para un estado
antiguo degraded sin identidad/token, instalar la App y desconectar/autorizar de
nuevo una vez. En navegador se observó ese estado junto a una App aún no instalada
y permisos de repositorio sin acceso; no se concedieron permisos ni se instaló la App.

Validación de esta corrección: 49 tests Rust y 5 tests de estado UI correctos,
build Vite y diff correctos. Incluye OAuth válido antes de instalar, verificación
que mantiene bloqueo hasta la instalación y recuperación de registros antiguos
sin token, sin leer credenciales reales ni modificar permisos externos.

Seguimiento del arranque: el frontend nuevo puede convivir por HMR con un binario
Rust anterior. El catálogo ahora declara authorization_flow_version=2; si falta,
la UI pide reiniciar el lanzador y no ofrece otra autorización destinada a fallar.
Se observó el proceso anterior activo y se comprobó el aviso en navegador. El
vínculo Alexa se movió a Configuración → Alexa con su propio resultado de guardado.
Validación posterior: 49 tests Rust, 6 tests UI, build Vite y revisión visual correctos.

## Decisiones operativas

Estado privado por checkout en `.alexia-data/`, ignorado por Git y con ACL protegida; credenciales separadas en Credential Manager del usuario. Motivo: aislar instalaciones/pruebas sin servicios externos. Para este piloto sustituye la propuesta de ubicación global bajo datos del usuario. No mover/publicar el directorio ni borrarlo para repetir un efecto desconocido: eliminarlo pierde idempotencia/vínculos/evidencia.

`jobs.jsonl`/`github.jsonl` usan schema_version 1. No contienen prompts/código ni tokens; el API limita entradas a selección y resultados a 64 KiB. Conexiones se compactan al último registro. Trabajos activos se interrumpen al reiniciar; EffectIntent sin recibo pasa a reconciling, sin reenvío automático. La reconciliación remota real queda pendiente.

Límites iniciales: 10 s/llamada HTTP, HTTPS exclusivo sin proxies heredados ni redirects, JSON 1 MiB/respuesta, 100 elementos/página y 20 páginas/listado. Superar límites produce error de cobertura/tamaño. Health no consulta servicios remotos. Son límites configurados, no mediciones de capacidad/latencia.

## Pendientes: los planes no están completos

- **G05/G06/G07:** aislamiento técnico Antigravity sin demostrar. Una rama con PR informa already_exists; de otro modo termina blocked/technical_isolation_unverified. No hay POST de creación ni título/body inventado.
- **G09/G10:** no existe precondición verificable de destino frente a retarget concurrente. Selección/diagnóstico terminan blocked/destination_guard_unverified antes de PATCH/PUT; no hay switch para saltarlo.
- **G03/BC-07a:** receptor firmado durable, índice y cobertura de pushes pendientes. Nunca se infiere última rama por fecha de commit.
- **BC-02b completo:** recuperación de EffectIntent probada; reconciliación con GitHub y recibos de bot duraderos pendientes de flujos de escritura.
- **BC-03 completo:** grants genéricos por capacidad/versiones/efectos e ingreso de clientes emparejados pendientes; piloto aplica owner/conexión/allowlists y deniega escritura.
- **BC-04/BC-08:** recetas, aislamiento técnico, instaladores y distribución comunitaria pendientes; no necesarios para lectura GitHub.
- **G08 completo:** requisitos/reviews y asociación exacta de todos los tests con base/intento, espera/seguimiento de heads bajo grants explícitos pendientes.
- **G11/BC-05b/BC-06b:** OAuth/App/repositorio desechable reales, simulador y Echo pendientes. Ningún test local los sustituye.

No se modificó infraestructura, Amazon ni repositorios remotos. Las pruebas usaron configuración aislada, sin cambiar `.env`/metaprompt personales.

## Evidencia

- Rust: proyectos, sesiones/pairing, vault sintético Windows, jobs/continuaciones, TTL/compactación, journal truncado/fallo de E/S, propiedad PR y gate.
- Proceso real terminado después de enviar un efecto a TCP fixture: recuperación reconciling sin reenvío.
- HTTP fixture: selección/continuación/merge diagnóstico, identidad ajena denegada, PR ajena excluida y contador de escrituras cero.
- Playwright CLI/backend real/assets compilados: **17 comprobaciones correctas**, sesión/dos pestañas/cola/foco/CRLF/escritorio/móvil 390×844 sin desbordamiento, cero errores de página/warnings finales; sin interceptar peticiones. La autorización de la primera pestaña se comprobó mediante respuesta HTTP 200 después de abrir la segunda.
- Primera ejecución del script de navegador esperó un control habilitado pero oculto en la segunda pestaña; se corrigió el script y se repitió. Capturas/salida transitoria: `target/plan00-01/`, excluido de Git, sin cuentas reales.
- ACL/vault/sockets/OpenSSL probados fuera del sandbox tras comprobar sus restricciones, sin debilitar controles.
- **47 pruebas Rust** pasaron en `cargo test --locked --offline`; `cargo check --locked --offline --all-targets` correcto. **8 pruebas Node** pasaron (2 de estado UI, 6 del verificador). Build Vite correcto y npm audit: cero vulnerabilidades. Rustfmt comprobado sobre los archivos Rust afectados; `git diff --check` y enlaces locales correctos. `cargo-audit` no está instalado; no se afirma auditoría automatizada Rust.
- Revisión independiente: corregidos CSRF entre pestañas, retención/capacidad, writer inválido tras fallo parcial, desconexión reintentable, selección reiniciada por polling, bienvenida/readiness y guía de permisos. Mutación de la condición de autoría hizo fallar el test esperado; se restauró el archivo y pasó la suite completa.

Fuentes revalidadas: [device flow](https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/generating-a-user-access-token-for-a-github-app), [PR/merge](https://docs.github.com/en/rest/pulls/pulls), [checks](https://docs.github.com/en/rest/checks/runs), [Actions](https://docs.github.com/en/rest/actions/workflow-runs), [Reqwest 0.12.28](https://docs.rs/reqwest/0.12.28/reqwest/struct.ClientBuilder.html). REST 2026-03-10; Cargo.lock resuelto para Rust 1.85 declarado. Compilador probado: 1.99.1; MSRV/contenedor no probados.
