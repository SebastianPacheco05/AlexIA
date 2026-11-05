# PLAN05 — Integración de GitHub Actions con AlexIA

Fecha: 2026-11-03. Estado: **plan propuesto; integración pendiente**. Prioridad: P1, diagnóstico de CI.
Audiencia: comunidad técnica, inicialmente instalaciones personales con proyectos autorizados.

## 1. Resultado y alcance

**Actualización R2, 2026-11-03:** las lecturas de este plan son dependencia del squash merge solicitado en [PLAN01 R2](PLAN01-GITHUB.md). Actions sigue sin fusionar por sí mismo; entrega evidencia estructurada al gate de GitHub. La PR elegida queda fijada y el contrato de tests debe referenciar el head, la base de destino y el intento vigentes. Las secciones originales de diagnóstico se conservan; no acreditan todavía ese gate.

Consultar ejecuciones de CI, explicar fallos con evidencia y permitir reejecución solo como capacidad posterior independiente.

**Primer recorrido entregable:** Elegir un PR y ver qué job falló, sobre qué SHA y qué fragmento del registro respalda el diagnóstico.

**Alcance inicial:**

- Lectura de runs, jobs y logs acotados.
- Vinculación por repository_id, head_sha, run_id y run_attempt.
- Consulta manual inicial y polling solamente mientras una ejecución esté visible.

**Ampliaciones posteriores:** Reejecutar jobs, cancelar runs y workflow_dispatch con parámetros tipados y autorización.

**Fuera de esta primera entrega:** Editar workflows, secrets o runners; ejecutar código de forks con privilegios del repositorio base.

## 2. Punto de partida real y prerrequisitos

No hay cliente Actions. La tarjeta GitHub no conecta cuentas. Los eventos de CI no forman parte del handler Alexa.

La interfaz actual administra configuración; no existe todavía el catálogo ejecutable de conectores ni la API de trabajos descrita aquí. El estado actual y las dependencias transversales están en [PLAN00 — Base común](PLAN00-BASE-COMUN.md). Las rutas, tipos, módulos y opciones nuevos que siguen son propuestas de implementación, no contratos existentes.

**Prerrequisitos específicos:**

- Conexión GitHub PLAN01 y núcleo de tareas.
- Repositorio de prueba con workflow de éxito y fallo intencional sin secretos.
- Parser seguro de logs/ZIP y presupuesto de descompresión.

**Dependencias entre planes:** PLAN01 para identidad y repositorios; PLAN00 para tareas. Independiente del runner local.

## 3. Decisión técnica y flujo de conexión

Reutilizar el cliente y tokens de GitHub, añadiendo Actions: read. Cada diagnóstico fija run_id y run_attempt; un nuevo intento nunca sobrescribe evidencia anterior. Consultar logs desde backend, limitar descargas y sanear antes del modelo. No resolver un job fallido como corregido por el solo hecho de generar una explicación.

**Procedimiento de conexión o habilitación:**

1. Añadir Actions: read a la App; comprobar aceptación del nuevo permiso por la instalación.
2. Elegir un repo permitido y consultar runs por SHA.
3. Resolver jobs del intento seleccionado y descargar solo los logs necesarios.
4. Validar destino de red y descomprimir en carpeta de trabajo sin traversal.
5. Mostrar pasos fallidos y enlace a GitHub; permitir actualización manual.
6. Habilitar acciones de escritura en una entrega posterior separada.

**Autenticación, renovación y revocación:** Reutilizar tokens temporales de PLAN01 y su renovación; ninguna credencial nueva por run. Si faltan permisos Actions, mostrar qué operación está restringida.

**Permisos externos y selección de recursos:** Actions: read para lectura; Actions: write para cancelación/reejecución. La ampliación no se infiere de que Contents ya permita escritura.

Las credenciales residen en el almacén del sistema descrito en PLAN00; la configuración conserva solamente referencias. El navegador, el modelo y la respuesta de Alexa no reciben tokens. Una instalación comunitaria no comparte credenciales con otra. Si el producto exige un servicio OAuth confidencial, su secreto permanece en ese servicio; nunca se distribuye en el frontend, extensión o binario público.

## 4. Capacidades autorizables

Los identificadores de esta tabla son capacidades internas de AlexIA; no son nombres de scopes del proveedor.

| Capacidad propuesta | Operación | Guarda específica |
|---|---|---|
| `github-actions.runs.read` | Consultar runs/jobs | Repositorio permitido |
| `github-actions.logs.read` | Leer logs seleccionados | Run e intento autorizados |
| `github-actions.runs.rerun` | Reejecución posterior | Política de coste/efectos y run exacto |

Las lecturas pueden preautorizarse para recursos concretos. Las escrituras o ejecuciones se sujetan a una política explícita por proyecto. Una política previa válida evita pedir confirmación repetida; si falta, preparar una vista previa concreta. La autorización queda ligada a identidad, recurso, parámetros y versión. Texto de un issue, archivo o registro no modifica permisos.

## 5. Superficie externa y contrato del adaptador

| Operación | API, comando o mecanismo | Tratamiento de la respuesta |
|---|---|---|
| Runs | GET /repos/{owner}/{repo}/actions/runs | Paginar y filtrar por SHA |
| Jobs | GET /repos/{owner}/{repo}/actions/runs/{run_id}/jobs | Preservar intento y conclusión |
| Log de job | GET /repos/{owner}/{repo}/actions/jobs/{job_id}/logs | Redirección a URL temporal sin filtrar Authorization |
| Reejecutar, fase posterior | POST /repos/{owner}/{repo}/actions/runs/{run_id}/rerun | No repetir tras timeout sin reconciliar |

Deshabilitar redirección automática con credenciales. Procesar una URL firmada de descarga solo tras validar su relación con una respuesta GitHub y una política de destinos; no aceptar URLs del prompt. Limitar bytes comprimidos y expandidos. Diferenciar workflow pendiente, cancelado, skipped, neutral y failure.

**Petición interna ilustrativa:**

```json
{
  "integration_id": "github-actions",
  "project_id": "project-demo",
  "capability": "github-actions.runs.read",
  "input": {
    "repository_id": "repo-demo",
    "head_sha": "demo-sha",
    "run_id": 101,
    "run_attempt": 1
  },
  "idempotency_key": "demo-github-actions-001"
}
```

Se envía al propuesto `POST /api/jobs`, exclusivo de la API administrativa autenticada. Respuesta: HTTP 202 con `job_id`, `status` y `status_url`. `GET /api/jobs/{job_id}` consulta progreso; `POST /api/jobs/{job_id}/cancel` solicita cancelación. Validar esquema y pertenencia antes de consultar credenciales o iniciar procesos. No existe un endpoint que acepte una URL o comando arbitrario del modelo.

**Datos específicos normalizados:** RunSummary, JobSummary, step/conclusion, intento, SHA, fragmentos saneados y cobertura de logs.

El resultado incluye `schema_version`, identidad del recurso, instante, versión de herramienta/API, alcance, evidencia saneada, advertencias y estado de cobertura. La respuesta puede terminar con hallazgos aunque el proceso haya funcionado correctamente. Diferenciar ausencia de resultados, permisos insuficientes, cobertura parcial y ejecución fallida.

## 6. Archivos y cambios previstos en AlexIA

Los archivos nuevos se crean durante la implementación. Evitar duplicar contratos del núcleo en el conector.

| Ruta relativa al repositorio | Responsabilidad |
|---|---|
| `src/integrations/github_actions.rs` | Operaciones Actions |
| `src/integrations/github/logs.rs` | Descarga y lectura limitada |
| `tests/integrations_github_actions.rs` | Estados, logs y seguridad |
| `ui/src/integrations/github-actions.js` | Runs y diagnóstico |

`src/lib.rs` registraría los módulos; `src/control.rs` delegaría rutas administrativas al núcleo de integraciones. `ui/src/main.js` consumiría estados y resultados; `ui/index.html`/`ui/src/components.js` presentarían la conexión. Conservar las guardas loopback actuales y actualizar [BACKEND.md](../../.agents/.docs/BACKEND.md), [UI.md](../../.agents/.docs/UI.md) y el documento operativo correspondiente al implementar.

## 7. Configuración propuesta

| Campo lógico | Tipo/protección | Uso |
|---|---|---|
| `repository_ids` | Lista | Selección heredada |
| `logs_max_bytes` | Entero | Propuesto 10 MiB descargados por job |
| `expanded_max_bytes` | Entero | Propuesto 25 MiB expandidos por tarea |
| `poll_interval_seconds` | Entero | Propuesto 10, solo jobs activos |

Estos campos pertenecen a un registro validado de integración, no a un conjunto de secretos nuevos en `.env`. Escribirlos desde la API local con control de revisión. Exponer solamente `configured`, permisos y errores saneados. La presencia de configuración no equivale a conexión verificada.

## 8. Implementación por entregas verificables

### 05.1 — Mostrar runs reales

**Trabajo:** Conectar lectura por SHA y tabla.

**Dependencias:** BC-01 a BC-05 de PLAN00; completar los prerrequisitos de este documento.

**Aceptación:**

- [ ] No mezcla branches ni repos.
- [ ] Run pendiente no se muestra como éxito.

**Verificación:** Fixtures de conclusiones y un run real.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### 05.2 — Diagnosticar log seleccionado

**Trabajo:** Descargar con límites y generar evidencia.

**Dependencias:** Tarea 05.1 completada y contrato de salida estable.

**Aceptación:**

- [ ] URL temporal no recibe token GitHub.
- [ ] ZIP hostil se rechaza.

**Verificación:** Fixture con traversal/bomba y log fallido inocuo.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### 05.3 — Recuperar y desconectar

**Trabajo:** Añadir cancelación local de lectura y renovación.

**Dependencias:** Tarea 05.2 completada y contrato de salida estable.

**Aceptación:**

- [ ] Reconexión conserva intento.
- [ ] No se reejecuta workflow al actualizar UI.

**Verificación:** Prueba de expiración y navegación CLI.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### Checkpoint de salida

- [ ] Primera operación real completa desde UI hasta proveedor/herramienta y resultado visible.
- [ ] Denegación comprobada al cambiar identidad, proyecto o recurso autorizado.
- [ ] Cancelación, error y desconexión dejan estados coherentes y permiten recuperación.
- [ ] No se modificó el contrato de firma, certificado, timestamp e ID de Alexa.
- [ ] La documentación distingue simulación, prueba real y funciones pendientes.

## 9. Errores, límites y recuperación

### Contrato adicional para el gate de merge de PLAN01 R2

Consistencia C1: usar status waiting_for_external y stage waiting_for_checks de PLAN00; un emisor no define estados nuevos del core. Antigravity no es dependencia runtime para consultar CI. Respetar follow_head_updates solo si el grant explícito es follow_ref; default snapshot. La evidencia puede diagnosticarse con guard de destino unverified, pero no habilita retarget/merge por sí sola. El efecto permanece sujeto a la condición C1 de PLAN01 y al journal durable BC-02b.

La dependencia de PLAN01 es su conexión/cliente de lectura G01, identificada CI-READ en el grafo común. No exigir que creación o merge de PLAN01 estén completos para construir este lector; G08 lo consume después. Esto evita el ciclo PLAN01 -> PLAN05 -> PLAN01 terminado.

- Definir antes del trabajo los workflows/jobs/contextos esperados por repositorio y destino, con emisores confiables; no elegir después solo los que pasaron.
- Entregar al orquestador run/job IDs, workflow ID, evento, head SHA, base/ref y commit de integración cuando corresponda, run_attempt, estado/conclusión y cobertura. Si un dato no puede acreditarse, marcarlo desconocido.
- Reunir también checks/statuses de terceros: Actions por sí solo no representa todas las condiciones de una PR.
- Un head nuevo, cambio de base, retarget o reintento relevante invalida la evidencia anterior para ese gate. No usar un run verde viejo ni depender solo de combined_status.
- Todos los tests esperados deben tener success; un expected job skipped/neutral, ausente, cancelado o fallido no significa que se ejecutó y pasó. Excepciones no aplicables se declaran antes en el contrato.
- Si CI no se dispara para el destino nuevo, devolver tests_not_triggered. No editar workflows, lanzar dispatch ni elevar a Actions: write automáticamente.
- El job de merge espera fuera de la petición Alexa hasta el deadline configurado. follow_head_updates solo permite revalidar la misma PR/destino, con nuevo gate y sin renovar el plazo indefinidamente.
- La aceptación requiere pruebas contra base cambiada, run viejo y emisor que imita un nombre. El efecto de merge pertenece a PLAN01; este adaptador no hace PUT de squash.

Proponer 60 s para diagnóstico y máximo de tres lecturas HTTP simultáneas por conexión, medido antes de liberar. Polling con backoff y fin al estado terminal. Reejecución futura puede consumir minutos de CI y lanzar efectos del workflow: mostrarlo antes de autorizar.

Límites iniciales de diseño: una ejecución por proyecto, cola local acotada, cuerpo administrativo de hasta 128 KiB y resultados paginados; los artefactos grandes se entregan por identificador. Los valores se medirán antes de liberar y no describen capacidad demostrada. HTTP 401 expira la sesión; 403 no se corrige ampliando permisos silenciosamente; 429 respeta `Retry-After` cuando exista. Un timeout después de una escritura produce estado `unknown_outcome` hasta reconciliar, sin repetición ciega.

## 10. Riesgos particulares y controles

| Riesgo | Control previsto | Evidencia requerida |
|---|---|---|
| Logs con secretos | Redacción previa a persistencia/modelo | Fixture centinela ausente |
| Descarga hostil o enorme | Allowlist, límite y extracción segura | Pruebas ZIP/redirect |
| Duplicar ejecuciones | Solo lectura inicial; reconciliar intentos | Timeout no dispara segundo run |

**Datos y retención:** No almacenar archivos de log completos indefinidamente; conservar extractos mínimos con TTL y enlaces. La redacción de GitHub no garantiza ausencia de datos sensibles.

## 11. Pruebas del adaptador y del flujo completo

**Fixtures y pruebas de contrato:**

- Run con varios intentos y job faltante.
- 403 Actions aunque Contents funcione.
- Log expirado, ZIP con rutas absolutas y stream truncado.

**Prueba real de aceptación:** Leer runs de un workflow de prueba, provocar un fallo inocuo y verificar que el informe apunta al job, intento y SHA correctos.

**Prueba de interfaz con la skill Playwright CLI:**

1. Elegir un SHA y abrir runs.
2. Abrir job fallido y contrastar su evidencia.
3. Simular 429 y comprobar reintento visible sin duplicar.
4. Comprobar que reejecutar está ausente o deshabilitado con lectura.

Usar sesión de navegador desechable; obtener referencias con `snapshot` y ejecutar comprobaciones con `run-code --filename=...` cuando se necesiten aserciones. Para el estado simulado, interceptar únicamente la API propuesta e identificar el resultado como prueba de UI. La prueba real debe comprobar el recurso externo o artefacto local. No automatizar pantallas de login ni guardar cookies del usuario para sustituir APIs oficiales.

Comandos de validación cuando se implemente: `cargo test --locked --test integrations_github_actions` para el archivo propuesto, `cargo check --locked --all-targets`, las pruebas de UI afectadas y `npm --prefix ui run build`. Estos comandos son objetivos; no se afirma que esos tests existan hoy. Añadir pruebas de esquema, autorización y recuperación antes de habilitar capacidades de escritura.

## 12. Despliegue, reversión y criterio de finalización

Habilitar primero para una instalación piloto mediante flag de integración y solamente las capacidades verificadas. Registrar versión y permisos. Después ampliar a un segundo proyecto y comprobar separación de conexiones; no equivale a un despliegue SaaS multiusuario.

**Reversión específica:** Desactivar Actions sin desconectar necesariamente GitHub; detener polling y borrar URLs temporales. No cancelar CI ajeno al deshabilitar lectura.

Deshabilitar el conector impide nuevos trabajos; cancelar o drenar los existentes según su efecto. Revocar credenciales cuando corresponda. Desconectar no borra recursos ajenos ni deshace efectos externos de forma automática. La reversión del software y la compensación de una operación externa son procedimientos distintos.

**Se considera completada cuando:** Un fallo real se diagnostica con evidencia correcta y el adaptador maneja intentos, permisos y descargas sin filtrar tokens.

**Decisiones o condiciones pendientes:** Definir si el usuario necesita reejecución en el piloto; mantenerla fuera hasta probar efectos y presupuesto.

## 13. Fuentes oficiales y revalidación

Consultadas el 2026-11-03. Sustentan los mecanismos externos; el diseño de AlexIA es una propuesta propia. Revalidar scopes, rutas, versión, licencia y condiciones de distribución antes de implementar. Los enlaces de contratos específicos prevalecen sobre ejemplos generales; una página accesible no demuestra que una cuenta tenga habilitada la función.

- [Runs e intentos](https://docs.github.com/en/rest/actions/workflow-runs) — Contrato de lectura y reejecución
- [Jobs y registros](https://docs.github.com/en/rest/actions/workflow-jobs) — Descarga y permisos por endpoint
