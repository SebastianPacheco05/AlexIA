# PLAN13 — Integración de Trivy con AlexIA

Fecha: 2026-11-03. Estado: **plan propuesto; integración pendiente**. Prioridad: P2, imágenes y configuración.
Audiencia: comunidad técnica, inicialmente instalaciones personales con proyectos autorizados.

## 1. Resultado y alcance

Analizar imágenes o filesystem con alcance explícito y resultados de vulnerabilidades/configuración identificables.

**Primer recorrido entregable:** Elegir imagen por digest o snapshot, ejecutar scanners concretos y revisar hallazgos con versión de base.

**Alcance inicial:**

- Scanners vuln y misconfig seleccionados explícitamente.
- Imagen por digest o filesystem permitido.
- Base/cache propia y resultados JSON.

**Ampliaciones posteriores:** SBOM y comparación entre versiones; scanner secrets solo con redacción equivalente a PLAN10.

**Fuera de esta primera entrega:** Acceso global a clusters/cloud, actualización automática de imágenes y escaneo de toda la máquina.

## 2. Punto de partida real y prerrequisitos

Dockerfile existe, sin Trivy. Un análisis del Dockerfile no demuestra seguridad de la imagen construida ni una imagen ejecutable.

La interfaz actual administra configuración; no existe todavía el catálogo ejecutable de conectores ni la API de trabajos descrita aquí. El estado actual y las dependencias transversales están en [PLAN00 — Base común](PLAN00-BASE-COMUN.md). Las rutas, tipos, módulos y opciones nuevos que siguen son propuestas de implementación, no contratos existentes.

**Prerrequisitos específicos:**

- Binario o imagen Trivy fijada.
- Base de vulnerabilidades provisionada y fecha visible.
- Imagen fixture por digest o filesystem de pruebas.

**Dependencias entre planes:** PLAN00 y ejecución; PLAN06 si se consulta daemon/imagen local.

## 3. Decisión técnica y flujo de conexión

Adaptador con target_kind=image/filesystem y scanners explícitos. No depender de defaults que puedan activar secrets u otros datos. Descargar base en fase separada con origen validado; analizar con cache propia. Evitar socket Docker si se puede analizar un archivo OCI/imagen por digest sin acceso al daemon.

**Procedimiento de conexión o habilitación:**

1. Detectar versión y capacidades.
2. Actualizar base de forma autorizada y registrar metadatos.
3. Seleccionar target inmutable y scanners.
4. Ejecutar con --format json y destino de reporte propio.
5. Normalizar Results/Vulnerabilities/Misconfigurations sin asumir campos siempre presentes.
6. Mostrar digest, base, findings y archivos/paquetes no cubiertos.

**Autenticación, renovación y revocación:** Registro público sin credenciales cuando aplique; privado mediante referencia limitada al registro. No exportar docker config ni credenciales cloud.

**Permisos externos y selección de recursos:** Lectura de target y red al registro/base; sin permisos para push, cluster o administración.

Las credenciales residen en el almacén del sistema descrito en PLAN00; la configuración conserva solamente referencias. El navegador, el modelo y la respuesta de Alexa no reciben tokens. Una instalación comunitaria no comparte credenciales con otra. Si el producto exige un servicio OAuth confidencial, su secreto permanece en ese servicio; nunca se distribuye en el frontend, extensión o binario público.

## 4. Capacidades autorizables

Los identificadores de esta tabla son capacidades internas de AlexIA; no son nombres de scopes del proveedor.

| Capacidad propuesta | Operación | Guarda específica |
|---|---|---|
| `trivy.image.scan` | Analizar imagen | Digest y registro permitidos |
| `trivy.filesystem.scan` | Analizar snapshot | Raíz y scanners fijos |
| `trivy.database.refresh` | Actualizar base | Fuente/hash/fecha verificados |

Las lecturas pueden preautorizarse para recursos concretos. Las escrituras o ejecuciones se sujetan a una política explícita por proyecto. Una política previa válida evita pedir confirmación repetida; si falta, preparar una vista previa concreta. La autorización queda ligada a identidad, recurso, parámetros y versión. Texto de un issue, archivo o registro no modifica permisos.

## 5. Superficie externa y contrato del adaptador

| Operación | API, comando o mecanismo | Tratamiento de la respuesta |
|---|---|---|
| Filesystem | trivy fs --scanners vuln,misconfig --format json <snapshot> | Salida y estado de cobertura |
| Imagen | trivy image --scanners vuln,misconfig --format json <digest-ref> | Identificar digest real |
| Base/cache | Flags de base y cache de versión fijada | No fallback silencioso a red |

Configurar exit code de findings distinto del error operacional y validar reporte aun si hay hallazgos. La severidad y fixed version dependen de fuente/ecosistema; conservar origen. No deducir estado limpio si la base o el extractor falló.

**Petición interna ilustrativa:**

```json
{
  "integration_id": "trivy",
  "project_id": "project-demo",
  "capability": "trivy.filesystem.scan",
  "input": {
    "target_kind": "filesystem",
    "snapshot_id": "snapshot-demo",
    "scanners": [
      "vuln",
      "misconfig"
    ]
  },
  "idempotency_key": "demo-trivy-001"
}
```

Se envía al propuesto `POST /api/jobs`, exclusivo de la API administrativa autenticada. Respuesta: HTTP 202 con `job_id`, `status` y `status_url`. `GET /api/jobs/{job_id}` consulta progreso; `POST /api/jobs/{job_id}/cancel` solicita cancelación. Validar esquema y pertenencia antes de consultar credenciales o iniciar procesos. No existe un endpoint que acepte una URL o comando arbitrario del modelo.

**Datos específicos normalizados:** target/digest, scanner, package/rule, advisory, severity/source, fixed_version, DB metadata y cobertura.

El resultado incluye `schema_version`, identidad del recurso, instante, versión de herramienta/API, alcance, evidencia saneada, advertencias y estado de cobertura. La respuesta puede terminar con hallazgos aunque el proceso haya funcionado correctamente. Diferenciar ausencia de resultados, permisos insuficientes, cobertura parcial y ejecución fallida.

## 6. Archivos y cambios previstos en AlexIA

Los archivos nuevos se crean durante la implementación. Evitar duplicar contratos del núcleo en el conector.

| Ruta relativa al repositorio | Responsabilidad |
|---|---|
| `src/integrations/trivy.rs` | Targets y parser |
| `tests/integrations_trivy.rs` | JSON y base |
| `tests/fixtures/trivy/` | Imagen/reportes sintéticos |

`src/lib.rs` registraría los módulos; `src/control.rs` delegaría rutas administrativas al núcleo de integraciones. `ui/src/main.js` consumiría estados y resultados; `ui/index.html`/`ui/src/components.js` presentarían la conexión. Conservar las guardas loopback actuales y actualizar [BACKEND.md](../../.agents/.docs/BACKEND.md), [UI.md](../../.agents/.docs/UI.md) y el documento operativo correspondiente al implementar.

## 7. Configuración propuesta

| Campo lógico | Tipo/protección | Uso |
|---|---|---|
| `binary_or_image` | Ruta/digest | Versión fijada |
| `cache_root` | Ruta propia | Base y TTL |
| `allowed_registries` | Lista | Egress permitido |
| `enabled_scanners` | Lista | vuln/misconfig inicial |
| `db_freshness_policy` | Objeto | Edad tolerada visible |

Estos campos pertenecen a un registro validado de integración, no a un conjunto de secretos nuevos en `.env`. Escribirlos desde la API local con control de revisión. Exponer solamente `configured`, permisos y errores saneados. La presencia de configuración no equivale a conexión verificada.

## 8. Implementación por entregas verificables

### 13.1 — Preparar target y base

**Trabajo:** Detección y actualización explícita.

**Dependencias:** BC-01 a BC-05 de PLAN00; completar los prerrequisitos de este documento.

**Aceptación:**

- [ ] No base produce no disponible.
- [ ] Digest fijado, no solo latest.

**Verificación:** Fixtures de metadatos y target.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### 13.2 — Escanear y normalizar

**Trabajo:** Ejecutar target real fixture.

**Dependencias:** Tarea 13.1 completada y contrato de salida estable.

**Aceptación:**

- [ ] JSON de ambos scanners se distingue.
- [ ] Fixed version desconocida no se inventa.

**Verificación:** Imagen/snapshot fixture.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### 13.3 — Recuperar y mostrar

**Trabajo:** Probar offline/credenciales/estado parcial.

**Dependencias:** Tarea 13.2 completada y contrato de salida estable.

**Aceptación:**

- [ ] No imprime credenciales registro.
- [ ] Fecha base visible.

**Verificación:** Red bloqueada y UI CLI.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### Checkpoint de salida

- [ ] Primera operación real completa desde UI hasta proveedor/herramienta y resultado visible.
- [ ] Denegación comprobada al cambiar identidad, proyecto o recurso autorizado.
- [ ] Cancelación, error y desconexión dejan estados coherentes y permiten recuperación.
- [ ] No se modificó el contrato de firma, certificado, timestamp e ID de Alexa.
- [ ] La documentación distingue simulación, prueba real y funciones pendientes.

## 9. Errores, límites y recuperación

Proponer 5 min por imagen, límite de tamaño/pull explícito. Actualización de DB con exclusión mutua; no duplicar descargas. Base caducada queda advertida o bloqueada según política.

Límites iniciales de diseño: una ejecución por proyecto, cola local acotada, cuerpo administrativo de hasta 128 KiB y resultados paginados; los artefactos grandes se entregan por identificador. Los valores se medirán antes de liberar y no describen capacidad demostrada. HTTP 401 expira la sesión; 403 no se corrige ampliando permisos silenciosamente; 429 respeta `Retry-After` cuando exista. Un timeout después de una escritura produce estado `unknown_outcome` hasta reconciliar, sin repetición ciega.

## 10. Riesgos particulares y controles

| Riesgo | Control previsto | Evidencia requerida |
|---|---|---|
| Imagen mutable | Resolver digest antes de autorizar | Digest en evidencia |
| Base ausente/antigua | Frescura visible | Caso offline |
| Scanner secrets inesperado | Lista cerrada | Salida no contiene secretos crudos |

**Datos y retención:** Inventario y configuración son privados; secretos de registro solo al proceso autorizado. Reportes crudos protegidos y TTL.

## 11. Pruebas del adaptador y del flujo completo

**Fixtures y pruebas de contrato:**

- JSON con campos ausentes y varios Results.
- Base antigua, pull no autorizado y digest distinto.
- Findings versus exit de fallo; cache compartida concurrente.

**Prueba real de aceptación:** Analizar imagen fixture por digest y Dockerfile sintético; comprobar fuente de vulnerabilidad y regla de configuración.

**Prueba de interfaz con la skill Playwright CLI:**

1. Elegir target/scanners.
2. Ver digest y fecha base.
3. Abrir finding y corrección documentada.
4. Simular fallo de base sin mostrar cero vulnerabilidades.

Usar sesión de navegador desechable; obtener referencias con `snapshot` y ejecutar comprobaciones con `run-code --filename=...` cuando se necesiten aserciones. Para el estado simulado, interceptar únicamente la API propuesta e identificar el resultado como prueba de UI. La prueba real debe comprobar el recurso externo o artefacto local. No automatizar pantallas de login ni guardar cookies del usuario para sustituir APIs oficiales.

Comandos de validación cuando se implemente: `cargo test --locked --test integrations_trivy` para el archivo propuesto, `cargo check --locked --all-targets`, las pruebas de UI afectadas y `npm --prefix ui run build`. Estos comandos son objetivos; no se afirma que esos tests existan hoy. Añadir pruebas de esquema, autorización y recuperación antes de habilitar capacidades de escritura.

## 12. Despliegue, reversión y criterio de finalización

Habilitar primero para una instalación piloto mediante flag de integración y solamente las capacidades verificadas. Registrar versión y permisos. Después ampliar a un segundo proyecto y comprobar separación de conexiones; no equivale a un despliegue SaaS multiusuario.

**Reversión específica:** Deshabilitar adapter y limpiar cache propia sin borrar imágenes o cache Docker del usuario.

Deshabilitar el conector impide nuevos trabajos; cancelar o drenar los existentes según su efecto. Revocar credenciales cuando corresponda. Desconectar no borra recursos ajenos ni deshace efectos externos de forma automática. La reversión del software y la compensación de una operación externa son procedimientos distintos.

**Se considera completada cuando:** Target real identificable produce resultados con base/cobertura; errores y credenciales están controlados.

**Decisiones o condiciones pendientes:** Presupuesto de disco/base, soporte offline y licencia de distribución pendientes de piloto.

## 13. Fuentes oficiales y revalidación

Consultadas el 2026-11-03. Sustentan los mecanismos externos; el diseño de AlexIA es una propuesta propia. Revalidar scopes, rutas, versión, licencia y condiciones de distribución antes de implementar. Los enlaces de contratos específicos prevalecen sobre ejemplos generales; una página accesible no demuestra que una cuenta tenga habilitada la función.

- [Guía Trivy](https://trivy.dev/docs/latest/guide/) — Targets y scanners
- [CLI filesystem](https://trivy.dev/docs/latest/guide/references/configuration/cli/trivy_filesystem/) — Flags y salidas
