# PLAN12 — Integración de Semgrep con AlexIA

Fecha: 2026-11-03. Estado: **plan propuesto; integración pendiente**. Prioridad: P2, análisis estático.
Audiencia: comunidad técnica, inicialmente instalaciones personales con proyectos autorizados.

## 1. Resultado y alcance

Ejecutar reglas revisadas sobre lenguajes compatibles y explicar hallazgos con ubicación y cobertura.

**Primer recorrido entregable:** Seleccionar un ruleset permitido, analizar snapshot y revisar una alerta con regla, archivo y explicación.

**Alcance inicial:**

- Semgrep CE local con reglas locales fijadas.
- Salida JSON y cobertura por lenguaje/archivo.
- Métricas desactivadas en modo local privado.

**Ampliaciones posteriores:** SARIF, baseline por rama y reglas propias revisadas; servicios cloud solo tras decisión independiente.

**Fuera de esta primera entrega:** Autofix automático, descarga de reglas arbitrarias del prompt y afirmar cobertura total para Rust/JavaScript u otro lenguaje.

## 2. Punto de partida real y prerrequisitos

No hay Semgrep ni rulesets. El lenguaje de AlexIA no garantiza soporte equivalente de todos los análisis; comprobar matriz CE y reglas antes de prometer cobertura.

La interfaz actual administra configuración; no existe todavía el catálogo ejecutable de conectores ni la API de trabajos descrita aquí. El estado actual y las dependencias transversales están en [PLAN00 — Base común](PLAN00-BASE-COMUN.md). Las rutas, tipos, módulos y opciones nuevos que siguen son propuestas de implementación, no contratos existentes.

**Prerrequisitos específicos:**

- Runtime compatible o contenedor fijado; comprobar Windows frente a Linux/WSL.
- Ruleset con versión/hash, licencia y lenguajes declarados.
- Runner con límites de CPU/RAM.

**Dependencias entre planes:** PLAN00 y PLAN04/06 cuando sea necesario por plataforma.

## 3. Decisión técnica y flujo de conexión

Ejecutar semgrep scan con --config de archivo/directorio revisado, --json y --metrics=off. No usar p/default remoto como regla silenciosamente mutable del producto. Mantener hash y fecha del ruleset. La IA explica un finding, no inventa flujos de datos ausentes del motor.

**Procedimiento de conexión o habilitación:**

1. Verificar versión/runtime y lenguajes con el proveedor oficial.
2. Provisionar reglas y registrar hash/licencia.
3. Seleccionar snapshot y excluir binarios, secretos y dependencias según política visible.
4. Ejecutar scan sin autofix con configuración local.
5. Parsear results y errors/skipped; señalar reglas que no corrieron.
6. Presentar hallazgos y preparar propuestas de corrección separadas.

**Autenticación, renovación y revocación:** CE con reglas locales no necesita cuenta SaaS. No introducir SEMGREP_APP_TOKEN ni activar semgrep ci/cloud por defecto.

**Permisos externos y selección de recursos:** Lectura de código y reglas; escritura de reporte propio. Cambiar reglas o habilitar autofix necesita capacidad distinta.

Las credenciales residen en el almacén del sistema descrito en PLAN00; la configuración conserva solamente referencias. El navegador, el modelo y la respuesta de Alexa no reciben tokens. Una instalación comunitaria no comparte credenciales con otra. Si el producto exige un servicio OAuth confidencial, su secreto permanece en ese servicio; nunca se distribuye en el frontend, extensión o binario público.

## 4. Capacidades autorizables

Los identificadores de esta tabla son capacidades internas de AlexIA; no son nombres de scopes del proveedor.

| Capacidad propuesta | Operación | Guarda específica |
|---|---|---|
| `semgrep.rulesets.read` | Ver reglas disponibles | Hash/licencia conocidos |
| `semgrep.scan.run` | Analizar snapshot | Reglas y lenguaje compatibles |
| `semgrep.findings.read` | Consultar alertas | Cobertura y errores explícitos |

Las lecturas pueden preautorizarse para recursos concretos. Las escrituras o ejecuciones se sujetan a una política explícita por proyecto. Una política previa válida evita pedir confirmación repetida; si falta, preparar una vista previa concreta. La autorización queda ligada a identidad, recurso, parámetros y versión. Texto de un issue, archivo o registro no modifica permisos.

## 5. Superficie externa y contrato del adaptador

| Operación | API, comando o mecanismo | Tratamiento de la respuesta |
|---|---|---|
| Escaneo local | semgrep scan --config <ruleset> --json --metrics=off <snapshot> | Capturar JSON y códigos documentados |
| Versión | semgrep --version | Guardar versión y hash de reglas |
| SARIF posterior | Salida SARIF documentada | Importar con mismo control de artefactos |

La ausencia de --error puede producir exit 0 con findings; clasificar desde JSON y política de hallazgos, sin confundir código de proceso. Timeouts de regla y errores de parseo restan cobertura. No ejecutar configuración remota desde un texto de issue.

**Petición interna ilustrativa:**

```json
{
  "integration_id": "semgrep",
  "project_id": "project-demo",
  "capability": "semgrep.scan.run",
  "input": {
    "snapshot_id": "snapshot-demo",
    "ruleset_id": "web-security-reviewed"
  },
  "idempotency_key": "demo-semgrep-001"
}
```

Se envía al propuesto `POST /api/jobs`, exclusivo de la API administrativa autenticada. Respuesta: HTTP 202 con `job_id`, `status` y `status_url`. `GET /api/jobs/{job_id}` consulta progreso; `POST /api/jobs/{job_id}/cancel` solicita cancelación. Validar esquema y pertenencia antes de consultar credenciales o iniciar procesos. No existe un endpoint que acepte una URL o comando arbitrario del modelo.

**Datos específicos normalizados:** rule_id, severity original, ruta/rango, mensaje saneado, lenguaje, errors/skipped y ruleset_hash.

El resultado incluye `schema_version`, identidad del recurso, instante, versión de herramienta/API, alcance, evidencia saneada, advertencias y estado de cobertura. La respuesta puede terminar con hallazgos aunque el proceso haya funcionado correctamente. Diferenciar ausencia de resultados, permisos insuficientes, cobertura parcial y ejecución fallida.

## 6. Archivos y cambios previstos en AlexIA

Los archivos nuevos se crean durante la implementación. Evitar duplicar contratos del núcleo en el conector.

| Ruta relativa al repositorio | Responsabilidad |
|---|---|
| `src/integrations/semgrep.rs` | CLI y resultado |
| `security/semgrep/` | Reglas revisadas futuras |
| `tests/integrations_semgrep.rs` | Cobertura y códigos |
| `tests/fixtures/semgrep/` | Código sintético |

`src/lib.rs` registraría los módulos; `src/control.rs` delegaría rutas administrativas al núcleo de integraciones. `ui/src/main.js` consumiría estados y resultados; `ui/index.html`/`ui/src/components.js` presentarían la conexión. Conservar las guardas loopback actuales y actualizar [BACKEND.md](../../.agents/.docs/BACKEND.md), [UI.md](../../.agents/.docs/UI.md) y el documento operativo correspondiente al implementar.

## 7. Configuración propuesta

| Campo lógico | Tipo/protección | Uso |
|---|---|---|
| `binary_or_image` | Ruta/digest | Runtime soportado |
| `ruleset_ids` | Lista/hash | Reglas fijas |
| `metrics_enabled` | Booleano | false inicial |
| `max_target_bytes` | Entero | Límite por archivo |
| `rule_timeout_seconds` | Entero | Presupuesto probado |

Estos campos pertenecen a un registro validado de integración, no a un conjunto de secretos nuevos en `.env`. Escribirlos desde la API local con control de revisión. Exponer solamente `configured`, permisos y errores saneados. La presencia de configuración no equivale a conexión verificada.

## 8. Implementación por entregas verificables

### 12.1 — Provisionar reglas

**Trabajo:** Validar licencia, runtime y parser.

**Dependencias:** BC-01 a BC-05 de PLAN00; completar los prerrequisitos de este documento.

**Aceptación:**

- [ ] Reglas remotas no autorizadas rechazadas.
- [ ] Lenguaje ausente informa limitación.

**Verificación:** Fixtures y consulta matriz oficial.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### 12.2 — Escanear caso real

**Trabajo:** Correr snippet vulnerable y corregido.

**Dependencias:** Tarea 12.1 completada y contrato de salida estable.

**Aceptación:**

- [ ] Regla y línea coinciden.
- [ ] Timeout no equivale a limpio.

**Verificación:** Dos fixtures sintéticas y JSON.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### 12.3 — Presentar y recuperar

**Trabajo:** Mostrar cobertura y resultados.

**Dependencias:** Tarea 12.2 completada y contrato de salida estable.

**Aceptación:**

- [ ] Findings no dependen solo de exit code.
- [ ] Autofix nunca se dispara.

**Verificación:** CLI y fallo de runtime.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### Checkpoint de salida

- [ ] Primera operación real completa desde UI hasta proveedor/herramienta y resultado visible.
- [ ] Denegación comprobada al cambiar identidad, proyecto o recurso autorizado.
- [ ] Cancelación, error y desconexión dejan estados coherentes y permiten recuperación.
- [ ] No se modificó el contrato de firma, certificado, timestamp e ID de Alexa.
- [ ] La documentación distingue simulación, prueba real y funciones pendientes.

## 9. Errores, límites y recuperación

Proponer 3 min/job y timeout por regla/archivo. Guardar errores de parseo saneados. No aumentar límites ilimitadamente si una regla consume recursos; marcar incomplete.

Límites iniciales de diseño: una ejecución por proyecto, cola local acotada, cuerpo administrativo de hasta 128 KiB y resultados paginados; los artefactos grandes se entregan por identificador. Los valores se medirán antes de liberar y no describen capacidad demostrada. HTTP 401 expira la sesión; 403 no se corrige ampliando permisos silenciosamente; 429 respeta `Retry-After` cuando exista. Un timeout después de una escritura produce estado `unknown_outcome` hasta reconciliar, sin repetición ciega.

## 10. Riesgos particulares y controles

| Riesgo | Control previsto | Evidencia requerida |
|---|---|---|
| Reglas mutables | Versionar/hash y revisión | Mismo snapshot/ruleset reproducible |
| Cobertura exagerada | Matriz de lenguaje y skipped | Informe con omisiones |
| Telemetría/código cloud | Modo CE local y red restringida | Destinos de red comprobados |

**Datos y retención:** Código permanece local en escaneo CE previsto; cualquier envío al modelo sigue selección y redacción del núcleo. No prometer offline si el provisioning aún descarga reglas.

## 11. Pruebas del adaptador y del flujo completo

**Fixtures y pruebas de contrato:**

- Findings con exit 0, errors y timeout.
- Archivo demasiado grande y lenguaje no soportado.
- Mensaje malicioso y regla remota rechazada.

**Prueba real de aceptación:** Escanear fixture de lenguaje soportado con regla local propia, comparar versión vulnerable/corregida y archivos omitidos.

**Prueba de interfaz con la skill Playwright CLI:**

1. Seleccionar reglas y ver cobertura esperada.
2. Abrir alerta navegable.
3. Ver errores de parseo.
4. Comprobar que botón de corrección prepara vista previa sin escribir.

Usar sesión de navegador desechable; obtener referencias con `snapshot` y ejecutar comprobaciones con `run-code --filename=...` cuando se necesiten aserciones. Para el estado simulado, interceptar únicamente la API propuesta e identificar el resultado como prueba de UI. La prueba real debe comprobar el recurso externo o artefacto local. No automatizar pantallas de login ni guardar cookies del usuario para sustituir APIs oficiales.

Comandos de validación cuando se implemente: `cargo test --locked --test integrations_semgrep` para el archivo propuesto, `cargo check --locked --all-targets`, las pruebas de UI afectadas y `npm --prefix ui run build`. Estos comandos son objetivos; no se afirma que esos tests existan hoy. Añadir pruebas de esquema, autorización y recuperación antes de habilitar capacidades de escritura.

## 12. Despliegue, reversión y criterio de finalización

Habilitar primero para una instalación piloto mediante flag de integración y solamente las capacidades verificadas. Registrar versión y permisos. Después ampliar a un segundo proyecto y comprobar separación de conexiones; no equivale a un despliegue SaaS multiusuario.

**Reversión específica:** Desactivar reglas/adaptador, conservar versión anterior y no aplicar autofix. Recalcular estado si una regla defectuosa se retira.

Deshabilitar el conector impide nuevos trabajos; cancelar o drenar los existentes según su efecto. Revocar credenciales cuando corresponda. Desconectar no borra recursos ajenos ni deshace efectos externos de forma automática. La reversión del software y la compensación de una operación externa son procedimientos distintos.

**Se considera completada cuando:** Resultado reproducible con regla/hash, ubicación y cobertura real; runtime y privacidad documentados.

**Decisiones o condiciones pendientes:** Lenguajes prioritarios y runtime Windows; validar licencia de cada paquete de reglas antes de distribuir.

## 13. Fuentes oficiales y revalidación

Consultadas el 2026-11-03. Sustentan los mecanismos externos; el diseño de AlexIA es una propuesta propia. Revalidar scopes, rutas, versión, licencia y condiciones de distribución antes de implementar. Los enlaces de contratos específicos prevalecen sobre ejemplos generales; una página accesible no demuestra que una cuenta tenga habilitada la función.

- [CLI Semgrep](https://docs.semgrep.dev/cli-reference) — Flags y códigos
- [Reglas locales](https://docs.semgrep.dev/running-rules) — Ejecución y configuración
