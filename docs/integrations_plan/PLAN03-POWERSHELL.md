# PLAN03 — Integración de PowerShell con AlexIA

Fecha: 2026-11-03. Estado: **plan propuesto; integración pendiente**. Prioridad: P1, ejecutor de tareas en Windows.
Audiencia: comunidad técnica, inicialmente instalaciones personales con proyectos autorizados.

## 1. Resultado y alcance

Ejecutar tareas Windows previamente definidas, capturar su resultado y cancelarlas sin convertir la conversación en una terminal irrestricta.

**Primer recorrido entregable:** Elegir Validar proyecto; AlexIA ejecuta el task_id autorizado y presenta salida acotada, exit code y artefactos.

**Alcance inicial:**

- PowerShell 7 detectado por ruta explícita; Windows PowerShell 5.1 solo como compatibilidad separada.
- Manifiesto de tareas versionado con ejecutable, argumentos, raíz, entorno y efectos.
- Ejecución no interactiva, cancelación del árbol de procesos y límites.

**Ampliaciones posteriores:** Tareas parametrizadas tipadas y perfiles de ejecución aislados; soporte de tareas repetibles por proyecto.

**Fuera de esta primera entrega:** Invoke-Expression, -Command generado por el modelo, elevación de privilegios, bypass de ExecutionPolicy y perfiles de usuario.

## 2. Punto de partida real y prerrequisitos

Tokio process ya se usa para Antigravity y el verificador, pero no es un ejecutor general seguro. `kill_on_drop` del hijo no garantiza terminar nietos. No existe manifiesto de tareas ni Windows Job Object.

La interfaz actual administra configuración; no existe todavía el catálogo ejecutable de conectores ni la API de trabajos descrita aquí. El estado actual y las dependencias transversales están en [PLAN00 — Base común](PLAN00-BASE-COMUN.md). Las rutas, tipos, módulos y opciones nuevos que siguen son propuestas de implementación, no contratos existentes.

**Prerrequisitos específicos:**

- BC-01/02/04 y workspace de prueba descartable.
- Detectar pwsh.exe con ruta y versión; no instalarlo silenciosamente.
- Implementar control de árbol en Windows y probar permisos del usuario estándar.

**Dependencias entre planes:** PLAN00; base local para PLAN07/10/11 y otros CLI. No requiere GitHub.

## 3. Decisión técnica y flujo de conexión

El backend acepta task_id y parámetros tipados, resuelve una definición aprobada y lanza Command::new con argumentos separados. Para binarios nativos como cargo, preferir ejecución directa sin shell. Cuando una tarea necesita PowerShell, usar script revisado mediante -File, -NoProfile y -NonInteractive. Ejecutar código del repositorio, incluso un script de pruebas, es ejecución de código: exigir confianza del proyecto y una política adecuada. Un directorio de trabajo no es un sandbox.

**Procedimiento de conexión o habilitación:**

1. Detectar PowerShell por ruta configurada y comprobar versión sin leer perfiles.
2. Registrar una raíz canónica permitida y clasificar el proyecto como confiable o aislado.
3. Crear manifiesto de tarea con script revisado, hash, tipos de parámetros y efectos esperados.
4. Mostrar ejecutable, argumentos saneados, directorio, red y archivos escribibles antes de autorizar.
5. Crear Job Object con cierre del árbol, pipes drenados concurrentemente y timeout.
6. Ejecutar, clasificar exit code, sanear salida y eliminar temporales propios.

**Autenticación, renovación y revocación:** Usa identidad local del sistema operativo y autenticación del cliente AlexIA. No necesita credenciales PowerShell. No heredar todo el entorno: construir allowlist, preservando variables del sistema estrictamente necesarias.

**Permisos externos y selección de recursos:** Capacidad por task_id y hash del script. Cambiar script, ejecutable, working directory o parámetros fuera del esquema invalida la autorización previa. Prohibir tareas administrativas iniciales.

Las credenciales residen en el almacén del sistema descrito en PLAN00; la configuración conserva solamente referencias. El navegador, el modelo y la respuesta de Alexa no reciben tokens. Una instalación comunitaria no comparte credenciales con otra. Si el producto exige un servicio OAuth confidencial, su secreto permanece en ese servicio; nunca se distribuye en el frontend, extensión o binario público.

## 4. Capacidades autorizables

Los identificadores de esta tabla son capacidades internas de AlexIA; no son nombres de scopes del proveedor.

| Capacidad propuesta | Operación | Guarda específica |
|---|---|---|
| `powershell.tasks.list` | Listar tareas registradas | Proyecto vinculado |
| `powershell.tasks.run` | Ejecutar una tarea aprobada | Hash, parámetros y efectos autorizados |
| `powershell.tasks.cancel` | Cancelar árbol propio | Propiedad de job y PID verificada |

Las lecturas pueden preautorizarse para recursos concretos. Las escrituras o ejecuciones se sujetan a una política explícita por proyecto. Una política previa válida evita pedir confirmación repetida; si falta, preparar una vista previa concreta. La autorización queda ligada a identidad, recurso, parámetros y versión. Texto de un issue, archivo o registro no modifica permisos.

## 5. Superficie externa y contrato del adaptador

| Operación | API, comando o mecanismo | Tratamiento de la respuesta |
|---|---|---|
| Descubrimiento | pwsh.exe -NoLogo -NoProfile -NonInteractive -File <script-revisado> | Contrato de salida versionado |
| Binario nativo | tokio::process::Command con argv separado | No añadir capa cmd /c |
| Script de tarea | -File con parámetros tipados | No interpolar en un comando de texto |
| Cancelación | Windows Job Object y espera de salida | Comprobar muerte de hijos y nietos |

Convención propuesta: exit code 0 para éxito de tarea; códigos de herramientas se preservan y el adaptador específico decide hallazgos frente a fallo. Para scripts propios, emitir una línea JSON de resumen a stdout y diagnósticos saneados a stderr. Capturar bytes con límite y decodificación explícita; no asumir que PowerShell 5.1 y 7 codifican igual.

**Petición interna ilustrativa:**

```json
{
  "integration_id": "powershell",
  "project_id": "project-demo",
  "capability": "powershell.tasks.run",
  "input": {
    "task_id": "project-check",
    "parameters": {
      "profile": "quick"
    }
  },
  "idempotency_key": "demo-powershell-001"
}
```

Se envía al propuesto `POST /api/jobs`, exclusivo de la API administrativa autenticada. Respuesta: HTTP 202 con `job_id`, `status` y `status_url`. `GET /api/jobs/{job_id}` consulta progreso; `POST /api/jobs/{job_id}/cancel` solicita cancelación. Validar esquema y pertenencia antes de consultar credenciales o iniciar procesos. No existe un endpoint que acepte una URL o comando arbitrario del modelo.

**Datos específicos normalizados:** task_id, executable_version, script_hash, cwd_id, exit_code, started_at, duration_ms, truncation y artifact_ids.

El resultado incluye `schema_version`, identidad del recurso, instante, versión de herramienta/API, alcance, evidencia saneada, advertencias y estado de cobertura. La respuesta puede terminar con hallazgos aunque el proceso haya funcionado correctamente. Diferenciar ausencia de resultados, permisos insuficientes, cobertura parcial y ejecución fallida.

## 6. Archivos y cambios previstos en AlexIA

Los archivos nuevos se crean durante la implementación. Evitar duplicar contratos del núcleo en el conector.

| Ruta relativa al repositorio | Responsabilidad |
|---|---|
| `src/execution/windows.rs` | Job Object y control del árbol |
| `src/execution/tasks.rs` | Manifiesto y parámetros tipados |
| `src/integrations/powershell.rs` | Detección y adaptación |
| `tests/integrations_powershell.rs` | Procesos y entradas hostiles |
| `tests/fixtures/tasks/check.ps1` | Fixture revisada sin efectos externos |

`src/lib.rs` registraría los módulos; `src/control.rs` delegaría rutas administrativas al núcleo de integraciones. `ui/src/main.js` consumiría estados y resultados; `ui/index.html`/`ui/src/components.js` presentarían la conexión. Conservar las guardas loopback actuales y actualizar [BACKEND.md](../../.agents/.docs/BACKEND.md), [UI.md](../../.agents/.docs/UI.md) y el documento operativo correspondiente al implementar.

## 7. Configuración propuesta

| Campo lógico | Tipo/protección | Uso |
|---|---|---|
| `executable_path` | Ruta absoluta | pwsh.exe autorizado |
| `task_manifest` | Ruta/hash | Definiciones revisadas |
| `timeout_seconds` | Entero acotado | Inicial propuesto 120 |
| `output_limit_bytes` | Entero | Inicial propuesto 1048576 por stream |
| `environment_allowlist` | Lista | Sin tokens del proveedor |

Estos campos pertenecen a un registro validado de integración, no a un conjunto de secretos nuevos en `.env`. Escribirlos desde la API local con control de revisión. Exponer solamente `configured`, permisos y errores saneados. La presencia de configuración no equivale a conexión verificada.

## 8. Implementación por entregas verificables

### 03.1 — Registrar tarea sin ejecutarla

**Trabajo:** Validar manifiesto, raíz y parámetros; mostrar vista previa.

**Dependencias:** BC-01 a BC-05 de PLAN00; completar los prerrequisitos de este documento.

**Aceptación:**

- [ ] Path traversal rechazado.
- [ ] Un parámetro no se convierte en código.

**Verificación:** Pruebas de ;, comillas, $(), backticks y rutas Unicode.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### 03.2 — Ejecutar y cancelar

**Trabajo:** Implementar pipes y Job Object con tarea fixture.

**Dependencias:** Tarea 03.1 completada y contrato de salida estable.

**Aceptación:**

- [ ] Timeout termina nietos.
- [ ] Salida enorme no bloquea ni excede el límite.

**Verificación:** Fixture con proceso descendiente y presión de stdout/stderr.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### 03.3 — Integrar resultado en UI

**Trabajo:** Ejecutar una tarea de prueba desde panel y mostrar evidencia.

**Dependencias:** Tarea 03.2 completada y contrato de salida estable.

**Aceptación:**

- [ ] Botón Cancelar afecta solo el job propio.
- [ ] Cambiar script invalida permiso.

**Verificación:** Prueba real local y recorrido CLI del panel.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### Checkpoint de salida

- [ ] Primera operación real completa desde UI hasta proveedor/herramienta y resultado visible.
- [ ] Denegación comprobada al cambiar identidad, proyecto o recurso autorizado.
- [ ] Cancelación, error y desconexión dejan estados coherentes y permiten recuperación.
- [ ] No se modificó el contrato de firma, certificado, timestamp e ID de Alexa.
- [ ] La documentación distingue simulación, prueba real y funciones pendientes.

## 9. Errores, límites y recuperación

Una tarea por proyecto y timeout propuesto de 120 s, configurable hasta un techo explícito. No reintentar tareas por defecto: pueden tener efectos. Si la aplicación reinicia, una tarea en ejecución se marca interrupted y se verifica su árbol antes de permitir otra. Probar stdin cerrado para evitar esperas interactivas.

Límites iniciales de diseño: una ejecución por proyecto, cola local acotada, cuerpo administrativo de hasta 128 KiB y resultados paginados; los artefactos grandes se entregan por identificador. Los valores se medirán antes de liberar y no describen capacidad demostrada. HTTP 401 expira la sesión; 403 no se corrige ampliando permisos silenciosamente; 429 respeta `Retry-After` cuando exista. Un timeout después de una escritura produce estado `unknown_outcome` hasta reconciliar, sin repetición ciega.

## 10. Riesgos particulares y controles

| Riesgo | Control previsto | Evidencia requerida |
|---|---|---|
| Inyección de comandos | -File revisado/argv; nada de Invoke-Expression | Metacaracteres permanecen datos |
| Proceso huérfano | Job Object KILL_ON_JOB_CLOSE y prueba de nietos | Ningún descendiente vivo tras timeout |
| Credenciales heredadas | Entorno por allowlist | Fixture no puede leer token centinela |

**Datos y retención:** La salida puede contener rutas, secretos o datos del proyecto; redacción antes de guardar y antes de enviar al modelo. No capturar la sesión de terminal personal del usuario.

## 11. Pruebas del adaptador y del flujo completo

**Fixtures y pruebas de contrato:**

- Ruta con espacios y Unicode; argumentos vacíos y comillas.
- Script modificado después de autorización, junction fuera de raíz y ejecutable sustituido.
- Cancelación durante generación masiva de stderr y proceso descendiente persistente.

**Prueba real de aceptación:** Ejecutar scripts fixture de éxito, fallo, timeout y nieto en una carpeta de prueba. Ejecutar una tarea de validación inocua del repositorio sin elevar permisos.

**Prueba de interfaz con la skill Playwright CLI:**

1. Abrir tareas, elegir project-check y revisar efectos.
2. Ejecutar, comprobar estados queued/running/completed y exit code.
3. Ejecutar fixture lenta y cancelar; contrastar con verificación del proceso en el host.
4. Comprobar que salida con HTML se muestra como texto y secretos centinela quedan redactados.

Usar sesión de navegador desechable; obtener referencias con `snapshot` y ejecutar comprobaciones con `run-code --filename=...` cuando se necesiten aserciones. Para el estado simulado, interceptar únicamente la API propuesta e identificar el resultado como prueba de UI. La prueba real debe comprobar el recurso externo o artefacto local. No automatizar pantallas de login ni guardar cookies del usuario para sustituir APIs oficiales.

Comandos de validación cuando se implemente: `cargo test --locked --test integrations_powershell` para el archivo propuesto, `cargo check --locked --all-targets`, las pruebas de UI afectadas y `npm --prefix ui run build`. Estos comandos son objetivos; no se afirma que esos tests existan hoy. Añadir pruebas de esquema, autorización y recuperación antes de habilitar capacidades de escritura.

## 12. Despliegue, reversión y criterio de finalización

Habilitar primero para una instalación piloto mediante flag de integración y solamente las capacidades verificadas. Registrar versión y permisos. Después ampliar a un segundo proyecto y comprobar separación de conexiones; no equivale a un despliegue SaaS multiusuario.

**Reversión específica:** Deshabilitar capacidad run, cerrar Job Objects propios, revocar tareas y conservar resultados saneados hasta TTL. No borrar carpetas del proyecto para limpiar una ejecución.

Deshabilitar el conector impide nuevos trabajos; cancelar o drenar los existentes según su efecto. Revocar credenciales cuando corresponda. Desconectar no borra recursos ajenos ni deshace efectos externos de forma automática. La reversión del software y la compensación de una operación externa son procedimientos distintos.

**Se considera completada cuando:** Se ejecutan tareas reales autorizadas sin inyección, el árbol termina al cancelar y la UI muestra la evidencia sin secretos.

**Decisiones o condiciones pendientes:** Elegir crate/binding de Job Objects y política para proyectos no confiables. Si no hay aislamiento suficiente, limitar ejecución al proyecto local confiable y declararlo.

## 13. Fuentes oficiales y revalidación

Consultadas el 2026-11-03. Sustentan los mecanismos externos; el diseño de AlexIA es una propuesta propia. Revalidar scopes, rutas, versión, licencia y condiciones de distribución antes de implementar. Los enlaces de contratos específicos prevalecen sobre ejemplos generales; una página accesible no demuestra que una cuenta tenga habilitada la función.

- [Parámetros de pwsh](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_pwsh?view=powershell-7.5) — Invocación no interactiva
- [Prevención de inyección](https://learn.microsoft.com/en-us/powershell/scripting/security/preventing-script-injection?view=powershell-7.5) — Argumentos y tipado
- [Job Objects](https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects) — Control de árboles en Windows
