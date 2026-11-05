# PLAN17 — Integración de Notion con AlexIA

Fecha: 2026-11-03. Estado: **plan propuesto; integración pendiente**. Prioridad: P2, contexto documental.
Audiencia: comunidad técnica, inicialmente instalaciones personales con proyectos autorizados.

## 1. Resultado y alcance

Consultar documentación compartida explícitamente y preparar páginas de cambios con trazabilidad.

**Primer recorrido entregable:** Seleccionar una página técnica autorizada y usar sus bloques como contexto citado para una tarea.

**Alcance inicial:**

- Conexión interna BYO para piloto; OAuth público para distribución posterior.
- Lectura de páginas/bloques seleccionados y paginación recursiva acotada.
- Borradores de documentación locales.

**Ampliaciones posteriores:** Crear/actualizar páginas con vista previa y data sources según versión vigente.

**Fuera de esta primera entrega:** Indexar workspace completo, descargar archivos firmados indiscriminadamente o sobrescribir páginas sin precondiciones.

## 2. Punto de partida real y prerrequisitos

No existe Notion ni almacén documental. La configuración actual del metaprompt no es una base de conocimiento.

La interfaz actual administra configuración; no existe todavía el catálogo ejecutable de conectores ni la API de trabajos descrita aquí. El estado actual y las dependencias transversales están en [PLAN00 — Base común](PLAN00-BASE-COMUN.md). Las rutas, tipos, módulos y opciones nuevos que siguen son propuestas de implementación, no contratos existentes.

**Prerrequisitos específicos:**

- Conexión Notion y página fixture compartida.
- Elegir Notion-Version y esquema por versión.
- Política de fragmentos, enlaces y profundidad máxima.

**Dependencias entre planes:** PLAN00; no requiere base vectorial ni Supabase para el piloto.

## 3. Decisión técnica y flujo de conexión

Cliente HTTP que lee solo IDs compartidos y límites definidos. Fijar Notion-Version. Recuperar página y children de bloques con paginación/profundidad; tratar unsupported como cobertura parcial. No seguir URLs externas de bloques automáticamente. Distinguir database y data_source de APIs actuales al añadir tablas; no usar rutas antiguas sin verificar versión.

**Procedimiento de conexión o habilitación:**

1. Crear conexión interna de prueba y conceder Read content.
2. Compartir explícitamente página fixture con esa conexión.
3. Guardar token en vault y probar lectura de página.
4. Seleccionar raíces permitidas y recuperar children con límites.
5. Mostrar fragmentos y enlaces de origen antes de enviarlos al modelo.
6. Añadir OAuth para múltiples usuarios tras resolver distribución y callback.

**Autenticación, renovación y revocación:** Bearer de conexión interna para piloto local BYO; OAuth público con state y canje en componente autorizado para comunidad. No asumir refresh o formato de token: implementar lo documentado para la versión/flujo elegido.

**Permisos externos y selección de recursos:** Read content inicialmente. Insert/Update content solo para escribir páginas seleccionadas en fases posteriores. Compartir una página no concede acceso universal al workspace.

Las credenciales residen en el almacén del sistema descrito en PLAN00; la configuración conserva solamente referencias. El navegador, el modelo y la respuesta de Alexa no reciben tokens. Una instalación comunitaria no comparte credenciales con otra. Si el producto exige un servicio OAuth confidencial, su secreto permanece en ese servicio; nunca se distribuye en el frontend, extensión o binario público.

## 4. Capacidades autorizables

Los identificadores de esta tabla son capacidades internas de AlexIA; no son nombres de scopes del proveedor.

| Capacidad propuesta | Operación | Guarda específica |
|---|---|---|
| `notion.pages.read` | Leer página/bloques | Raíces compartidas |
| `notion.documents.preview` | Preparar documentación | Sin escritura externa |
| `notion.pages.update` | Actualizar posteriormente | Versión/contenido/destino autorizados |

Las lecturas pueden preautorizarse para recursos concretos. Las escrituras o ejecuciones se sujetan a una política explícita por proyecto. Una política previa válida evita pedir confirmación repetida; si falta, preparar una vista previa concreta. La autorización queda ligada a identidad, recurso, parámetros y versión. Texto de un issue, archivo o registro no modifica permisos.

## 5. Superficie externa y contrato del adaptador

| Operación | API, comando o mecanismo | Tratamiento de la respuesta |
|---|---|---|
| Página | GET /v1/pages/{page_id} | Metadatos y permisos |
| Bloques hijos | GET /v1/blocks/{block_id}/children | has_more/next_cursor y profundidad |
| Bloque | GET /v1/blocks/{block_id} | Tipo soportado explícito |
| Escritura posterior | POST /v1/pages o PATCH blocks según operación | No sobrescribir bloques ajenos |

Extraer rich_text a texto seguro conservando provenance por block_id. Los enlaces y URLs temporales de archivos no son texto confiable para egress. Si solo hay acceso parcial, informar sin inventar contenido de subpáginas.

**Petición interna ilustrativa:**

```json
{
  "integration_id": "notion",
  "project_id": "project-demo",
  "capability": "notion.pages.read",
  "input": {
    "root_page_id": "page-demo",
    "max_depth": 3
  },
  "idempotency_key": "demo-notion-001"
}
```

Se envía al propuesto `POST /api/jobs`, exclusivo de la API administrativa autenticada. Respuesta: HTTP 202 con `job_id`, `status` y `status_url`. `GET /api/jobs/{job_id}` consulta progreso; `POST /api/jobs/{job_id}/cancel` solicita cancelación. Validar esquema y pertenencia antes de consultar credenciales o iniciar procesos. No existe un endpoint que acepte una URL o comando arbitrario del modelo.

**Datos específicos normalizados:** workspace/page/block IDs, last_edited_time, versión API, fragmentos, tipos omitidos y enlaces.

El resultado incluye `schema_version`, identidad del recurso, instante, versión de herramienta/API, alcance, evidencia saneada, advertencias y estado de cobertura. La respuesta puede terminar con hallazgos aunque el proceso haya funcionado correctamente. Diferenciar ausencia de resultados, permisos insuficientes, cobertura parcial y ejecución fallida.

## 6. Archivos y cambios previstos en AlexIA

Los archivos nuevos se crean durante la implementación. Evitar duplicar contratos del núcleo en el conector.

| Ruta relativa al repositorio | Responsabilidad |
|---|---|
| `src/integrations/notion.rs` | HTTP y límites de bloques |
| `src/integrations/notion_blocks.rs` | Conversión segura |
| `tests/integrations_notion.rs` | Permisos/paginación |
| `ui/src/integrations/notion.js` | Raíces y fragmentos |

`src/lib.rs` registraría los módulos; `src/control.rs` delegaría rutas administrativas al núcleo de integraciones. `ui/src/main.js` consumiría estados y resultados; `ui/index.html`/`ui/src/components.js` presentarían la conexión. Conservar las guardas loopback actuales y actualizar [BACKEND.md](../../.agents/.docs/BACKEND.md), [UI.md](../../.agents/.docs/UI.md) y el documento operativo correspondiente al implementar.

## 7. Configuración propuesta

| Campo lógico | Tipo/protección | Uso |
|---|---|---|
| `credential_ref` | Secreto | Conexión/OAuth |
| `root_page_ids` | Lista | Contenido elegido |
| `notion_version` | Versión | Contrato fijado |
| `max_depth` | Entero | Propuesto 3 |
| `max_blocks` | Entero | Propuesto 500 por tarea |

Estos campos pertenecen a un registro validado de integración, no a un conjunto de secretos nuevos en `.env`. Escribirlos desde la API local con control de revisión. Exponer solamente `configured`, permisos y errores saneados. La presencia de configuración no equivale a conexión verificada.

## 8. Implementación por entregas verificables

### 17.1 — Conectar página

**Trabajo:** Lectura y permisos.

**Dependencias:** BC-01 a BC-05 de PLAN00; completar los prerrequisitos de este documento.

**Aceptación:**

- [ ] Página no compartida no aparece accesible.
- [ ] Token oculto.

**Verificación:** Dos páginas fixture con permisos distintos.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### 17.2 — Extraer contexto

**Trabajo:** Recursión/paginación/rich_text.

**Dependencias:** Tarea 17.1 completada y contrato de salida estable.

**Aceptación:**

- [ ] Unsupported visible.
- [ ] Límites producen parcial.

**Verificación:** Página con varios tipos de bloque.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### 17.3 — Borrador y desconexión

**Trabajo:** Generar documento local citado.

**Dependencias:** Tarea 17.2 completada y contrato de salida estable.

**Aceptación:**

- [ ] No escribe Notion al generar.
- [ ] Revocar elimina acceso/caches según política.

**Verificación:** CLI y revocación real fixture.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### Checkpoint de salida

- [ ] Primera operación real completa desde UI hasta proveedor/herramienta y resultado visible.
- [ ] Denegación comprobada al cambiar identidad, proyecto o recurso autorizado.
- [ ] Cancelación, error y desconexión dejan estados coherentes y permiten recuperación.
- [ ] No se modificó el contrato de firma, certificado, timestamp e ID de Alexa.
- [ ] La documentación distingue simulación, prueba real y funciones pendientes.

## 9. Errores, límites y recuperación

Respetar Retry-After y paginación. Proponer 60 s para extracción limitada. Cache de contenido opcional y desactivada por defecto; invalidar por edición o revocación.

Límites iniciales de diseño: una ejecución por proyecto, cola local acotada, cuerpo administrativo de hasta 128 KiB y resultados paginados; los artefactos grandes se entregan por identificador. Los valores se medirán antes de liberar y no describen capacidad demostrada. HTTP 401 expira la sesión; 403 no se corrige ampliando permisos silenciosamente; 429 respeta `Retry-After` cuando exista. Un timeout después de una escritura produce estado `unknown_outcome` hasta reconciliar, sin repetición ciega.

## 10. Riesgos particulares y controles

| Riesgo | Control previsto | Evidencia requerida |
|---|---|---|
| Página contiene instrucciones hostiles | Tratar como contenido/citar | No cambia permisos |
| Recursión grande | Profundidad y bloques máximos | Resultado parcial |
| Edición concurrente | last_edited_time y relectura | Borrador obsoleto detectado |

**Datos y retención:** Documentación puede contener incidentes y secretos; selección/redacción. No conservar páginas completas ni adjuntos por defecto.

## 11. Pruebas del adaptador y del flujo completo

**Fixtures y pruebas de contrato:**

- Paginación, bloques unsupported y falta de permiso.
- Ciclos/referencias y límite de profundidad.
- Links externos, contenido activo y edición concurrente.

**Prueba real de aceptación:** Leer página fixture con varios bloques y otra no compartida; comparar cobertura y revocar.

**Prueba de interfaz con la skill Playwright CLI:**

1. Conectar y seleccionar raíz.
2. Revisar fragmentos/citas.
3. Ver indicador de contenido parcial.
4. Crear borrador local sin modificar página.

Usar sesión de navegador desechable; obtener referencias con `snapshot` y ejecutar comprobaciones con `run-code --filename=...` cuando se necesiten aserciones. Para el estado simulado, interceptar únicamente la API propuesta e identificar el resultado como prueba de UI. La prueba real debe comprobar el recurso externo o artefacto local. No automatizar pantallas de login ni guardar cookies del usuario para sustituir APIs oficiales.

Comandos de validación cuando se implemente: `cargo test --locked --test integrations_notion` para el archivo propuesto, `cargo check --locked --all-targets`, las pruebas de UI afectadas y `npm --prefix ui run build`. Estos comandos son objetivos; no se afirma que esos tests existan hoy. Añadir pruebas de esquema, autorización y recuperación antes de habilitar capacidades de escritura.

## 12. Despliegue, reversión y criterio de finalización

Habilitar primero para una instalación piloto mediante flag de integración y solamente las capacidades verificadas. Registrar versión y permisos. Después ampliar a un segundo proyecto y comprobar separación de conexiones; no equivale a un despliegue SaaS multiusuario.

**Reversión específica:** Revocar token/conexión, quitar raíces y borrar cache autorizada; no archivar páginas remotas automáticamente.

Deshabilitar el conector impide nuevos trabajos; cancelar o drenar los existentes según su efecto. Revocar credenciales cuando corresponda. Desconectar no borra recursos ajenos ni deshace efectos externos de forma automática. La reversión del software y la compensación de una operación externa son procedimientos distintos.

**Se considera completada cuando:** Lectura real con citas, permisos y cobertura; borrador no modifica contenido externo.

**Decisiones o condiciones pendientes:** Versión API y OAuth público; escritura/data sources se diseñan al elegir función concreta.

## 13. Fuentes oficiales y revalidación

Consultadas el 2026-11-03. Sustentan los mecanismos externos; el diseño de AlexIA es una propuesta propia. Revalidar scopes, rutas, versión, licencia y condiciones de distribución antes de implementar. Los enlaces de contratos específicos prevalecen sobre ejemplos generales; una página accesible no demuestra que una cuenta tenga habilitada la función.

- [Autorización Notion](https://developers.notion.com/guides/authorization/overview) — Conexiones y OAuth
- [Bloques](https://developers.notion.com/reference/retrieve-a-block) — Tipos y versionado
- [Inicio API](https://developers.notion.com/) — Modelo de páginas y data sources
