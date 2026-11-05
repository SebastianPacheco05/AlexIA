# PLAN02 — Integración de Visual Studio Code con AlexIA

Fecha: 2026-11-03. Estado: **plan propuesto; integración pendiente**. Prioridad: P2, después del primer flujo de trabajos.
Audiencia: comunidad técnica, inicialmente instalaciones personales con proyectos autorizados.

## 1. Resultado y alcance

Usar AlexIA desde el editor con contexto explícitamente seleccionado y resultados navegables a archivos y líneas.

**Primer recorrido entregable:** El desarrollador selecciona código, ejecuta Analizar con AlexIA y obtiene un resultado que abre el archivo correcto sin exportar todo el workspace.

**Alcance inicial:**

- Extensión TypeScript con comandos y vista de trabajos.
- Selección, URI, lenguaje, versión del documento y diagnósticos elegidos.
- Vista de diferencias para propuestas; escritura deshabilitada inicialmente.

**Ampliaciones posteriores:** Aplicación de WorkspaceEdit tras validación, proveedor de tareas y herramientas para agentes del editor.

**Fuera de esta primera entrega:** Dependencia obligatoria de Copilot, acceso irrestricto al disco, aplicación automática de parches y compatibilidad afirmada con otros editores.

## 2. Punto de partida real y prerrequisitos

AlexIA tiene una UI Vite local, sin extensión ni protocolo para clientes del editor. Las guardas de /api/settings exigen origen y marcador de la UI; no deben relajarse para aceptar extensiones.

La interfaz actual administra configuración; no existe todavía el catálogo ejecutable de conectores ni la API de trabajos descrita aquí. El estado actual y las dependencias transversales están en [PLAN00 — Base común](PLAN00-BASE-COMUN.md). Las rutas, tipos, módulos y opciones nuevos que siguen son propuestas de implementación, no contratos existentes.

**Prerrequisitos específicos:**

- Núcleo de trabajos y autenticación de clientes locales BC-03.
- Node/npm y herramientas de desarrollo de extensiones fijadas al implementar.
- Workspace de prueba con archivo modificado, archivo fuera de raíz y workspace no confiable.

**Dependencias entre planes:** PLAN00; reutiliza PLAN03/04 para ejecutar tareas. No depende de Github ni del API de modelos de VS Code.

## 3. Decisión técnica y flujo de conexión

Crear extensions/vscode como paquete separado con una extensión mínima. La extensión se empareja con el backend mediante código de un solo uso mostrado en la UI local; obtiene credencial acotada guardada en SecretStorage. Usar un endpoint de clientes locales autenticados separado de las rutas que dependen de Origin de navegador. Webviews, si se necesitan, usan CSP estricta, recursos locales acotados y mensajes validados. Comenzar con comandos y TreeView para reducir superficie.

**Procedimiento de conexión o habilitación:**

1. Detectar backend local y mostrar versión compatible sin escanear puertos arbitrarios.
2. Pedir emparejamiento en la UI de AlexIA y consumir código de un solo uso con TTL.
3. Guardar credencial en ExtensionContext.secrets; registrar carpeta y permisos concretos.
4. Exigir Workspace Trust y selección de contenido antes de enviarlo al proveedor.
5. Enviar documento/selección con version y hash; seguir el job_id en la vista.
6. Abrir diagnósticos y diff; al desinstalar o desvincular, revocar el cliente.

**Autenticación, renovación y revocación:** Token local acotado a client_id y project_id, revocable y con expiración. Nunca usar la cabecera X-Alexia-UI como contraseña. Un proceso local malicioso sigue siendo parte del modelo de amenazas; pairing y loopback reducen exposición pero no aíslan el sistema operativo.

**Permisos externos y selección de recursos:** Lectura de selección/archivo elegido y de diagnósticos del proyecto confiable. Lectura de archivos, ejecución y edición son capacidades independientes. No enviar terminales, variables de entorno ni secretos del editor.

Las credenciales residen en el almacén del sistema descrito en PLAN00; la configuración conserva solamente referencias. El navegador, el modelo y la respuesta de Alexa no reciben tokens. Una instalación comunitaria no comparte credenciales con otra. Si el producto exige un servicio OAuth confidencial, su secreto permanece en ese servicio; nunca se distribuye en el frontend, extensión o binario público.

## 4. Capacidades autorizables

Los identificadores de esta tabla son capacidades internas de AlexIA; no son nombres de scopes del proveedor.

| Capacidad propuesta | Operación | Guarda específica |
|---|---|---|
| `vscode.selection.read` | Entregar selección explícita | Workspace Trust y URI permitida |
| `vscode.diagnostics.read` | Consultar diagnósticos seleccionados | Solo proyecto vinculado |
| `vscode.edits.preview` | Abrir diff propuesto | Versión/hash actuales, sin escritura inicial |

Las lecturas pueden preautorizarse para recursos concretos. Las escrituras o ejecuciones se sujetan a una política explícita por proyecto. Una política previa válida evita pedir confirmación repetida; si falta, preparar una vista previa concreta. La autorización queda ligada a identidad, recurso, parámetros y versión. Texto de un issue, archivo o registro no modifica permisos.

## 5. Superficie externa y contrato del adaptador

| Operación | API, comando o mecanismo | Tratamiento de la respuesta |
|---|---|---|
| Comando | vscode.commands.registerCommand | Entrada validada en Extension Host |
| Selección y documento | window.activeTextEditor y TextDocument | URI/version/rango/hash |
| Diagnósticos | languages.getDiagnostics | Filtrar por raíz permitida |
| Vista de diff | Comando vscode.diff y documento virtual | Contenido original separado de propuesta |
| Edición posterior | WorkspaceEdit y workspace.applyEdit | Revalidar versión y permisos antes de aplicar |

Protocolo de extensión versionado por major. Resolver URI de documento, no concatenar rutas. Rechazar esquemas remotos o virtuales no soportados. Para Remote SSH/WSL decidir dónde corre backend y Extension Host; un localhost remoto no es el localhost del usuario. Un webview no accede al secreto de pairing.

**Petición interna ilustrativa:**

```json
{
  "integration_id": "vscode",
  "project_id": "project-demo",
  "capability": "vscode.selection.read",
  "input": {
    "document_uri": "file:///workspace/demo/src/main.rs",
    "document_version": 3,
    "range": {
      "start_line": 10,
      "end_line": 20
    }
  },
  "idempotency_key": "demo-vscode-001"
}
```

Se envía al propuesto `POST /api/jobs`, exclusivo de la API administrativa autenticada. Respuesta: HTTP 202 con `job_id`, `status` y `status_url`. `GET /api/jobs/{job_id}` consulta progreso; `POST /api/jobs/{job_id}/cancel` solicita cancelación. Validar esquema y pertenencia antes de consultar credenciales o iniciar procesos. No existe un endpoint que acepte una URL o comando arbitrario del modelo.

**Datos específicos normalizados:** client_id, workspace_id, URI canónica, versión, hash, rango, lenguaje y diagnósticos sin contenido ajeno.

El resultado incluye `schema_version`, identidad del recurso, instante, versión de herramienta/API, alcance, evidencia saneada, advertencias y estado de cobertura. La respuesta puede terminar con hallazgos aunque el proceso haya funcionado correctamente. Diferenciar ausencia de resultados, permisos insuficientes, cobertura parcial y ejecución fallida.

## 6. Archivos y cambios previstos en AlexIA

Los archivos nuevos se crean durante la implementación. Evitar duplicar contratos del núcleo en el conector.

| Ruta relativa al repositorio | Responsabilidad |
|---|---|
| `extensions/vscode/package.json` | Manifest, comandos y requisitos |
| `extensions/vscode/src/extension.ts` | Activación y cliente local |
| `extensions/vscode/src/context.ts` | Selección y control de URI |
| `src/integrations/vscode.rs` | Pairing y contrato cliente |
| `tests/integrations_vscode.rs` | Autorización de cliente y versiones |

`src/lib.rs` registraría los módulos; `src/control.rs` delegaría rutas administrativas al núcleo de integraciones. `ui/src/main.js` consumiría estados y resultados; `ui/index.html`/`ui/src/components.js` presentarían la conexión. Conservar las guardas loopback actuales y actualizar [BACKEND.md](../../.agents/.docs/BACKEND.md), [UI.md](../../.agents/.docs/UI.md) y el documento operativo correspondiente al implementar.

## 7. Configuración propuesta

| Campo lógico | Tipo/protección | Uso |
|---|---|---|
| `client_id` | Identificador | Extensión emparejada |
| `backend_origin` | Origen local validado | Sin destinos remotos arbitrarios |
| `workspace_ids` | Lista | Proyectos autorizados |
| `pairing_credential_ref` | Referencia secreta | SecretStorage en editor; verificador en backend |

Estos campos pertenecen a un registro validado de integración, no a un conjunto de secretos nuevos en `.env`. Escribirlos desde la API local con control de revisión. Exponer solamente `configured`, permisos y errores saneados. La presencia de configuración no equivale a conexión verificada.

## 8. Implementación por entregas verificables

### 02.1 — Emparejar y consultar trabajos

**Trabajo:** Crear extensión mínima y endpoint local autenticado.

**Dependencias:** BC-01 a BC-05 de PLAN00; completar los prerrequisitos de este documento.

**Aceptación:**

- [ ] Sin pairing se rechaza el cliente.
- [ ] Revocar rompe la sesión del editor.

**Verificación:** Pruebas de protocolo y Extension Development Host.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### 02.2 — Enviar selección y navegar resultado

**Trabajo:** Capturar rango/version y mostrar diagnósticos navegables.

**Dependencias:** Tarea 02.1 completada y contrato de salida estable.

**Aceptación:**

- [ ] No se envían archivos no seleccionados.
- [ ] Documento cambiado marca resultado obsoleto.

**Verificación:** Fixture de contenido y prueba real con documento editado.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### 02.3 — Previsualizar diferencias

**Trabajo:** Añadir documento virtual y diff de propuesta.

**Dependencias:** Tarea 02.2 completada y contrato de salida estable.

**Aceptación:**

- [ ] Vista previa no escribe.
- [ ] No se abre una URI fuera del proyecto.

**Verificación:** Prueba de extensión con URI hostil y revisión visual del diff.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### Checkpoint de salida

- [ ] Primera operación real completa desde UI hasta proveedor/herramienta y resultado visible.
- [ ] Denegación comprobada al cambiar identidad, proyecto o recurso autorizado.
- [ ] Cancelación, error y desconexión dejan estados coherentes y permiten recuperación.
- [ ] No se modificó el contrato de firma, certificado, timestamp e ID de Alexa.
- [ ] La documentación distingue simulación, prueba real y funciones pendientes.

## 9. Errores, límites y recuperación

Handshake y creación de tarea con timeout corto; el trabajo continúa en backend aunque se cierre la vista. Reconexión consulta por job_id y no duplica la ejecución. Revocar cliente cancela acceso a resultados. Compatibilidad de versiones produce un error accionable.

Límites iniciales de diseño: una ejecución por proyecto, cola local acotada, cuerpo administrativo de hasta 128 KiB y resultados paginados; los artefactos grandes se entregan por identificador. Los valores se medirán antes de liberar y no describen capacidad demostrada. HTTP 401 expira la sesión; 403 no se corrige ampliando permisos silenciosamente; 429 respeta `Retry-After` cuando exista. Un timeout después de una escritura produce estado `unknown_outcome` hasta reconciliar, sin repetición ciega.

## 10. Riesgos particulares y controles

| Riesgo | Control previsto | Evidencia requerida |
|---|---|---|
| Confundir seguridad web con clientes locales | Ruta y credencial propias; conservar Origin de UI | Cliente falso denegado |
| Documento cambiado | Comparar version/hash al aplicar o navegar | Prueba de edición concurrente |
| Backend remoto equivocado | Resolver topología Remote/WSL explícitamente | Matriz host local/remote |

**Datos y retención:** Seleccionar contenido antes del envío. No hacer indexación global ni telemetría de código. El usuario puede inspeccionar qué fragmentos saldrán a su proveedor.

## 11. Pruebas del adaptador y del flujo completo

**Fixtures y pruebas de contrato:**

- Pairing vencido/reutilizado y token revocado.
- URI de otro workspace, enlace simbólico y esquemas no soportados.
- Documento sin guardar y selección Unicode con rangos correctos.

**Prueba real de aceptación:** Emparejar en Extension Development Host, analizar una selección de proyecto de prueba y abrir el resultado en la línea esperada. Desvincular y comprobar rechazo.

**Prueba de interfaz con la skill Playwright CLI:**

1. En la UI web comprobar creación y revocación del cliente.
2. Comprobar que la tarea recibida del editor muestra origen, proyecto y rango.
3. Verificar mensajes de credencial vencida sin revelar el token.
4. Usar pruebas de Extension Host para el editor: Playwright CLI sobre la UI web no demuestra que una extensión VS Code funcione.

Usar sesión de navegador desechable; obtener referencias con `snapshot` y ejecutar comprobaciones con `run-code --filename=...` cuando se necesiten aserciones. Para el estado simulado, interceptar únicamente la API propuesta e identificar el resultado como prueba de UI. La prueba real debe comprobar el recurso externo o artefacto local. No automatizar pantallas de login ni guardar cookies del usuario para sustituir APIs oficiales.

Comandos de validación cuando se implemente: `cargo test --locked --test integrations_vscode` para el archivo propuesto, `cargo check --locked --all-targets`, las pruebas de UI afectadas y `npm --prefix ui run build`. Estos comandos son objetivos; no se afirma que esos tests existan hoy. Añadir pruebas de esquema, autorización y recuperación antes de habilitar capacidades de escritura.

## 12. Despliegue, reversión y criterio de finalización

Habilitar primero para una instalación piloto mediante flag de integración y solamente las capacidades verificadas. Registrar versión y permisos. Después ampliar a un segundo proyecto y comprobar separación de conexiones; no equivale a un despliegue SaaS multiusuario.

**Reversión específica:** Deshabilitar la extensión y revocar client_id. Quitar comandos y documentos virtuales. No modificar configuraciones globales del editor ni borrar archivos del usuario.

Deshabilitar el conector impide nuevos trabajos; cancelar o drenar los existentes según su efecto. Revocar credenciales cuando corresponda. Desconectar no borra recursos ajenos ni deshace efectos externos de forma automática. La reversión del software y la compensación de una operación externa son procedimientos distintos.

**Se considera completada cuando:** Selección, resultados y revocación funcionan en Extension Host; UI web comprobada por CLI; no hay edición implícita ni exposición de contenido ajeno.

**Decisiones o condiciones pendientes:** Versiones mínimas de VS Code y soporte Remote/WSL se fijan tras piloto; publicar en Marketplace es una entrega posterior explícita.

## 13. Fuentes oficiales y revalidación

Consultadas el 2026-11-03. Sustentan los mecanismos externos; el diseño de AlexIA es una propuesta propia. Revalidar scopes, rutas, versión, licencia y condiciones de distribución antes de implementar. Los enlaces de contratos específicos prevalecen sobre ejemplos generales; una página accesible no demuestra que una cuenta tenga habilitada la función.

- [API de extensiones](https://code.visualstudio.com/api/) — Puntos de extensión
- [Webview y seguridad](https://code.visualstudio.com/api/extension-guides/webview) — CSP y canal de mensajes
- [Workspace Trust](https://code.visualstudio.com/api/extension-guides/workspace-trust) — Restricciones por confianza
