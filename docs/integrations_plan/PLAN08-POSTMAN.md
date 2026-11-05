# PLAN08 — Integración de Postman con AlexIA

Fecha: 2026-11-03. Estado: **plan propuesto; integración pendiente**. Prioridad: P2, pruebas de APIs.
Audiencia: comunidad técnica, inicialmente instalaciones personales con proyectos autorizados.

## 1. Resultado y alcance

Reutilizar colecciones de APIs y ejecutar validaciones con resultados estructurados.

**Primer recorrido entregable:** Seleccionar una colección permitida y entorno de pruebas, ejecutar y mostrar aserciones fallidas sin exponer variables secretas.

**Alcance inicial:**

- Lectura opcional de colecciones mediante Postman API.
- Runner local seleccionado según formato: Newman para v2.1 compatible o Postman CLI si lo requiere la colección.
- Resultados por request/aserción y límites de red.

**Ampliaciones posteriores:** Sincronización de colecciones y preparación de cambios revisables.

**Fuera de esta primera entrega:** Publicar ambientes secretos, modificar workspaces o ejecutar colecciones de terceros sin revisar scripts.

## 2. Punto de partida real y prerrequisitos

No hay cliente Postman ni runner Newman/Postman en Cargo o ui/package.json. Las colecciones necesitan un ejecutor aparte del proveedor.

La interfaz actual administra configuración; no existe todavía el catálogo ejecutable de conectores ni la API de trabajos descrita aquí. El estado actual y las dependencias transversales están en [PLAN00 — Base común](PLAN00-BASE-COMUN.md). Las rutas, tipos, módulos y opciones nuevos que siguen son propuestas de implementación, no contratos existentes.

**Prerrequisitos específicos:**

- BC-04 y colección fixture sin efectos reales.
- Elegir formato y runner: la documentación actual diferencia colecciones v2.1 y v3.
- Cuenta/API key solo si se habilita sincronización cloud; archivos locales pueden probarse sin ella.

**Dependencias entre planes:** PLAN00 y PLAN03/04/06; no depende de GitHub.

## 3. Decisión técnica y flujo de conexión

Separar cliente de catálogo remoto y adaptador runner. Guardar una instantánea por hash de colección y entorno saneado antes de ejecutar. Newman es una ruta para v2.1; no afirmar compatibilidad con v3, Package Library o todos los flujos OAuth. Postman CLI se evalúa explícitamente según soporte y requisitos de cuenta. Scripts pre-request/test son código, y el host de destino debe validarse tras interpolar variables.

**Procedimiento de conexión o habilitación:**

1. Registrar colección local o referencia de workspace autorizada.
2. Si hay cloud, guardar API key en vault y comprobar acceso solo al recurso elegido.
3. Detectar schema de colección y seleccionar runner compatible; bloquear formatos no probados.
4. Resolver entorno de prueba y variables secretas sin devolver valores a UI.
5. Mostrar requests/hosts y efectos previstos; ejecutar en perfil con red limitada.
6. Parsear JSON/JUnit según runner y mostrar aserciones, status y duración.

**Autenticación, renovación y revocación:** La API de Postman usa API key en cabecera X-Api-Key. Las credenciales del API probado son otra conexión, inyectada solo al runner; no mezclar ambos tokens.

**Permisos externos y selección de recursos:** Recursos del workspace elegidos y base URLs permitidas. Una colección puede contener POST/DELETE y scripts; lectura de colección no autoriza ejecutarla.

Las credenciales residen en el almacén del sistema descrito en PLAN00; la configuración conserva solamente referencias. El navegador, el modelo y la respuesta de Alexa no reciben tokens. Una instalación comunitaria no comparte credenciales con otra. Si el producto exige un servicio OAuth confidencial, su secreto permanece en ese servicio; nunca se distribuye en el frontend, extensión o binario público.

## 4. Capacidades autorizables

Los identificadores de esta tabla son capacidades internas de AlexIA; no son nombres de scopes del proveedor.

| Capacidad propuesta | Operación | Guarda específica |
|---|---|---|
| `postman.collections.read` | Consultar colección | Workspace/ID permitido |
| `postman.collections.run` | Ejecutar snapshot | Hash, entorno y hosts revisados |
| `postman.results.read` | Ver resultados | Redacción de headers/cuerpos |

Las lecturas pueden preautorizarse para recursos concretos. Las escrituras o ejecuciones se sujetan a una política explícita por proyecto. Una política previa válida evita pedir confirmación repetida; si falta, preparar una vista previa concreta. La autorización queda ligada a identidad, recurso, parámetros y versión. Texto de un issue, archivo o registro no modifica permisos.

## 5. Superficie externa y contrato del adaptador

| Operación | API, comando o mecanismo | Tratamiento de la respuesta |
|---|---|---|
| Colección remota | GET https://api.getpostman.com/collections/{collection_id} | X-Api-Key, límites y hash |
| Runner v2.1 | newman run <snapshot> -e <entorno-temporal> -r json | Opciones de reporte fijadas en definición |
| Runner alternativo | Postman CLI según formato verificado | No asumir comandos o formato idéntico a Newman |

No invocar runner via npx con descarga implícita por trabajo. Fijar versión en harness. Redactar Authorization/Cookie y variables antes de conservar resultados. El request final puede cambiar por scripts; limitar egress a nivel de ejecución, no solo inspeccionar el JSON inicial.

**Petición interna ilustrativa:**

```json
{
  "integration_id": "postman",
  "project_id": "project-demo",
  "capability": "postman.collections.run",
  "input": {
    "collection_id": "collection-demo",
    "snapshot_hash": "demo",
    "environment_id": "staging-demo"
  },
  "idempotency_key": "demo-postman-001"
}
```

Se envía al propuesto `POST /api/jobs`, exclusivo de la API administrativa autenticada. Respuesta: HTTP 202 con `job_id`, `status` y `status_url`. `GET /api/jobs/{job_id}` consulta progreso; `POST /api/jobs/{job_id}/cancel` solicita cancelación. Validar esquema y pertenencia antes de consultar credenciales o iniciar procesos. No existe un endpoint que acepte una URL o comando arbitrario del modelo.

**Datos específicos normalizados:** collection/schema/runner_version, snapshot_hash, environment_id, request/assertion counts, failures y duración.

El resultado incluye `schema_version`, identidad del recurso, instante, versión de herramienta/API, alcance, evidencia saneada, advertencias y estado de cobertura. La respuesta puede terminar con hallazgos aunque el proceso haya funcionado correctamente. Diferenciar ausencia de resultados, permisos insuficientes, cobertura parcial y ejecución fallida.

## 6. Archivos y cambios previstos en AlexIA

Los archivos nuevos se crean durante la implementación. Evitar duplicar contratos del núcleo en el conector.

| Ruta relativa al repositorio | Responsabilidad |
|---|---|
| `src/integrations/postman.rs` | Catálogo y selección |
| `src/integrations/postman_runner.rs` | Compatibilidad y resultado |
| `tools/api-tests/package.json` | Runner fijado separado |
| `tests/integrations_postman.rs` | Contratos y datos secretos |

`src/lib.rs` registraría los módulos; `src/control.rs` delegaría rutas administrativas al núcleo de integraciones. `ui/src/main.js` consumiría estados y resultados; `ui/index.html`/`ui/src/components.js` presentarían la conexión. Conservar las guardas loopback actuales y actualizar [BACKEND.md](../../.agents/.docs/BACKEND.md), [UI.md](../../.agents/.docs/UI.md) y el documento operativo correspondiente al implementar.

## 7. Configuración propuesta

| Campo lógico | Tipo/protección | Uso |
|---|---|---|
| `api_key_ref` | Referencia secreta | Solo catálogo cloud |
| `collection_ids` | Lista | Colecciones elegidas |
| `runner_kind` | Enum | newman-v2.1 o postman-cli verificado |
| `allowed_origin_ids` | Lista | Destinos de pruebas |
| `environment_secret_refs` | Referencias | Credenciales del API probado |

Estos campos pertenecen a un registro validado de integración, no a un conjunto de secretos nuevos en `.env`. Escribirlos desde la API local con control de revisión. Exponer solamente `configured`, permisos y errores saneados. La presencia de configuración no equivale a conexión verificada.

## 8. Implementación por entregas verificables

### 08.1 — Importar y validar formato

**Trabajo:** Leer colección y crear snapshot.

**Dependencias:** BC-01 a BC-05 de PLAN00; completar los prerrequisitos de este documento.

**Aceptación:**

- [ ] v3 no va a Newman.
- [ ] Scripts y hosts se presentan al usuario.

**Verificación:** Fixtures v2.1, v3 y colección inválida.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### 08.2 — Ejecutar contra API fixture

**Trabajo:** Inyectar entorno y parsear resultado.

**Dependencias:** Tarea 08.1 completada y contrato de salida estable.

**Aceptación:**

- [ ] Un assert fallido queda identificado.
- [ ] Secreto no aparece en resultados.

**Verificación:** API local con 200, 401 y fallo de aserción.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### 08.3 — Conectar catálogo y recuperar

**Trabajo:** Añadir cloud opcional, cancelación y desconexión.

**Dependencias:** Tarea 08.2 completada y contrato de salida estable.

**Aceptación:**

- [ ] Revocar key bloquea catálogo.
- [ ] Un run no modifica colección remota.

**Verificación:** Lectura real autorizada más UI CLI.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### Checkpoint de salida

- [ ] Primera operación real completa desde UI hasta proveedor/herramienta y resultado visible.
- [ ] Denegación comprobada al cambiar identidad, proyecto o recurso autorizado.
- [ ] Cancelación, error y desconexión dejan estados coherentes y permiten recuperación.
- [ ] No se modificó el contrato de firma, certificado, timestamp e ID de Alexa.
- [ ] La documentación distingue simulación, prueba real y funciones pendientes.

## 9. Errores, límites y recuperación

Timeout por request y global; proponer 10 s/request y 3 min/colección. Reportar requests omitidos por bail. No repetir requests con efectos después de una interrupción. Una colección incompleta no se declara aprobada.

Límites iniciales de diseño: una ejecución por proyecto, cola local acotada, cuerpo administrativo de hasta 128 KiB y resultados paginados; los artefactos grandes se entregan por identificador. Los valores se medirán antes de liberar y no describen capacidad demostrada. HTTP 401 expira la sesión; 403 no se corrige ampliando permisos silenciosamente; 429 respeta `Retry-After` cuando exista. Un timeout después de una escritura produce estado `unknown_outcome` hasta reconciliar, sin repetición ciega.

## 10. Riesgos particulares y controles

| Riesgo | Control previsto | Evidencia requerida |
|---|---|---|
| Scripts con acceso/red no previstos | Runner aislado y egress cerrado | Host no autorizado inaccesible |
| Secretos en reporte | Redacción previa y entorno temporal | Centinela ausente |
| Formato incompatible | Detección y matriz runner/schema | v3 rechazado en Newman |

**Datos y retención:** No persistir cuerpos completos por defecto; conservar códigos y aserciones saneadas. El entorno temporal tiene ACL y se borra al finalizar.

## 11. Pruebas del adaptador y del flujo completo

**Fixtures y pruebas de contrato:**

- Formato incompatible, host variable y scripts.
- 401 cloud frente a 401 del API probado.
- Reporte parcial, secret en headers y timeout con efecto desconocido.

**Prueba real de aceptación:** Ejecutar colección fixture contra API local y, por separado, leer una colección de workspace autorizado. No modificar recursos reales.

**Prueba de interfaz con la skill Playwright CLI:**

1. Elegir colección y ver runner detectado.
2. Revisar entorno sin ver secretos.
3. Ejecutar y abrir aserción fallida.
4. Simular falta de permisos y comprobar que no se cambia de runner automáticamente.

Usar sesión de navegador desechable; obtener referencias con `snapshot` y ejecutar comprobaciones con `run-code --filename=...` cuando se necesiten aserciones. Para el estado simulado, interceptar únicamente la API propuesta e identificar el resultado como prueba de UI. La prueba real debe comprobar el recurso externo o artefacto local. No automatizar pantallas de login ni guardar cookies del usuario para sustituir APIs oficiales.

Comandos de validación cuando se implemente: `cargo test --locked --test integrations_postman` para el archivo propuesto, `cargo check --locked --all-targets`, las pruebas de UI afectadas y `npm --prefix ui run build`. Estos comandos son objetivos; no se afirma que esos tests existan hoy. Añadir pruebas de esquema, autorización y recuperación antes de habilitar capacidades de escritura.

## 12. Despliegue, reversión y criterio de finalización

Habilitar primero para una instalación piloto mediante flag de integración y solamente las capacidades verificadas. Registrar versión y permisos. Después ampliar a un segundo proyecto y comprobar separación de conexiones; no equivale a un despliegue SaaS multiusuario.

**Reversión específica:** Deshabilitar sincronización/runner, revocar key y borrar temporales; no borrar colecciones o ambientes remotos.

Deshabilitar el conector impide nuevos trabajos; cancelar o drenar los existentes según su efecto. Revocar credenciales cuando corresponda. Desconectar no borra recursos ajenos ni deshace efectos externos de forma automática. La reversión del software y la compensación de una operación externa son procedimientos distintos.

**Se considera completada cuando:** Colección compatible real produce resultados correctos y secretos redactados; formatos no soportados se rechazan claramente.

**Decisiones o condiciones pendientes:** Seleccionar runner y formato piloto, requisitos de cuenta y licencia antes de empaquetar.

## 13. Fuentes oficiales y revalidación

Consultadas el 2026-11-03. Sustentan los mecanismos externos; el diseño de AlexIA es una propuesta propia. Revalidar scopes, rutas, versión, licencia y condiciones de distribución antes de implementar. Los enlaces de contratos específicos prevalecen sobre ejemplos generales; una página accesible no demuestra que una cuenta tenga habilitada la función.

- [API de Postman](https://learning.postman.com/docs/reference/postman-api/intro-api/) — Catálogo y autenticación
- [Newman y limitaciones](https://learning.postman.com/docs/reference/newman-cli/command-line-integration-with-newman/) — Compatibilidad v2.1 y alternativa CLI
