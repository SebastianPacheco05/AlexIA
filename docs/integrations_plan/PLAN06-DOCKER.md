# PLAN06 — Integración de Docker con AlexIA

Fecha: 2026-11-03. Estado: **plan propuesto; integración pendiente**. Prioridad: P2, ejecución reproducible.
Audiencia: comunidad técnica, inicialmente instalaciones personales con proyectos autorizados.

## 1. Resultado y alcance

Levantar entornos de prueba definidos y gestionar únicamente contenedores creados para tareas AlexIA.

**Primer recorrido entregable:** Elegir una receta aprobada, crear un entorno efímero, ejecutar comprobaciones y retirar los recursos propios.

**Alcance inicial:**

- Detectar Engine/Context y estado del daemon.
- Recetas revisadas con imagen por digest, recursos, red y montajes explícitos.
- Logs acotados, inspección y limpieza por ownership labels.

**Ampliaciones posteriores:** Compose revisado, caché de imágenes y perfiles por lenguaje.

**Fuera de esta primera entrega:** Contenedores privilegiados, socket Docker dentro del contenedor, host networking y volúmenes arbitrarios.

## 2. Punto de partida real y prerrequisitos

Hay Dockerfile para alojar AlexIA, pero esto no es un runner de proyectos. La evidencia histórica señala motor Linux inactivo; volver a comprobarlo, sin asumir disponibilidad actual.

La interfaz actual administra configuración; no existe todavía el catálogo ejecutable de conectores ni la API de trabajos descrita aquí. El estado actual y las dependencias transversales están en [PLAN00 — Base común](PLAN00-BASE-COMUN.md). Las rutas, tipos, módulos y opciones nuevos que siguen son propuestas de implementación, no contratos existentes.

**Prerrequisitos específicos:**

- Daemon instalado/activo y permiso del propietario; no activarlo ni cambiarlo sin instrucción.
- Recetas de prueba y usuario no root cuando la imagen lo soporte.
- Contrato de ejecución y almacenamiento temporal de PLAN00.

**Dependencias entre planes:** PLAN00; puede soportar PLAN07/12/13/14 y proyectos no confiables con límites documentados.

## 3. Decisión técnica y flujo de conexión

En piloto usar Docker CLI con argv validado; Engine API es una alternativa posterior, evitando exponer su socket a la UI o al modelo. El acceso al daemon es de alto privilegio; el contenedor no constituye aislamiento completo frente a código hostil. Para comunidad con ejecución no confiable evaluar VM o entorno dedicado. Los recursos se marcan con project_id y job_id internos y un namespace AlexIA.

**Procedimiento de conexión o habilitación:**

1. Comprobar docker version e info sin cambiar contexto.
2. Mostrar host/context y exigir selección si es remoto.
3. Seleccionar receta revisada; resolver digest y verificar política de descarga.
4. Montar copia del proyecto o bind read-only; volumen de salida propio.
5. Aplicar cpu/memoria/pids, cap-drop, no-new-privileges y red limitada.
6. Ejecutar, recopilar resultados y eliminar solo recursos con ownership confirmado.

**Autenticación, renovación y revocación:** Usa contexto Docker del propietario. No leer ni exportar ~/.docker/config.json. Para registro privado, reutilizar credential helper autorizado sin entregar credenciales al modelo.

**Permisos externos y selección de recursos:** Permiso por contexto, receta y montaje. Construir una imagen ejecuta instrucciones del Dockerfile y necesita política distinta a leer metadata. No compartir daemon entre usuarios de un SaaS sin diseño adicional.

Las credenciales residen en el almacén del sistema descrito en PLAN00; la configuración conserva solamente referencias. El navegador, el modelo y la respuesta de Alexa no reciben tokens. Una instalación comunitaria no comparte credenciales con otra. Si el producto exige un servicio OAuth confidencial, su secreto permanece en ese servicio; nunca se distribuye en el frontend, extensión o binario público.

## 4. Capacidades autorizables

Los identificadores de esta tabla son capacidades internas de AlexIA; no son nombres de scopes del proveedor.

| Capacidad propuesta | Operación | Guarda específica |
|---|---|---|
| `docker.engine.inspect` | Ver disponibilidad/contexto | Sin cambios de host |
| `docker.recipes.run` | Ejecutar receta aprobada | Digest/montajes/recursos fijos |
| `docker.resources.cleanup` | Eliminar recursos propios | Labels y IDs verificados |

Las lecturas pueden preautorizarse para recursos concretos. Las escrituras o ejecuciones se sujetan a una política explícita por proyecto. Una política previa válida evita pedir confirmación repetida; si falta, preparar una vista previa concreta. La autorización queda ligada a identidad, recurso, parámetros y versión. Texto de un issue, archivo o registro no modifica permisos.

## 5. Superficie externa y contrato del adaptador

| Operación | API, comando o mecanismo | Tratamiento de la respuesta |
|---|---|---|
| Disponibilidad | docker version --format json | Separar cliente y servidor |
| Crear/ejecutar | docker create / start / wait con argv | Capturar ID y estado |
| Logs | docker logs <id> | Drenar stdout/stderr con límite |
| Limpieza | docker rm <id> y recursos explícitos | Nada de system prune global |

Generar argv desde receta tipada y verificar el contenedor inspeccionado contra la receta. No admitir Compose arbitrario en el MVP. En Docker Desktop/WSL validar rutas compartidas y filesystem real. Un pull necesita salida de red al registro; red de ejecución y red de descarga son políticas diferentes.

**Petición interna ilustrativa:**

```json
{
  "integration_id": "docker",
  "project_id": "project-demo",
  "capability": "docker.recipes.run",
  "input": {
    "recipe_id": "rust-check",
    "image_digest": "sha256:demo",
    "network_profile": "none"
  },
  "idempotency_key": "demo-docker-001"
}
```

Se envía al propuesto `POST /api/jobs`, exclusivo de la API administrativa autenticada. Respuesta: HTTP 202 con `job_id`, `status` y `status_url`. `GET /api/jobs/{job_id}` consulta progreso; `POST /api/jobs/{job_id}/cancel` solicita cancelación. Validar esquema y pertenencia antes de consultar credenciales o iniciar procesos. No existe un endpoint que acepte una URL o comando arbitrario del modelo.

**Datos específicos normalizados:** context_id, receta/hash, image_digest, container_id interno, recursos, mounts, exit_code y cleanup_status.

El resultado incluye `schema_version`, identidad del recurso, instante, versión de herramienta/API, alcance, evidencia saneada, advertencias y estado de cobertura. La respuesta puede terminar con hallazgos aunque el proceso haya funcionado correctamente. Diferenciar ausencia de resultados, permisos insuficientes, cobertura parcial y ejecución fallida.

## 6. Archivos y cambios previstos en AlexIA

Los archivos nuevos se crean durante la implementación. Evitar duplicar contratos del núcleo en el conector.

| Ruta relativa al repositorio | Responsabilidad |
|---|---|
| `src/integrations/docker.rs` | Cliente CLI y disponibilidad |
| `src/execution/container.rs` | Recetas y ciclo de vida |
| `tests/integrations_docker.rs` | Ownership y errores |
| `tests/fixtures/docker/` | Imagen y recetas de prueba |

`src/lib.rs` registraría los módulos; `src/control.rs` delegaría rutas administrativas al núcleo de integraciones. `ui/src/main.js` consumiría estados y resultados; `ui/index.html`/`ui/src/components.js` presentarían la conexión. Conservar las guardas loopback actuales y actualizar [BACKEND.md](../../.agents/.docs/BACKEND.md), [UI.md](../../.agents/.docs/UI.md) y el documento operativo correspondiente al implementar.

## 7. Configuración propuesta

| Campo lógico | Tipo/protección | Uso |
|---|---|---|
| `docker_path` | Ruta | CLI autorizado |
| `context_id` | ID validado | Host elegido |
| `allowed_recipes` | Lista/hash | Recetas revisadas |
| `resource_limits` | Objeto | CPU, RAM y pids |
| `allowed_mount_roots` | Rutas | Origen y salida propios |

Estos campos pertenecen a un registro validado de integración, no a un conjunto de secretos nuevos en `.env`. Escribirlos desde la API local con control de revisión. Exponer solamente `configured`, permisos y errores saneados. La presencia de configuración no equivale a conexión verificada.

## 8. Implementación por entregas verificables

### 06.1 — Detectar y mostrar receta

**Trabajo:** Lectura de daemon y vista previa.

**Dependencias:** BC-01 a BC-05 de PLAN00; completar los prerrequisitos de este documento.

**Aceptación:**

- [ ] Daemon inactivo produce diagnóstico.
- [ ] Contexto remoto se identifica.

**Verificación:** Mock y host real sin crear recursos.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### 06.2 — Ejecutar entorno efímero

**Trabajo:** Crear contenedor de fixture y guardar ownership.

**Dependencias:** Tarea 06.1 completada y contrato de salida estable.

**Aceptación:**

- [ ] Sin socket ni privilegios.
- [ ] Exit code visible y montajes correctos.

**Verificación:** Inspección real de fixture.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### 06.3 — Cancelar y limpiar

**Trabajo:** Gestionar stop/kill/rm y recuperación tras reinicio.

**Dependencias:** Tarea 06.2 completada y contrato de salida estable.

**Aceptación:**

- [ ] No se elimina contenedor ajeno.
- [ ] Fallo de limpieza queda visible.

**Verificación:** Fixture con recurso ajeno y job interrumpido.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### Checkpoint de salida

- [ ] Primera operación real completa desde UI hasta proveedor/herramienta y resultado visible.
- [ ] Denegación comprobada al cambiar identidad, proyecto o recurso autorizado.
- [ ] Cancelación, error y desconexión dejan estados coherentes y permiten recuperación.
- [ ] No se modificó el contrato de firma, certificado, timestamp e ID de Alexa.
- [ ] La documentación distingue simulación, prueba real y funciones pendientes.

## 9. Errores, límites y recuperación

Una receta simultánea inicialmente; deadline propuesto 5 min para tareas y 2 min para pull, cancelables. Imagen no disponible no dispara build automático. Al reiniciar, reconciliar IDs etiquetados antes de recolectar basura.

Límites iniciales de diseño: una ejecución por proyecto, cola local acotada, cuerpo administrativo de hasta 128 KiB y resultados paginados; los artefactos grandes se entregan por identificador. Los valores se medirán antes de liberar y no describen capacidad demostrada. HTTP 401 expira la sesión; 403 no se corrige ampliando permisos silenciosamente; 429 respeta `Retry-After` cuando exista. Un timeout después de una escritura produce estado `unknown_outcome` hasta reconciliar, sin repetición ciega.

## 10. Riesgos particulares y controles

| Riesgo | Control previsto | Evidencia requerida |
|---|---|---|
| Privilegio del daemon | Contexto explícito y recetas cerradas | Inspección de argv/contendor |
| Montaje fuera de proyecto | Rutas canónicas y copia read-only | Intento de montar perfil rechazado |
| Borrado de recursos ajenos | ID más labels de propiedad | Contenedor ajeno sobrevive limpieza |

**Datos y retención:** Logs y volúmenes pueden contener datos privados; TTL por trabajo. No montar HOME, claves SSH ni credenciales del proveedor.

## 11. Pruebas del adaptador y del flujo completo

**Fixtures y pruebas de contrato:**

- Motor inactivo, versión incompatible y contexto remoto.
- Receta con socket/privileged rechazada.
- Cancelación y limpieza repetida idempotente de recursos propios.

**Prueba real de aceptación:** En daemon autorizado, ejecutar receta mínima con red deshabilitada; inspeccionar límites y verificar limpieza sin tocar otro contenedor fixture.

**Prueba de interfaz con la skill Playwright CLI:**

1. Ver disponibilidad real.
2. Revisar receta y ejecutar.
3. Ver logs y cancelar fixture lenta.
4. Comprobar estado cleanup_failed mediante fixture separada.

Usar sesión de navegador desechable; obtener referencias con `snapshot` y ejecutar comprobaciones con `run-code --filename=...` cuando se necesiten aserciones. Para el estado simulado, interceptar únicamente la API propuesta e identificar el resultado como prueba de UI. La prueba real debe comprobar el recurso externo o artefacto local. No automatizar pantallas de login ni guardar cookies del usuario para sustituir APIs oficiales.

Comandos de validación cuando se implemente: `cargo test --locked --test integrations_docker` para el archivo propuesto, `cargo check --locked --all-targets`, las pruebas de UI afectadas y `npm --prefix ui run build`. Estos comandos son objetivos; no se afirma que esos tests existan hoy. Añadir pruebas de esquema, autorización y recuperación antes de habilitar capacidades de escritura.

## 12. Despliegue, reversión y criterio de finalización

Habilitar primero para una instalación piloto mediante flag de integración y solamente las capacidades verificadas. Registrar versión y permisos. Después ampliar a un segundo proyecto y comprobar separación de conexiones; no equivale a un despliegue SaaS multiusuario.

**Reversión específica:** Deshabilitar nuevas recetas, detener recursos propios y limpiar por IDs; nunca docker system prune.

Deshabilitar el conector impide nuevos trabajos; cancelar o drenar los existentes según su efecto. Revocar credenciales cuando corresponda. Desconectar no borra recursos ajenos ni deshace efectos externos de forma automática. La reversión del software y la compensación de una operación externa son procedimientos distintos.

**Se considera completada cuando:** Una receta real cumple límites, cancelación y limpieza por propiedad; daemon ausente se maneja sin falsa conexión.

**Decisiones o condiciones pendientes:** Elegir aislamiento suficiente para código no confiable y soporte de Docker Desktop frente a Linux; no habilitar nube compartida por extensión del MVP.

## 13. Fuentes oficiales y revalidación

Consultadas el 2026-11-03. Sustentan los mecanismos externos; el diseño de AlexIA es una propuesta propia. Revalidar scopes, rutas, versión, licencia y condiciones de distribución antes de implementar. Los enlaces de contratos específicos prevalecen sobre ejemplos generales; una página accesible no demuestra que una cuenta tenga habilitada la función.

- [Engine API](https://docs.docker.com/reference/api/engine/) — Contrato y compatibilidad
- [Seguridad del Engine](https://docs.docker.com/engine/security/) — Límites del aislamiento y daemon
