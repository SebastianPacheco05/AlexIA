# PLAN04 — Integración de Bash con AlexIA

Fecha: 2026-11-03. Estado: **plan propuesto; integración pendiente**. Prioridad: P1 para Linux; P2 para WSL.
Audiencia: comunidad técnica, inicialmente instalaciones personales con proyectos autorizados.

## 1. Resultado y alcance

Ejecutar tareas Linux declaradas conservando las mismas garantías de identidad, raíz y cancelación del ejecutor Windows.

**Primer recorrido entregable:** Seleccionar una tarea validada de un proyecto Linux y obtener resultado reproducible sin cargar archivos de inicio del usuario.

**Alcance inicial:**

- Bash nativo Linux; WSL como backend explícito distinto.
- Scripts revisados con argv tipado, entorno mínimo y raíz permitida.
- Grupos de procesos y aislamiento según confianza del repositorio.

**Ampliaciones posteriores:** Integración con runners remotos autenticados; nunca una sesión SSH compartida por todos los usuarios.

**Fuera de esta primera entrega:** bash -c con texto del modelo, source de perfiles, sudo, eval y conversión automática de rutas Windows/WSL.

## 2. Punto de partida real y prerrequisitos

El código actual usa Tokio process para proveedores, sin ejecutor Bash ni registro de distribuciones WSL. Docker no demuestra que exista un runner seguro para código ajeno.

La interfaz actual administra configuración; no existe todavía el catálogo ejecutable de conectores ni la API de trabajos descrita aquí. El estado actual y las dependencias transversales están en [PLAN00 — Base común](PLAN00-BASE-COMUN.md). Las rutas, tipos, módulos y opciones nuevos que siguen son propuestas de implementación, no contratos existentes.

**Prerrequisitos específicos:**

- BC-01/02/04 y host Linux o distribución WSL identificada.
- Pruebas Linux reales además del desarrollo Windows.
- Definir usuario sin privilegios, permisos de red y directorios escribibles.

**Dependencias entre planes:** PLAN00 y contrato de tareas de PLAN03, con backend propio de procesos; PLAN06 puede proporcionar ejecución contenida.

## 3. Decisión técnica y flujo de conexión

Mantener TaskSpec común y una implementación Unix. Preferir Command::new del ejecutable nativo; usar bash --noprofile --norc script.sh con argv separado solo para scripts revisados. Eliminar BASH_ENV y ENV del entorno. No usar -l. Un process group sirve para cancelación, pero no es aislamiento: código no confiable necesita contenedor endurecido o VM y controles de recursos. WSL define explícitamente distribución, usuario, cwd Linux y conversión de rutas.

**Procedimiento de conexión o habilitación:**

1. Detectar /bin/bash y versión; registrar backend native-linux o wsl con distribución concreta.
2. Vincular raíz Linux real; resolver symlinks y evitar escape de directorio.
3. Validar TaskSpec, parámetros, hash y clasificación de efectos.
4. Construir entorno mínimo y grupo de procesos propio; preparar aislamiento si el proyecto no es confiable.
5. Ejecutar sin archivos de inicio, drenar streams y esperar con deadline.
6. Cancelar grupo con TERM y luego KILL si no termina; esperar hijos y registrar interrupción.

**Autenticación, renovación y revocación:** Identidad local del usuario Linux o usuario WSL seleccionado; token del cliente AlexIA autoriza task_id. SSH y credenciales remotas quedan fuera de la primera fase.

**Permisos externos y selección de recursos:** Solo tareas declaradas y raíces registradas. No permitir que parámetros seleccionen una distribución, usuario root, socket Docker o script externo. Un permiso de lectura no autoriza ejecutar make/npm del repositorio.

Las credenciales residen en el almacén del sistema descrito en PLAN00; la configuración conserva solamente referencias. El navegador, el modelo y la respuesta de Alexa no reciben tokens. Una instalación comunitaria no comparte credenciales con otra. Si el producto exige un servicio OAuth confidencial, su secreto permanece en ese servicio; nunca se distribuye en el frontend, extensión o binario público.

## 4. Capacidades autorizables

Los identificadores de esta tabla son capacidades internas de AlexIA; no son nombres de scopes del proveedor.

| Capacidad propuesta | Operación | Guarda específica |
|---|---|---|
| `bash.tasks.list` | Listar tareas Linux disponibles | Backend y raíz registrados |
| `bash.tasks.run` | Ejecutar TaskSpec | Script/hash y perfil de aislamiento |
| `bash.tasks.cancel` | Cancelar procesos propios | Process group y job_id propios |

Las lecturas pueden preautorizarse para recursos concretos. Las escrituras o ejecuciones se sujetan a una política explícita por proyecto. Una política previa válida evita pedir confirmación repetida; si falta, preparar una vista previa concreta. La autorización queda ligada a identidad, recurso, parámetros y versión. Texto de un issue, archivo o registro no modifica permisos.

## 5. Superficie externa y contrato del adaptador

| Operación | API, comando o mecanismo | Tratamiento de la respuesta |
|---|---|---|
| Script nativo | bash --noprofile --norc <script> <argv> | No pasar comando compuesto a -c |
| WSL opcional | wsl.exe -d <distro> --user <user> --exec <ejecutable> <argv> | Cwd Linux definido explícitamente |
| Binario nativo | Command::new(program).args(argv) | Evitar shell cuando no hace falta |
| Cancelación Unix | Grupo propio; SIGTERM seguido de SIGKILL | Esperar y verificar procesos descendientes |

Los parámetros son elementos de argv, no texto escapado para una shell. Usar -- cuando la herramienta lo admita para datos que empiecen por guion. No ejecutar hook Git ni scripts package.json automáticamente al inspeccionar un repositorio. La detección de capacidad falla de forma explícita si no existe host compatible.

**Petición interna ilustrativa:**

```json
{
  "integration_id": "bash",
  "project_id": "project-demo",
  "capability": "bash.tasks.run",
  "input": {
    "task_id": "linux-check",
    "backend_id": "linux-local",
    "parameters": {
      "mode": "quick"
    }
  },
  "idempotency_key": "demo-bash-001"
}
```

Se envía al propuesto `POST /api/jobs`, exclusivo de la API administrativa autenticada. Respuesta: HTTP 202 con `job_id`, `status` y `status_url`. `GET /api/jobs/{job_id}` consulta progreso; `POST /api/jobs/{job_id}/cancel` solicita cancelación. Validar esquema y pertenencia antes de consultar credenciales o iniciar procesos. No existe un endpoint que acepte una URL o comando arbitrario del modelo.

**Datos específicos normalizados:** backend_id, distribución/usuario cuando aplique, task_id, script_hash, raíz, pid/group internos, exit code y artefactos.

El resultado incluye `schema_version`, identidad del recurso, instante, versión de herramienta/API, alcance, evidencia saneada, advertencias y estado de cobertura. La respuesta puede terminar con hallazgos aunque el proceso haya funcionado correctamente. Diferenciar ausencia de resultados, permisos insuficientes, cobertura parcial y ejecución fallida.

## 6. Archivos y cambios previstos en AlexIA

Los archivos nuevos se crean durante la implementación. Evitar duplicar contratos del núcleo en el conector.

| Ruta relativa al repositorio | Responsabilidad |
|---|---|
| `src/execution/unix.rs` | Grupos de procesos y cancelación |
| `src/execution/wsl.rs` | Backend opcional con rutas explícitas |
| `src/integrations/bash.rs` | Detección y capacidades |
| `tests/integrations_bash.rs` | Contratos Linux |
| `tests/fixtures/tasks/check.sh` | Scripts revisados de prueba |

`src/lib.rs` registraría los módulos; `src/control.rs` delegaría rutas administrativas al núcleo de integraciones. `ui/src/main.js` consumiría estados y resultados; `ui/index.html`/`ui/src/components.js` presentarían la conexión. Conservar las guardas loopback actuales y actualizar [BACKEND.md](../../.agents/.docs/BACKEND.md), [UI.md](../../.agents/.docs/UI.md) y el documento operativo correspondiente al implementar.

## 7. Configuración propuesta

| Campo lógico | Tipo/protección | Uso |
|---|---|---|
| `backend_kind` | Enum | native-linux o wsl |
| `bash_path` | Ruta absoluta | Binario autorizado |
| `distribution` | Texto validado | Solo backend WSL |
| `linux_project_root` | Ruta canónica | Sin conversión implícita |
| `network_profile` | Enum | denied o destinos concretos |

Estos campos pertenecen a un registro validado de integración, no a un conjunto de secretos nuevos en `.env`. Escribirlos desde la API local con control de revisión. Exponer solamente `configured`, permisos y errores saneados. La presencia de configuración no equivale a conexión verificada.

## 8. Implementación por entregas verificables

### 04.1 — Detectar y validar backend

**Trabajo:** Implementar disponibilidad, rutas y manifiesto común.

**Dependencias:** BC-01 a BC-05 de PLAN00; completar los prerrequisitos de este documento.

**Aceptación:**

- [ ] Windows sin WSL informa no disponible.
- [ ] Symlink fuera de raíz se rechaza.

**Verificación:** Matriz Linux nativo y WSL presente/ausente.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### 04.2 — Ejecutar scripts controlados

**Trabajo:** Añadir entorno mínimo y proceso/grupo propio.

**Dependencias:** Tarea 04.1 completada y contrato de salida estable.

**Aceptación:**

- [ ] BASH_ENV no se ejecuta.
- [ ] Metacaracteres no agregan comandos.

**Verificación:** Fixtures con variables hostiles y argv registrado.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### 04.3 — Verificar cancelación y UI

**Trabajo:** Integrar resultados y cancelación en el panel.

**Dependencias:** Tarea 04.2 completada y contrato de salida estable.

**Aceptación:**

- [ ] TERM/KILL no afecta procesos ajenos.
- [ ] Reinicio marca tarea interrumpida.

**Verificación:** Prueba Linux con nieto y recorrido web por CLI.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### Checkpoint de salida

- [ ] Primera operación real completa desde UI hasta proveedor/herramienta y resultado visible.
- [ ] Denegación comprobada al cambiar identidad, proyecto o recurso autorizado.
- [ ] Cancelación, error y desconexión dejan estados coherentes y permiten recuperación.
- [ ] No se modificó el contrato de firma, certificado, timestamp e ID de Alexa.
- [ ] La documentación distingue simulación, prueba real y funciones pendientes.

## 9. Errores, límites y recuperación

Deadline propuesto 120 s y máximo 1 MiB por stream; medir. No habilitar WSL por inferencia de PATH. Una pérdida de conexión con runner remoto futuro requiere reconciliar ejecución; nunca asumir cancelación porque cerró el socket.

Límites iniciales de diseño: una ejecución por proyecto, cola local acotada, cuerpo administrativo de hasta 128 KiB y resultados paginados; los artefactos grandes se entregan por identificador. Los valores se medirán antes de liberar y no describen capacidad demostrada. HTTP 401 expira la sesión; 403 no se corrige ampliando permisos silenciosamente; 429 respeta `Retry-After` cuando exista. Un timeout después de una escritura produce estado `unknown_outcome` hasta reconciliar, sin repetición ciega.

## 10. Riesgos particulares y controles

| Riesgo | Control previsto | Evidencia requerida |
|---|---|---|
| Inicio de código por BASH_ENV | Limpiar entorno; no login shell | Archivo centinela no ejecutado |
| Escape de rutas en WSL | Separar identidades de host y raíces | Caso /mnt/c fuera de permiso rechazado |
| Falsa seguridad de proceso/grupo | Aislamiento aparte para código no confiable | Prueba de red y filesystem del perfil |

**Datos y retención:** No heredar SSH_AUTH_SOCK, tokens cloud ni variables del proveedor. Guardar stdout saneado y borrar temporales propios por TTL, sin modificar el HOME del usuario.

## 11. Pruebas del adaptador y del flujo completo

**Fixtures y pruebas de contrato:**

- Argumentos con ;, $(), backticks, saltos de línea y prefijo --.
- Archivo script sustituido y enlace simbólico fuera de raíz.
- Nietos, timeout, BASH_ENV hostil y salida binaria.

**Prueba real de aceptación:** Ejecutar en Linux/WSL fixtures de éxito, error y descendientes; comparar argv y verificar que ningún archivo centinela externo cambió.

**Prueba de interfaz con la skill Playwright CLI:**

1. Seleccionar backend y ver disponibilidad real.
2. Ejecutar linux-check y consultar el job.
3. Comprobar estado no disponible en host sin Bash.
4. Cancelar fixture lenta y verificar en host el cierre completo; navegador solo comprueba la presentación.

Usar sesión de navegador desechable; obtener referencias con `snapshot` y ejecutar comprobaciones con `run-code --filename=...` cuando se necesiten aserciones. Para el estado simulado, interceptar únicamente la API propuesta e identificar el resultado como prueba de UI. La prueba real debe comprobar el recurso externo o artefacto local. No automatizar pantallas de login ni guardar cookies del usuario para sustituir APIs oficiales.

Comandos de validación cuando se implemente: `cargo test --locked --test integrations_bash` para el archivo propuesto, `cargo check --locked --all-targets`, las pruebas de UI afectadas y `npm --prefix ui run build`. Estos comandos son objetivos; no se afirma que esos tests existan hoy. Añadir pruebas de esquema, autorización y recuperación antes de habilitar capacidades de escritura.

## 12. Despliegue, reversión y criterio de finalización

Habilitar primero para una instalación piloto mediante flag de integración y solamente las capacidades verificadas. Registrar versión y permisos. Después ampliar a un segundo proyecto y comprobar separación de conexiones; no equivale a un despliegue SaaS multiusuario.

**Reversión específica:** Deshabilitar backend, terminar grupos propios y retirar credenciales de runners si se añadieran. No desinstalar WSL ni cambiar shells o distribuciones del usuario.

Deshabilitar el conector impide nuevos trabajos; cancelar o drenar los existentes según su efecto. Revocar credenciales cuando corresponda. Desconectar no borra recursos ajenos ni deshace efectos externos de forma automática. La reversión del software y la compensación de una operación externa son procedimientos distintos.

**Se considera completada cuando:** La matriz de host soportado pasa, argumentos se preservan y la cancelación termina descendientes propios sin tocar otros procesos.

**Decisiones o condiciones pendientes:** Prioridad Linux frente a WSL y tecnología de aislamiento. La documentación GNU no pudo recuperarse con el navegador de investigación; revalidar manual local y fuente antes de fijar flags adicionales.

## 13. Fuentes oficiales y revalidación

Consultadas el 2026-11-03. Sustentan los mecanismos externos; el diseño de AlexIA es una propuesta propia. Revalidar scopes, rutas, versión, licencia y condiciones de distribución antes de implementar. Los enlaces de contratos específicos prevalecen sobre ejemplos generales; una página accesible no demuestra que una cuenta tenga habilitada la función.

- [Código y manual Bash, fuente GNU](https://git.savannah.gnu.org/cgit/bash.git/tree/doc/bashref.texi) — Fuente oficial alternativa del manual
- [WSL: comandos](https://learn.microsoft.com/en-us/windows/wsl/basic-commands) — Distribución, usuario e invocación
- [Proceso asíncrono Tokio](https://docs.rs/tokio/latest/tokio/process/index.html) — Gestión de procesos; no promete sandbox
