# PLAN10 — Integración de Gitleaks con AlexIA

Fecha: 2026-11-03. Estado: **plan propuesto; integración pendiente**. Prioridad: P1, primera validación de secretos.
Audiencia: comunidad técnica, inicialmente instalaciones personales con proyectos autorizados.

## 1. Resultado y alcance

Detectar posibles secretos en archivos o historial y reportar ubicación sin propagar el secreto encontrado.

**Primer recorrido entregable:** Analizar un snapshot autorizado y recibir hallazgos redactados, con explicación para rotar o revocar credenciales fuera de AlexIA.

**Alcance inicial:**

- Modo dir para snapshot y git para historial seleccionado.
- Versión fijada, reglas revisadas y salida JSON redactada.
- Deduplicación mediante fingerprint/hash de ubicación, nunca hash simple del secreto.

**Ampliaciones posteriores:** Baselines y excepciones con responsable, motivo y vencimiento.

**Fuera de esta primera entrega:** Validar credenciales contra servicios reales, imprimir Secret/Match y borrar historial automáticamente.

## 2. Punto de partida real y prerrequisitos

No hay ejecutable ni adaptador Gitleaks declarado. La UI que oculta variables no sustituye un escaneo del repositorio.

La interfaz actual administra configuración; no existe todavía el catálogo ejecutable de conectores ni la API de trabajos descrita aquí. El estado actual y las dependencias transversales están en [PLAN00 — Base común](PLAN00-BASE-COMUN.md). Las rutas, tipos, módulos y opciones nuevos que siguen son propuestas de implementación, no contratos existentes.

**Prerrequisitos específicos:**

- Runner y snapshot con secretos sintéticos de prueba.
- Binario verificado por checksum y versión.
- Reglas elegidas y política de historial/redacción.

**Dependencias entre planes:** PLAN00 y ejecución PLAN03/04; no necesita cuenta cloud.

## 3. Decisión técnica y flujo de conexión

Invocar binario directamente sobre snapshot/copia permitida y generar JSON en directorio privado del job. Activar redacción del CLI y volver a filtrar campos en parser, pues configuración o versión puede variar. Separar secreto detectado de exposición confirmada: un match es hallazgo potencial. No enviar la coincidencia al modelo.

**Procedimiento de conexión o habilitación:**

1. Detectar versión y confirmar flags con --help de la versión fijada.
2. Elegir dir o git y mostrar alcance; historial puede ampliar mucho el análisis.
3. Preparar copia/checkout sin hooks y reglas con hash.
4. Ejecutar con --redact y reporte JSON propio.
5. Parsear solo RuleID, ruta, líneas, commit y evidencia segura.
6. Eliminar reporte crudo una vez normalizado y ofrecer acciones orientativas de rotación.

**Autenticación, renovación y revocación:** No requiere autenticación para análisis local. No necesita tokens del repositorio remoto si se usa una copia local ya autorizada.

**Permisos externos y selección de recursos:** Lectura de snapshot e historial seleccionado; escritura solo de artefacto propio. Cambiar configuración o baseline requiere permiso independiente.

Las credenciales residen en el almacén del sistema descrito en PLAN00; la configuración conserva solamente referencias. El navegador, el modelo y la respuesta de Alexa no reciben tokens. Una instalación comunitaria no comparte credenciales con otra. Si el producto exige un servicio OAuth confidencial, su secreto permanece en ese servicio; nunca se distribuye en el frontend, extensión o binario público.

## 4. Capacidades autorizables

Los identificadores de esta tabla son capacidades internas de AlexIA; no son nombres de scopes del proveedor.

| Capacidad propuesta | Operación | Guarda específica |
|---|---|---|
| `gitleaks.scan.directory` | Analizar archivos | Raíz/snapshot autorizados |
| `gitleaks.scan.history` | Analizar historial elegido | Rango y presupuesto visibles |
| `gitleaks.findings.read` | Leer hallazgos redactados | Sin Secret ni Match crudos |

Las lecturas pueden preautorizarse para recursos concretos. Las escrituras o ejecuciones se sujetan a una política explícita por proyecto. Una política previa válida evita pedir confirmación repetida; si falta, preparar una vista previa concreta. La autorización queda ligada a identidad, recurso, parámetros y versión. Texto de un issue, archivo o registro no modifica permisos.

## 5. Superficie externa y contrato del adaptador

| Operación | API, comando o mecanismo | Tratamiento de la respuesta |
|---|---|---|
| Snapshot | gitleaks dir <snapshot> --redact --report-format json --report-path <report> | Confirmar flags en versión fijada |
| Historial | gitleaks git <repo> con rango de log validado | No interpolar opciones de git |
| Versión | gitleaks version | Registrar versión y checksum |

Asignar código de hallazgos explícito soportado por versión y distinguirlo de error operativo. No adoptar comandos históricos detect/protect sin revisar CLI actual. El parser rechaza salida malformada y no representa error como lista vacía.

**Petición interna ilustrativa:**

```json
{
  "integration_id": "gitleaks",
  "project_id": "project-demo",
  "capability": "gitleaks.scan.directory",
  "input": {
    "snapshot_id": "snapshot-demo",
    "mode": "directory",
    "ruleset_id": "approved-secrets"
  },
  "idempotency_key": "demo-gitleaks-001"
}
```

Se envía al propuesto `POST /api/jobs`, exclusivo de la API administrativa autenticada. Respuesta: HTTP 202 con `job_id`, `status` y `status_url`. `GET /api/jobs/{job_id}` consulta progreso; `POST /api/jobs/{job_id}/cancel` solicita cancelación. Validar esquema y pertenencia antes de consultar credenciales o iniciar procesos. No existe un endpoint que acepte una URL o comando arbitrario del modelo.

**Datos específicos normalizados:** rule_id, ruta relativa, línea, commit si aplica, fingerprint del hallazgo, redacted=true y coverage.

El resultado incluye `schema_version`, identidad del recurso, instante, versión de herramienta/API, alcance, evidencia saneada, advertencias y estado de cobertura. La respuesta puede terminar con hallazgos aunque el proceso haya funcionado correctamente. Diferenciar ausencia de resultados, permisos insuficientes, cobertura parcial y ejecución fallida.

## 6. Archivos y cambios previstos en AlexIA

Los archivos nuevos se crean durante la implementación. Evitar duplicar contratos del núcleo en el conector.

| Ruta relativa al repositorio | Responsabilidad |
|---|---|
| `src/integrations/gitleaks.rs` | Invocación y parser seguro |
| `tests/integrations_gitleaks.rs` | Redacción y códigos |
| `tests/fixtures/gitleaks/` | Secretos sintéticos y reportes |

`src/lib.rs` registraría los módulos; `src/control.rs` delegaría rutas administrativas al núcleo de integraciones. `ui/src/main.js` consumiría estados y resultados; `ui/index.html`/`ui/src/components.js` presentarían la conexión. Conservar las guardas loopback actuales y actualizar [BACKEND.md](../../.agents/.docs/BACKEND.md), [UI.md](../../.agents/.docs/UI.md) y el documento operativo correspondiente al implementar.

## 7. Configuración propuesta

| Campo lógico | Tipo/protección | Uso |
|---|---|---|
| `binary_path` | Ruta/checksum | Binario confiable |
| `ruleset_hash` | Hash | Reglas aprobadas |
| `scan_mode` | Enum | directory o history |
| `baseline_id` | Opcional | Excepciones revisadas |

Estos campos pertenecen a un registro validado de integración, no a un conjunto de secretos nuevos en `.env`. Escribirlos desde la API local con control de revisión. Exponer solamente `configured`, permisos y errores saneados. La presencia de configuración no equivale a conexión verificada.

## 8. Implementación por entregas verificables

### 10.1 — Detectar y normalizar

**Trabajo:** Añadir parser que elimina campos peligrosos.

**Dependencias:** BC-01 a BC-05 de PLAN00; completar los prerrequisitos de este documento.

**Aceptación:**

- [ ] Secret/Match nunca salen.
- [ ] JSON inválido produce fallo.

**Verificación:** Fixture con centinela en varios campos.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### 10.2 — Escanear snapshot real

**Trabajo:** Integrar dir e historial acotado.

**Dependencias:** Tarea 10.1 completada y contrato de salida estable.

**Aceptación:**

- [ ] Match sintético aparece redactado.
- [ ] Modo elegido respeta alcance.

**Verificación:** Repositorio temporal con commit fixture.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### 10.3 — UI y excepciones

**Trabajo:** Presentar resultados y preparar baseline posterior.

**Dependencias:** Tarea 10.2 completada y contrato de salida estable.

**Aceptación:**

- [ ] Cero hallazgos distingue error.
- [ ] No se oculta alerta sin motivo.

**Verificación:** CLI y búsqueda de centinelas en artefactos.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### Checkpoint de salida

- [ ] Primera operación real completa desde UI hasta proveedor/herramienta y resultado visible.
- [ ] Denegación comprobada al cambiar identidad, proyecto o recurso autorizado.
- [ ] Cancelación, error y desconexión dejan estados coherentes y permiten recuperación.
- [ ] No se modificó el contrato de firma, certificado, timestamp e ID de Alexa.
- [ ] La documentación distingue simulación, prueba real y funciones pendientes.

## 9. Errores, límites y recuperación

Proponer 60 s snapshot y 3 min historial pequeño; máximo de archivos y tamaño explícitos. No activar logs verbose con contenido sensible. Un reporte parcial conserva estado partial y archivos omitidos.

Límites iniciales de diseño: una ejecución por proyecto, cola local acotada, cuerpo administrativo de hasta 128 KiB y resultados paginados; los artefactos grandes se entregan por identificador. Los valores se medirán antes de liberar y no describen capacidad demostrada. HTTP 401 expira la sesión; 403 no se corrige ampliando permisos silenciosamente; 429 respeta `Retry-After` cuando exista. Un timeout después de una escritura produce estado `unknown_outcome` hasta reconciliar, sin repetición ciega.

## 10. Riesgos particulares y controles

| Riesgo | Control previsto | Evidencia requerida |
|---|---|---|
| Reporte contiene secreto | --redact más filtrado del parser | Centinela ausente en UI/logs |
| Falso positivo | Regla/evidencia y triage humano | Excepción con motivo |
| Escaneo desborda historial | Rango/presupuesto | Timeout marcado parcial |

**Datos y retención:** No guardar valores ni snippets que contengan coincidencias completas. Los fingerprints se basan en metadatos de hallazgo; no almacenar hashes vulnerables a diccionario del secreto.

## 11. Pruebas del adaptador y del flujo completo

**Fixtures y pruebas de contrato:**

- Códigos de éxito/hallazgo/error.
- Secreto sintético en Secret, Match y mensajes.
- Historial incompleto, ruta fuera de snapshot y baseline obsoleto.

**Prueba real de aceptación:** Escanear repo temporal con token sintético válido para regla fixture y archivo limpio; comprobar detección sin valor visible.

**Prueba de interfaz con la skill Playwright CLI:**

1. Ejecutar escaneo fixture.
2. Abrir hallazgo y ver regla/ruta/línea.
3. Comprobar que no existe botón que revele valor.
4. Verificar error de herramienta frente a cero hallazgos.

Usar sesión de navegador desechable; obtener referencias con `snapshot` y ejecutar comprobaciones con `run-code --filename=...` cuando se necesiten aserciones. Para el estado simulado, interceptar únicamente la API propuesta e identificar el resultado como prueba de UI. La prueba real debe comprobar el recurso externo o artefacto local. No automatizar pantallas de login ni guardar cookies del usuario para sustituir APIs oficiales.

Comandos de validación cuando se implemente: `cargo test --locked --test integrations_gitleaks` para el archivo propuesto, `cargo check --locked --all-targets`, las pruebas de UI afectadas y `npm --prefix ui run build`. Estos comandos son objetivos; no se afirma que esos tests existan hoy. Añadir pruebas de esquema, autorización y recuperación antes de habilitar capacidades de escritura.

## 12. Despliegue, reversión y criterio de finalización

Habilitar primero para una instalación piloto mediante flag de integración y solamente las capacidades verificadas. Registrar versión y permisos. Después ampliar a un segundo proyecto y comprobar separación de conexiones; no equivale a un despliegue SaaS multiusuario.

**Reversión específica:** Desactivar escáner, borrar artefactos crudos y revocar tareas; no alterar historial Git ni intentar rotar credenciales automáticamente.

Deshabilitar el conector impide nuevos trabajos; cancelar o drenar los existentes según su efecto. Revocar credenciales cuando corresponda. Desconectar no borra recursos ajenos ni deshace efectos externos de forma automática. La reversión del software y la compensación de una operación externa son procedimientos distintos.

**Se considera completada cuando:** Detecta fixture real y no filtra secreto en ningún resultado, log o envío al modelo.

**Decisiones o condiciones pendientes:** Versión/binario y reglas iniciales; no instalar ni escanear secretos reales durante esta tarea documental.

## 13. Fuentes oficiales y revalidación

Consultadas el 2026-11-03. Sustentan los mecanismos externos; el diseño de AlexIA es una propuesta propia. Revalidar scopes, rutas, versión, licencia y condiciones de distribución antes de implementar. Los enlaces de contratos específicos prevalecen sobre ejemplos generales; una página accesible no demuestra que una cuenta tenga habilitada la función.

- [Repositorio oficial Gitleaks](https://github.com/gitleaks/gitleaks) — CLI, redacción y formato
