# PLAN14 — Integración de ZAP con AlexIA

Fecha: 2026-11-03. Estado: **plan propuesto; integración pendiente**. Prioridad: P3, seguridad dinámica web.
Audiencia: comunidad técnica, inicialmente instalaciones personales con proyectos autorizados.

## 1. Resultado y alcance

Automatizar análisis de una aplicación web de prueba con alcance verificable y evidencia, distinguiendo análisis pasivo de activo.

**Primer recorrido entregable:** Registrar un entorno staging autorizado, revisar plan de escaneo y recibir alertas con límites y cobertura.

**Alcance inicial:**

- Automation Framework con plan YAML revisado.
- Pasivo sobre tráfico de prueba y contexto limitado inicialmente.
- Reportes saneados y API del scanner solo local.

**Ampliaciones posteriores:** Spider y active scan como capacidades separadas, con cuenta/entorno dedicado y perfil de carga.

**Fuera de esta primera entrega:** URLs arbitrarias, escaneo activo de producción por defecto, pruebas fuera de alcance y desactivar controles TLS.

## 2. Punto de partida real y prerrequisitos

No hay ZAP ni control de targets web. El túnel Alexa no autoriza escanear la infraestructura que lo aloja.

La interfaz actual administra configuración; no existe todavía el catálogo ejecutable de conectores ni la API de trabajos descrita aquí. El estado actual y las dependencias transversales están en [PLAN00 — Base común](PLAN00-BASE-COMUN.md). Las rutas, tipos, módulos y opciones nuevos que siguen son propuestas de implementación, no contratos existentes.

**Prerrequisitos específicos:**

- Entorno autorizado y límites de host/path/método/red.
- Java/runtime o imagen ZAP fijada y runner dedicado.
- Cuenta de prueba y ventana de ejecución para etapas activas.

**Dependencias entre planes:** PLAN00, PLAN06 o runner aislado y política de egress; PLAN07 puede generar tráfico fixture.

## 3. Decisión técnica y flujo de conexión

Generar plan de Automation Framework a partir de plantilla revisada, nunca YAML arbitrario del modelo. Separar jobs passiveScan-wait, report y exitStatus según versión. Un escáner pasivo no lanza ataques, pero navegar/spider puede invocar efectos de aplicación: empezar con tráfico de fixture conocido. Restringir redirects y DNS también a nivel de red. La API ZAP no se expone mediante túnel.

**Procedimiento de conexión o habilitación:**

1. Registrar titular/autorización operativa del target y origen/path permitido.
2. Verificar entorno staging y usuario de prueba sin datos reales.
3. Elegir perfil pasivo y mostrar tráfico/efectos esperados.
4. Arrancar instancia ZAP propia con API local protegida o modo autorun sin API expuesta.
5. Ejecutar plan con presupuesto; parar si se sale del alcance.
6. Recopilar alertas saneadas, cobertura y limpiar sesión.

**Autenticación, renovación y revocación:** Credencial del target solo al scanner; clave API ZAP si se usa daemon local. La clave del scanner no autoriza acciones sobre cualquier sitio.

**Permisos externos y selección de recursos:** Permisos separados para tráfico de prueba, spider y active scan. Activar active scan exige target exacto, perfil y efectos autorizados; nunca se deriva de una solicitud genérica de revisión.

Las credenciales residen en el almacén del sistema descrito en PLAN00; la configuración conserva solamente referencias. El navegador, el modelo y la respuesta de Alexa no reciben tokens. Una instalación comunitaria no comparte credenciales con otra. Si el producto exige un servicio OAuth confidencial, su secreto permanece en ese servicio; nunca se distribuye en el frontend, extensión o binario público.

## 4. Capacidades autorizables

Los identificadores de esta tabla son capacidades internas de AlexIA; no son nombres de scopes del proveedor.

| Capacidad propuesta | Operación | Guarda específica |
|---|---|---|
| `zap.passive.run` | Analizar tráfico permitido | Contexto y flujo conocidos |
| `zap.active.run` | Escaneo activo posterior | Autorización y ventana concretas |
| `zap.reports.read` | Leer alertas | Evidencia saneada y cobertura |

Las lecturas pueden preautorizarse para recursos concretos. Las escrituras o ejecuciones se sujetan a una política explícita por proyecto. Una política previa válida evita pedir confirmación repetida; si falta, preparar una vista previa concreta. La autorización queda ligada a identidad, recurso, parámetros y versión. Texto de un issue, archivo o registro no modifica permisos.

## 5. Superficie externa y contrato del adaptador

| Operación | API, comando o mecanismo | Tratamiento de la respuesta |
|---|---|---|
| Automation Framework | zap.sh -cmd -autorun <plan-revisado> en runtime soportado | Plan por hash y reportes propios |
| Contexto | Hosts/paths incluidos y excluidos | Denegar redirecciones fuera de alcance |
| Reporte | Job report con formato validado | No incluir cuerpos/credenciales completos |

No reutilizar ejemplos de zap-baseline como si fueran idénticos a Automation Framework. Fijar complementos/versión para que el plan sea reproducible. Riesgo de alerta y confirmación de vulnerabilidad son campos distintos. Reportar qué URLs fueron observadas y cuáles no.

**Petición interna ilustrativa:**

```json
{
  "integration_id": "zap",
  "project_id": "project-demo",
  "capability": "zap.passive.run",
  "input": {
    "target_id": "staging-demo",
    "profile_id": "passive-reviewed",
    "authorization_id": "scope-demo"
  },
  "idempotency_key": "demo-zap-001"
}
```

Se envía al propuesto `POST /api/jobs`, exclusivo de la API administrativa autenticada. Respuesta: HTTP 202 con `job_id`, `status` y `status_url`. `GET /api/jobs/{job_id}` consulta progreso; `POST /api/jobs/{job_id}/cancel` solicita cancelación. Validar esquema y pertenencia antes de consultar credenciales o iniciar procesos. No existe un endpoint que acepte una URL o comando arbitrario del modelo.

**Datos específicos normalizados:** target/context, profile_hash, alert_id/risk/confidence, URL saneada, método, evidencia limitada y coverage.

El resultado incluye `schema_version`, identidad del recurso, instante, versión de herramienta/API, alcance, evidencia saneada, advertencias y estado de cobertura. La respuesta puede terminar con hallazgos aunque el proceso haya funcionado correctamente. Diferenciar ausencia de resultados, permisos insuficientes, cobertura parcial y ejecución fallida.

## 6. Archivos y cambios previstos en AlexIA

Los archivos nuevos se crean durante la implementación. Evitar duplicar contratos del núcleo en el conector.

| Ruta relativa al repositorio | Responsabilidad |
|---|---|
| `src/integrations/zap.rs` | Plan y parser |
| `security/zap/profiles/` | Plantillas revisadas futuras |
| `tests/integrations_zap.rs` | Alcance y redacción |
| `tests/fixtures/zap/` | Aplicación y reportes fixture |

`src/lib.rs` registraría los módulos; `src/control.rs` delegaría rutas administrativas al núcleo de integraciones. `ui/src/main.js` consumiría estados y resultados; `ui/index.html`/`ui/src/components.js` presentarían la conexión. Conservar las guardas loopback actuales y actualizar [BACKEND.md](../../.agents/.docs/BACKEND.md), [UI.md](../../.agents/.docs/UI.md) y el documento operativo correspondiente al implementar.

## 7. Configuración propuesta

| Campo lógico | Tipo/protección | Uso |
|---|---|---|
| `runtime_image` | Digest | ZAP fijado |
| `target_id` | Referencia | Origen/path autorizados |
| `profile_id` | Hash | Perfil revisado |
| `target_credentials_ref` | Secreto | Cuenta fixture |
| `max_duration_seconds` | Entero | Propuesto 300 pasivo |

Estos campos pertenecen a un registro validado de integración, no a un conjunto de secretos nuevos en `.env`. Escribirlos desde la API local con control de revisión. Exponer solamente `configured`, permisos y errores saneados. La presencia de configuración no equivale a conexión verificada.

## 8. Implementación por entregas verificables

### 14.1 — Registrar alcance

**Trabajo:** Validar target, redirects y perfil.

**Dependencias:** BC-01 a BC-05 de PLAN00; completar los prerrequisitos de este documento.

**Aceptación:**

- [ ] Host fuera de alcance bloqueado.
- [ ] Activo no se habilita implícitamente.

**Verificación:** Fixture DNS/redirect y permisos.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### 14.2 — Ejecutar pasivo real

**Trabajo:** Procesar tráfico conocido y reporte.

**Dependencias:** Tarea 14.1 completada y contrato de salida estable.

**Aceptación:**

- [ ] Alerta tiene evidencia saneada.
- [ ] URLs no visitadas figuran sin cobertura.

**Verificación:** Aplicación local sintética.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### 14.3 — Cancelar y preparar activo

**Trabajo:** Cerrar instancia propia y documentar política activa.

**Dependencias:** Tarea 14.2 completada y contrato de salida estable.

**Aceptación:**

- [ ] Cancelación detiene tráfico.
- [ ] API scanner no es pública.

**Verificación:** Captura de red fixture y UI CLI.

**Tamaño objetivo:** S/M, uno a cinco archivos; dividir la tarea antes de implementarla si excede ese tamaño.

### Checkpoint de salida

- [ ] Primera operación real completa desde UI hasta proveedor/herramienta y resultado visible.
- [ ] Denegación comprobada al cambiar identidad, proyecto o recurso autorizado.
- [ ] Cancelación, error y desconexión dejan estados coherentes y permiten recuperación.
- [ ] No se modificó el contrato de firma, certificado, timestamp e ID de Alexa.
- [ ] La documentación distingue simulación, prueba real y funciones pendientes.

## 9. Errores, límites y recuperación

Deadline global, límites de solicitudes/concurrencia y ventana de escaneo. Mantener tasa baja en piloto y medir impacto. Timeout da partial; no reintentar active scan automáticamente.

Límites iniciales de diseño: una ejecución por proyecto, cola local acotada, cuerpo administrativo de hasta 128 KiB y resultados paginados; los artefactos grandes se entregan por identificador. Los valores se medirán antes de liberar y no describen capacidad demostrada. HTTP 401 expira la sesión; 403 no se corrige ampliando permisos silenciosamente; 429 respeta `Retry-After` cuando exista. Un timeout después de una escritura produce estado `unknown_outcome` hasta reconciliar, sin repetición ciega.

## 10. Riesgos particulares y controles

| Riesgo | Control previsto | Evidencia requerida |
|---|---|---|
| Salir de alcance | Egress/redirect/DNS y contexto | Target externo centinela no recibe tráfico |
| Efectos de navegación | Flujos fixture y cuentas desechables | Datos de prueba revisados |
| API ZAP expuesta | Loopback/clave o autorun | Verificación de puertos |

**Datos y retención:** URLs, cookies y cuerpos de reporte pueden ser sensibles. Eliminar query secrets, cabeceras y sesiones al cerrar; limitar artefactos.

## 11. Pruebas del adaptador y del flujo completo

**Fixtures y pruebas de contrato:**

- Redirect y DNS a destino no permitido.
- Alerta con cookie/Authorization y confianza baja.
- Timeout, API no autorizada y reporte parcial.

**Prueba real de aceptación:** Ejecutar perfil pasivo sobre aplicación fixture local, verificar alertas esperadas y ausencia de tráfico al destino fuera de scope.

**Prueba de interfaz con la skill Playwright CLI:**

1. Revisar target y perfil.
2. Ver que activo requiere permiso específico.
3. Ejecutar fixture y abrir alerta saneada.
4. Cancelar y contrastar que no continúa tráfico.

Usar sesión de navegador desechable; obtener referencias con `snapshot` y ejecutar comprobaciones con `run-code --filename=...` cuando se necesiten aserciones. Para el estado simulado, interceptar únicamente la API propuesta e identificar el resultado como prueba de UI. La prueba real debe comprobar el recurso externo o artefacto local. No automatizar pantallas de login ni guardar cookies del usuario para sustituir APIs oficiales.

Comandos de validación cuando se implemente: `cargo test --locked --test integrations_zap` para el archivo propuesto, `cargo check --locked --all-targets`, las pruebas de UI afectadas y `npm --prefix ui run build`. Estos comandos son objetivos; no se afirma que esos tests existan hoy. Añadir pruebas de esquema, autorización y recuperación antes de habilitar capacidades de escritura.

## 12. Despliegue, reversión y criterio de finalización

Habilitar primero para una instalación piloto mediante flag de integración y solamente las capacidades verificadas. Registrar versión y permisos. Después ampliar a un segundo proyecto y comprobar separación de conexiones; no equivale a un despliegue SaaS multiusuario.

**Reversión específica:** Detener instancia propia, revocar cuenta fixture si procede y eliminar sesión. No intentar deshacer efectos del target mediante borrados automáticos.

Deshabilitar el conector impide nuevos trabajos; cancelar o drenar los existentes según su efecto. Revocar credenciales cuando corresponda. Desconectar no borra recursos ajenos ni deshace efectos externos de forma automática. La reversión del software y la compensación de una operación externa son procedimientos distintos.

**Se considera completada cuando:** Escaneo fixture respeta alcance, API y límites; evidencia y cobertura son correctas, sin habilitación activa implícita.

**Decisiones o condiciones pendientes:** Perfiles y tecnología de aislamiento; pruebas activas se habilitan solo tras validación del entorno.

## 13. Fuentes oficiales y revalidación

Consultadas el 2026-11-03. Sustentan los mecanismos externos; el diseño de AlexIA es una propuesta propia. Revalidar scopes, rutas, versión, licencia y condiciones de distribución antes de implementar. Los enlaces de contratos específicos prevalecen sobre ejemplos generales; una página accesible no demuestra que una cuenta tenga habilitada la función.

- [Automatización ZAP](https://www.zaproxy.org/docs/automate/) — Opciones de integración
- [Automation Framework](https://www.zaproxy.org/docs/desktop/addons/automation-framework/) — Jobs y planes
