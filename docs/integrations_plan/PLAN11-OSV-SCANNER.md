# PLAN11 — Integración de OSV-Scanner con AlexIA

Fecha: 2026-11-03. Estado: **plan propuesto; integración pendiente**. Prioridad: P1, dependencias vulnerables.
Audiencia: comunidad técnica, inicialmente instalaciones personales con proyectos autorizados.

## 1. Resultado y alcance

Relacionar dependencias fijadas con vulnerabilidades conocidas y mostrar evidencia de versiones afectadas.

**Primer recorrido entregable:** Escanear lockfiles de un snapshot y recibir paquetes, avisos y versiones corregidas cuando estén documentadas.

**Alcance inicial:**

- OSV-Scanner v2, lockfiles soportados y salida JSON.
- Cargo.lock y package-lock.json de proyectos como AlexIA, previa comprobación de soporte.
- Online explícito u offline con base identificada; cobertura por archivo.

**Ampliaciones posteriores:** Diferenciar dependencia directa/transitiva, comparación entre ramas y remediación propuesta.

**Fuera de esta primera entrega:** Actualizar dependencias automáticamente, afirmar explotabilidad solo por un CVE o ejecutar instaladores.

## 2. Punto de partida real y prerrequisitos

Existen Cargo.lock y ui/package-lock.json; no hay adaptador OSV. npm audit histórico no demuestra estado actual ni cubre todo el proyecto.

La interfaz actual administra configuración; no existe todavía el catálogo ejecutable de conectores ni la API de trabajos descrita aquí. El estado actual y las dependencias transversales están en [PLAN00 — Base común](PLAN00-BASE-COMUN.md). Las rutas, tipos, módulos y opciones nuevos que siguen son propuestas de implementación, no contratos existentes.

**Prerrequisitos específicos:**

- Runner y binario OSV fijado.
- Fixtures de lockfiles con paquetes afectados conocidos y versiones limpias.
- Decisión de modo online/offline y privacidad del inventario.

**Dependencias entre planes:** PLAN00 y PLAN03/04; PLAN06 opcional.

## 3. Decisión técnica y flujo de conexión

Extraer dependencias mediante el scanner sin instalar ni ejecutar paquetes. La versión v2 organiza escaneo bajo subcomandos scan; confirmar flags y JSON de la versión elegida, sin copiar comandos v1. Mantener resultado bruto protegido y modelo normalizado con aliases de avisos para evitar duplicados. La base usada y su fecha forman parte de la evidencia.

**Procedimiento de conexión o habilitación:**

1. Detectar versión y formato soportado mediante --help y documentación v2.
2. Descubrir lockfiles dentro de raíz y mostrar cuáles se procesarán.
3. Elegir online con consentimiento de salida de inventario o base offline provisionada.
4. Ejecutar scan source sobre snapshot con salida JSON mediante argv compatible.
5. Normalizar paquetes, avisos, rangos/versiones y aliases.
6. Mostrar cobertura y recomendaciones sin modificar manifests.

**Autenticación, renovación y revocación:** El análisis público OSV no requiere copiar credenciales de Google. Conservar autenticación solo si una fuente específica la exige en el futuro.

**Permisos externos y selección de recursos:** Lectura de manifiestos/lockfiles; red solo hacia fuentes de vulnerabilidades previstas. No ejecutar scripts de instalación para completar datos.

Las credenciales residen en el almacén del sistema descrito en PLAN00; la configuración conserva solamente referencias. El navegador, el modelo y la respuesta de Alexa no reciben tokens. Una instalación comunitaria no comparte credenciales con otra. Si el producto exige un servicio OAuth confidencial, su secreto permanece en ese servicio; nunca se distribuye en el frontend, extensión o binario público.

## 4. Capacidades autorizables

Los identificadores de esta tabla son capacidades internas de AlexIA; no son nombres de scopes del proveedor.

| Capacidad propuesta | Operación | Guarda específica |
|---|---|---|
| `osv-scanner.dependencies.scan` | Escanear dependencias | Snapshot y modo de red elegidos |
| `osv-scanner.findings.read` | Consultar avisos | Fuentes y fecha visibles |
| `osv-scanner.database.refresh` | Actualizar base offline | Descarga explícita y verificada |

Las lecturas pueden preautorizarse para recursos concretos. Las escrituras o ejecuciones se sujetan a una política explícita por proyecto. Una política previa válida evita pedir confirmación repetida; si falta, preparar una vista previa concreta. La autorización queda ligada a identidad, recurso, parámetros y versión. Texto de un issue, archivo o registro no modifica permisos.

## 5. Superficie externa y contrato del adaptador

| Operación | API, comando o mecanismo | Tratamiento de la respuesta |
|---|---|---|
| Análisis v2 | osv-scanner scan source --recursive --format=json --output-file <reporte> <snapshot> | Confirmar orden/flags exactos en versión fijada |
| Salida | Formato JSON documentado de v2 | Normalizar paquetes y aliases |
| Offline | Modo offline según versión/base instalada | No hacer fallback online silencioso |

El comando de producción se fija después de verificar --help de la versión instalada. No copiar combinaciones de flags de v1 sin validar su posición y semántica en v2; la documentación actual también muestra alias de compatibilidad. Distinguir paquete detectado, soporte de extractor y coincidencia de aviso. Un lockfile omitido produce advertencia, no proyecto seguro.

**Petición interna ilustrativa:**

```json
{
  "integration_id": "osv-scanner",
  "project_id": "project-demo",
  "capability": "osv-scanner.dependencies.scan",
  "input": {
    "snapshot_id": "snapshot-demo",
    "mode": "online",
    "manifest_ids": [
      "cargo-lock",
      "ui-lock"
    ]
  },
  "idempotency_key": "demo-osv-scanner-001"
}
```

Se envía al propuesto `POST /api/jobs`, exclusivo de la API administrativa autenticada. Respuesta: HTTP 202 con `job_id`, `status` y `status_url`. `GET /api/jobs/{job_id}` consulta progreso; `POST /api/jobs/{job_id}/cancel` solicita cancelación. Validar esquema y pertenencia antes de consultar credenciales o iniciar procesos. No existe un endpoint que acepte una URL o comando arbitrario del modelo.

**Datos específicos normalizados:** ecosystem, package/name/version, source lockfile, advisory_id/aliases, rangos, fix conocido, base/fecha y cobertura.

El resultado incluye `schema_version`, identidad del recurso, instante, versión de herramienta/API, alcance, evidencia saneada, advertencias y estado de cobertura. La respuesta puede terminar con hallazgos aunque el proceso haya funcionado correctamente. Diferenciar ausencia de resultados, permisos insuficientes, cobertura parcial y ejecución fallida.

## 6. Archivos y cambios previstos en AlexIA

Los archivos nuevos se crean durante la implementación. Evitar duplicar contratos del núcleo en el conector.

| Ruta relativa al repositorio | Responsabilidad |
|---|---|
| `src/integrations/osv_scanner.rs` | Detección y parser v2 |
| `tests/integrations_osv_scanner.rs` | Aliases/cobertura |
| `tests/fixtures/osv/` | Lockfiles y respuestas versionadas |

`src/lib.rs` registraría los módulos; `src/control.rs` delegaría rutas administrativas al núcleo de integraciones. `ui/src/main.js` consumiría estados y resultados; `ui/index.html`/`ui/src/components.js` presentarían la conexión. Conservar las guardas loopback actuales y actualizar [BACKEND.md](../../.agents/.docs/BACKEND.md), [UI.md](../../.agents/.docs/UI.md) y el documento operativo correspondiente al implementar.

## 7. Configuración propuesta

| Campo lógico | Tipo/protección | Uso |
|---|---|---|
| `binary_path` | Ruta/checksum | OSV v2 fijado |
| `network_mode` | Enum | online/offline |
| `database_path` | Ruta | Base offline provisionada |
| `supported_extractors` | Lista | Cobertura probada |

Estos campos pertenecen a un registro validado de integración, no a un conjunto de secretos nuevos en `.env`. Escribirlos desde la API local con control de revisión. Exponer solamente `configured`, permisos y errores saneados. La presencia de configuración no equivale a conexión verificada.

## 8. Implementación por entregas verificables

### 11.1 — Validar formatos

**Trabajo:** Detectar lockfiles y parser v2.

**Dependencias:** BC-01 a BC-05 de PLAN00; completar los prerrequisitos de este documento.

**Aceptación:**

- [ ] Omitidos visibles.
- [ ] Aliases no duplican hallazgos.

**Verificación:** Fixtures Cargo/npm y formato inválido.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### 11.2 — Escanear con evidencia

**Trabajo:** Ejecutar snapshot y registrar base.

**Dependencias:** Tarea 11.1 completada y contrato de salida estable.

**Aceptación:**

- [ ] Paquete afectado identifica versión.
- [ ] No instala dependencias.

**Verificación:** Fixture real con advisories controlados.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### 11.3 — Comparar y recuperar

**Trabajo:** Manejar base ausente/red y UI.

**Dependencias:** Tarea 11.2 completada y contrato de salida estable.

**Aceptación:**

- [ ] Offline no consulta red.
- [ ] Base vencida se señala.

**Verificación:** Red bloqueada y CLI del panel.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### Checkpoint de salida

- [ ] Primera operación real completa desde UI hasta proveedor/herramienta y resultado visible.
- [ ] Denegación comprobada al cambiar identidad, proyecto o recurso autorizado.
- [ ] Cancelación, error y desconexión dejan estados coherentes y permiten recuperación.
- [ ] No se modificó el contrato de firma, certificado, timestamp e ID de Alexa.
- [ ] La documentación distingue simulación, prueba real y funciones pendientes.

## 9. Errores, límites y recuperación

Deadline propuesto 2 min; separar extracción de consulta de avisos. Cachear únicamente según política del proyecto y base identificada. Una caída de red deja incomplete, no cero hallazgos.

Límites iniciales de diseño: una ejecución por proyecto, cola local acotada, cuerpo administrativo de hasta 128 KiB y resultados paginados; los artefactos grandes se entregan por identificador. Los valores se medirán antes de liberar y no describen capacidad demostrada. HTTP 401 expira la sesión; 403 no se corrige ampliando permisos silenciosamente; 429 respeta `Retry-After` cuando exista. Un timeout después de una escritura produce estado `unknown_outcome` hasta reconciliar, sin repetición ciega.

## 10. Riesgos particulares y controles

| Riesgo | Control previsto | Evidencia requerida |
|---|---|---|
| Inventario enviado sin advertencia | Modo explícito y perfil de red | Captura de destinos autorizados |
| CVE interpretado como explotación | Mostrar afectación y límites | Informe evita certeza no respaldada |
| Base desactualizada | Fecha y política de freshness | Fixture base antigua |

**Datos y retención:** Dependencias revelan tecnología privada; mostrar salida de inventario. Enviar al modelo solo hallazgos relevantes, no archivos completos.

## 11. Pruebas del adaptador y del flujo completo

**Fixtures y pruebas de contrato:**

- Aliases CVE/GHSA/OSV y rangos distintos.
- Formato v1 rechazado por parser v2.
- Sin base offline, red caída y extractor no soportado.

**Prueba real de aceptación:** Analizar lockfile fixture con versión afectada y otra corregida; comparar aviso y fuente. Probar offline sin conexión si se ofrece ese modo.

**Prueba de interfaz con la skill Playwright CLI:**

1. Elegir lockfiles y modo.
2. Ver aviso, paquete y evidencia.
3. Mostrar cobertura parcial.
4. Comprobar que no se actualiza paquete al abrir recomendación.

Usar sesión de navegador desechable; obtener referencias con `snapshot` y ejecutar comprobaciones con `run-code --filename=...` cuando se necesiten aserciones. Para el estado simulado, interceptar únicamente la API propuesta e identificar el resultado como prueba de UI. La prueba real debe comprobar el recurso externo o artefacto local. No automatizar pantallas de login ni guardar cookies del usuario para sustituir APIs oficiales.

Comandos de validación cuando se implemente: `cargo test --locked --test integrations_osv_scanner` para el archivo propuesto, `cargo check --locked --all-targets`, las pruebas de UI afectadas y `npm --prefix ui run build`. Estos comandos son objetivos; no se afirma que esos tests existan hoy. Añadir pruebas de esquema, autorización y recuperación antes de habilitar capacidades de escritura.

## 12. Despliegue, reversión y criterio de finalización

Habilitar primero para una instalación piloto mediante flag de integración y solamente las capacidades verificadas. Registrar versión y permisos. Después ampliar a un segundo proyecto y comprobar separación de conexiones; no equivale a un despliegue SaaS multiusuario.

**Reversión específica:** Desactivar scanner y limpiar base/cache propia según política; mantener lockfiles intactos.

Deshabilitar el conector impide nuevos trabajos; cancelar o drenar los existentes según su efecto. Revocar credenciales cuando corresponda. Desconectar no borra recursos ajenos ni deshace efectos externos de forma automática. La reversión del software y la compensación de una operación externa son procedimientos distintos.

**Se considera completada cuando:** Fixtures real afectada/limpia se diferencian, offline se respeta y cobertura/fuente son visibles.

**Decisiones o condiciones pendientes:** Versión v2, distribución offline y política de actualización de base pendientes de piloto.

## 13. Fuentes oficiales y revalidación

Consultadas el 2026-11-03. Sustentan los mecanismos externos; el diseño de AlexIA es una propuesta propia. Revalidar scopes, rutas, versión, licencia y condiciones de distribución antes de implementar. Los enlaces de contratos específicos prevalecen sobre ejemplos generales; una página accesible no demuestra que una cuenta tenga habilitada la función.

- [Uso OSV v2](https://google.github.io/osv-scanner/usage/) — Subcomandos y formatos
- [Proyecto oficial](https://google.github.io/osv-scanner/) — Objetivo y fuentes de avisos
