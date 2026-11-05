# PLAN07 — Integración de Playwright con AlexIA

Fecha: 2026-11-03. Estado: **plan propuesto; integración pendiente**. Prioridad: P2, validación web.
Audiencia: comunidad técnica, inicialmente instalaciones personales con proyectos autorizados.

## 1. Resultado y alcance

Ejecutar pruebas web reproducibles y publicar evidencias navegables sin mezclar sesiones o credenciales de usuarios.

**Primer recorrido entregable:** Elegir una suite registrada, ejecutarla contra una aplicación de prueba autorizada y revisar tests fallidos, capturas y trazas saneadas.

**Alcance inicial:**

- Playwright Test como runner de suites versionadas.
- Playwright CLI como interfaz de exploración y comprobación humana/agentic separada.
- Resultados JSON y artefactos con límites y TTL.

**Ampliaciones posteriores:** Generación de tests propuesta y revisada, comparación visual y matrices de navegadores.

**Fuera de esta primera entrega:** Usar el navegador personal, guardar cookies sin decisión, recorrer URLs arbitrarias o dar por válida una integración solo por una captura.

## 2. Punto de partida real y prerrequisitos

No hay suite Playwright en el proyecto. En esta tarea se instaló la skill oficial de microsoft/playwright-cli y el ejecutable @playwright/cli 0.1.23 fuera de las dependencias de AlexIA. La prueba final documental se registra en PLAN00; no equivale a implementar este adaptador.

La interfaz actual administra configuración; no existe todavía el catálogo ejecutable de conectores ni la API de trabajos descrita aquí. El estado actual y las dependencias transversales están en [PLAN00 — Base común](PLAN00-BASE-COMUN.md). Las rutas, tipos, módulos y opciones nuevos que siguen son propuestas de implementación, no contratos existentes.

**Prerrequisitos específicos:**

- Runner de PLAN03/04 o PLAN06 y proyecto de pruebas confiable.
- Versiones de @playwright/test y navegadores fijadas conjuntamente.
- Orígenes permitidos y cuentas de prueba sin datos reales.

**Dependencias entre planes:** PLAN00 y ejecución; puede usar PLAN06 para entorno efímero.

## 3. Decisión técnica y flujo de conexión

Crear adaptador de suites por suite_id. Ejecutar Playwright Test instalado en el proyecto/harness con configuración revisada, reporter JSON y artefactos por job. CLI tiene sesiones nombradas desechables; no reemplaza los tests persistentes. Los tests, config y scripts Node son código ejecutable y requieren confianza/aislamiento. Restringir salida de red también fuera de page.route: procesos Node pueden hacer solicitudes por su cuenta.

**Procedimiento de conexión o habilitación:**

1. Registrar suite y configuración con hash, comando y baseURL permitida.
2. Instalar versión fijada y navegadores en fase explícita de provisioning; no descargar durante cada job.
3. Iniciar aplicación fixture o verificar readiness sin depender del proveedor IA.
4. Crear directorio de artefactos propio, contexto limpio y cuenta de prueba.
5. Ejecutar suite con reporter JSON y capturas/trazas de fallos.
6. Parsear resumen, verificar artefactos y cerrar browser/procesos aun en timeout.

**Autenticación, renovación y revocación:** No requiere cuenta Playwright. Secretos de aplicación de prueba se inyectan solo al runner autorizado. storageState es material secreto y no se versiona ni se entrega al frontend.

**Permisos externos y selección de recursos:** Suite, origen de aplicación, credenciales de prueba y red autorizados por proyecto. No reutilizar sesión CLI entre proyectos o identidades.

Las credenciales residen en el almacén del sistema descrito en PLAN00; la configuración conserva solamente referencias. El navegador, el modelo y la respuesta de Alexa no reciben tokens. Una instalación comunitaria no comparte credenciales con otra. Si el producto exige un servicio OAuth confidencial, su secreto permanece en ese servicio; nunca se distribuye en el frontend, extensión o binario público.

## 4. Capacidades autorizables

Los identificadores de esta tabla son capacidades internas de AlexIA; no son nombres de scopes del proveedor.

| Capacidad propuesta | Operación | Guarda específica |
|---|---|---|
| `playwright.suites.read` | Listar suites | Manifiesto revisado |
| `playwright.tests.run` | Ejecutar suite | Origen/config/hash autorizados |
| `playwright.artifacts.read` | Ver evidencia | Propiedad y MIME validado |

Las lecturas pueden preautorizarse para recursos concretos. Las escrituras o ejecuciones se sujetan a una política explícita por proyecto. Una política previa válida evita pedir confirmación repetida; si falta, preparar una vista previa concreta. La autorización queda ligada a identidad, recurso, parámetros y versión. Texto de un issue, archivo o registro no modifica permisos.

## 5. Superficie externa y contrato del adaptador

| Operación | API, comando o mecanismo | Tratamiento de la respuesta |
|---|---|---|
| Versión | playwright --version del paquete fijado | Registrar versión y navegador |
| Suite | playwright test --config=<config-revisada> | Config contiene reporter JSON y salida por job |
| Exploración CLI | playwright-cli -s=<job> open <origen-permitido> | Contexto nuevo; cerrar solo esa sesión |
| Aserciones CLI | run-code --filename=<archivo-revisado> | Errores hacen fallar verificación, no solo screenshot |

Configurar reporters JSON/HTML antes de ejecutar. HTML y trace pueden contener contenido activo o datos de red; servir mediante descarga o visor aislado, no insertarlos como HTML en la UI administrativa. Capturas son evidencia parcial; la comprobación funcional usa aserciones. No llamar close-all si hay sesiones ajenas.

**Petición interna ilustrativa:**

```json
{
  "integration_id": "playwright",
  "project_id": "project-demo",
  "capability": "playwright.tests.run",
  "input": {
    "suite_id": "web-smoke",
    "base_url_id": "demo-local",
    "browser": "chromium"
  },
  "idempotency_key": "demo-playwright-001"
}
```

Se envía al propuesto `POST /api/jobs`, exclusivo de la API administrativa autenticada. Respuesta: HTTP 202 con `job_id`, `status` y `status_url`. `GET /api/jobs/{job_id}` consulta progreso; `POST /api/jobs/{job_id}/cancel` solicita cancelación. Validar esquema y pertenencia antes de consultar credenciales o iniciar procesos. No existe un endpoint que acepte una URL o comando arbitrario del modelo.

**Datos específicos normalizados:** suite/hash, proyecto Playwright, test_id, intento, estado, duración, navegador, origen y artifact_ids.

El resultado incluye `schema_version`, identidad del recurso, instante, versión de herramienta/API, alcance, evidencia saneada, advertencias y estado de cobertura. La respuesta puede terminar con hallazgos aunque el proceso haya funcionado correctamente. Diferenciar ausencia de resultados, permisos insuficientes, cobertura parcial y ejecución fallida.

## 6. Archivos y cambios previstos en AlexIA

Los archivos nuevos se crean durante la implementación. Evitar duplicar contratos del núcleo en el conector.

| Ruta relativa al repositorio | Responsabilidad |
|---|---|
| `src/integrations/playwright.rs` | Selección y parseo de reportes |
| `tools/web-tests/package.json` | Harness separado y versiones fijadas |
| `tools/web-tests/playwright.config.ts` | Reporters y límites |
| `tests/integrations_playwright.rs` | Resultados y aislamiento |
| `ui/src/integrations/playwright.js` | Tests y artefactos |

`src/lib.rs` registraría los módulos; `src/control.rs` delegaría rutas administrativas al núcleo de integraciones. `ui/src/main.js` consumiría estados y resultados; `ui/index.html`/`ui/src/components.js` presentarían la conexión. Conservar las guardas loopback actuales y actualizar [BACKEND.md](../../.agents/.docs/BACKEND.md), [UI.md](../../.agents/.docs/UI.md) y el documento operativo correspondiente al implementar.

## 7. Configuración propuesta

| Campo lógico | Tipo/protección | Uso |
|---|---|---|
| `suite_ids` | Lista/hash | Suites revisadas |
| `allowed_origin_ids` | Lista | Destinos permitidos |
| `browser_profile` | Enum | Chromium inicial |
| `artifact_ttl_hours` | Entero | Propuesto 24 |
| `artifact_size_limit` | Entero | Propuesto 50 MiB por job |

Estos campos pertenecen a un registro validado de integración, no a un conjunto de secretos nuevos en `.env`. Escribirlos desde la API local con control de revisión. Exponer solamente `configured`, permisos y errores saneados. La presencia de configuración no equivale a conexión verificada.

## 8. Implementación por entregas verificables

### 07.1 — Registrar y detectar suite

**Trabajo:** Validar configuración y versión.

**Dependencias:** BC-01 a BC-05 de PLAN00; completar los prerrequisitos de este documento.

**Aceptación:**

- [ ] Sin navegador muestra provisioning requerido.
- [ ] Origen ajeno se rechaza.

**Verificación:** Pruebas de manifests y detección.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### 07.2 — Ejecutar y parsear

**Trabajo:** Correr suite con un test exitoso y uno fallido.

**Dependencias:** Tarea 07.1 completada y contrato de salida estable.

**Aceptación:**

- [ ] Fallo del test no se confunde con fallo del runner.
- [ ] Test skipped figura como omitido.

**Verificación:** Fixture real con JSON, captura y trace.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### 07.3 — Mostrar evidencia y cerrar

**Trabajo:** Visor seguro, cancelación y limpieza.

**Dependencias:** Tarea 07.2 completada y contrato de salida estable.

**Aceptación:**

- [ ] Artefactos de otro job rechazados.
- [ ] Cierre no toca sesiones ajenas.

**Verificación:** CLI en UI y verificación de procesos.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### Checkpoint de salida

- [ ] Primera operación real completa desde UI hasta proveedor/herramienta y resultado visible.
- [ ] Denegación comprobada al cambiar identidad, proyecto o recurso autorizado.
- [ ] Cancelación, error y desconexión dejan estados coherentes y permiten recuperación.
- [ ] No se modificó el contrato de firma, certificado, timestamp e ID de Alexa.
- [ ] La documentación distingue simulación, prueba real y funciones pendientes.

## 9. Errores, límites y recuperación

Timeout de test/suite/global independientes; proponer 30 s por test y 5 min por suite inicial. Retries explícitos y visibles; no ocultar flaky bajo éxito. Contextos nuevos por prueba según configuración. Fallo de JSON marca runner_error, no cero fallos.

Límites iniciales de diseño: una ejecución por proyecto, cola local acotada, cuerpo administrativo de hasta 128 KiB y resultados paginados; los artefactos grandes se entregan por identificador. Los valores se medirán antes de liberar y no describen capacidad demostrada. HTTP 401 expira la sesión; 403 no se corrige ampliando permisos silenciosamente; 429 respeta `Retry-After` cuando exista. Un timeout después de una escritura produce estado `unknown_outcome` hasta reconciliar, sin repetición ciega.

## 10. Riesgos particulares y controles

| Riesgo | Control previsto | Evidencia requerida |
|---|---|---|
| Tests arbitrarios ejecutan Node | Trust/aislamiento de config y suite | Fixture de red/FS bloqueada |
| Trace contiene secretos | TTL, redacción y acceso | Revisión de artefactos centinela |
| Sesión compartida | Contextos y nombres propios | Dos proyectos no comparten cookies |

**Datos y retención:** No capturar perfiles personales; trazas pueden incluir DOM, capturas y cuerpos de red. Permitir desactivar grabación sensible y borrar resultados por job.

## 11. Pruebas del adaptador y del flujo completo

**Fixtures y pruebas de contrato:**

- JSON corrupto, test omitted, flaky y timeout global.
- Artifact traversal, MIME activo y tamaño excesivo.
- Cancelación deja procesos y perfiles propios cerrados.

**Prueba real de aceptación:** Ejecutar suite contra aplicación fixture local con caso de éxito y fallo deliberado; comparar JSON con UI y abrir captura del fallo.

**Prueba de interfaz con la skill Playwright CLI:**

1. Usar la skill instalada para abrir UI y consultar la tarea.
2. Comprobar conteos por estado y duración.
3. Abrir captura y comprobar acceso denegado de otro job.
4. Verificar consola y red; anotar si se usaron mocks o backend real.

Usar sesión de navegador desechable; obtener referencias con `snapshot` y ejecutar comprobaciones con `run-code --filename=...` cuando se necesiten aserciones. Para el estado simulado, interceptar únicamente la API propuesta e identificar el resultado como prueba de UI. La prueba real debe comprobar el recurso externo o artefacto local. No automatizar pantallas de login ni guardar cookies del usuario para sustituir APIs oficiales.

Comandos de validación cuando se implemente: `cargo test --locked --test integrations_playwright` para el archivo propuesto, `cargo check --locked --all-targets`, las pruebas de UI afectadas y `npm --prefix ui run build`. Estos comandos son objetivos; no se afirma que esos tests existan hoy. Añadir pruebas de esquema, autorización y recuperación antes de habilitar capacidades de escritura.

## 12. Despliegue, reversión y criterio de finalización

Habilitar primero para una instalación piloto mediante flag de integración y solamente las capacidades verificadas. Registrar versión y permisos. Después ampliar a un segundo proyecto y comprobar separación de conexiones; no equivale a un despliegue SaaS multiusuario.

**Reversión específica:** Deshabilitar adaptador y cerrar sesiones propias; eliminar artefactos según TTL. Mantener tests versionados del usuario.

Deshabilitar el conector impide nuevos trabajos; cancelar o drenar los existentes según su efecto. Revocar credenciales cuando corresponda. Desconectar no borra recursos ajenos ni deshace efectos externos de forma automática. La reversión del software y la compensación de una operación externa son procedimientos distintos.

**Se considera completada cuando:** Una suite real produce estados y evidencias correctos, con límites y cierre comprobados; CLI y Playwright Test están diferenciados.

**Decisiones o condiciones pendientes:** Definir navegadores, estrategia de provisioning y privacidad de trazas antes de distribuir; no añadir dependencias al frontend solo por el runner.

## 13. Fuentes oficiales y revalidación

Consultadas el 2026-11-03. Sustentan los mecanismos externos; el diseño de AlexIA es una propuesta propia. Revalidar scopes, rutas, versión, licencia y condiciones de distribución antes de implementar. Los enlaces de contratos específicos prevalecen sobre ejemplos generales; una página accesible no demuestra que una cuenta tenga habilitada la función.

- [Playwright CLI oficial](https://github.com/microsoft/playwright-cli/tree/main) — Skill y comandos CLI
- [Reporters](https://playwright.dev/docs/test-reporters) — Salida estructurada
- [Instalación y navegadores](https://playwright.dev/docs/intro) — Runner y compatibilidad
