# Auditoría de consistencia — PLAN00 y GitHub R2

Fecha: 2026-11-03. Revisión: **C1**. Estado: **11 conflictos corregidos en los contratos; una condición de habilitación técnica pendiente**.

## Resultado

Se contrastaron [PLAN00](../PLAN00-BASE-COMUN.md), [PLAN01](../PLAN01-GITHUB.md) y su dependencia lectora [PLAN05](../PLAN05-GITHUB-ACTIONS.md), además de las reglas del repositorio y el código existente de Alexa/proveedor. La revisión anterior de enlaces/JSON no detectaba conflictos de ejecución entre planes; esta revisión evalúa también orden de operaciones, identidad, continuidad y efectos.

No se implementaron conectores ni se llamó a APIs operativas GitHub. Las correcciones dejan el diseño coherente; no prueban funcionamiento en producción. Las 41 comprobaciones Playwright de la entrega anterior siguen siendo evidencia de UI/API actual, no de estos contratos futuros.

## Hallazgos y tratamiento

| ID / prioridad | Conflicto antes de C1 | Consecuencia si se implementaba literalmente | Corrección / estado |
|---|---|---|---|
| C01 / P1 | PLAN00 exige project_id antes del job; GitHub descubre proyecto buscando última rama/PR | Búsqueda dentro de la petición Alexa, timeout o project_id ficticio para saltar validación | Job selection por conexión sin efectos; hijo execution con ProjectRef/grant real. Corregido en ambos |
| C02 / P1 | ProjectRef y BC-01 solo describen raíz local; GitHub puede operar sobre un repo sin checkout | Pedir carpetas innecesarias, inventar raíz o negar todos los repos remotos | source_kind local/remote_repository con IDs estables y snapshot propio. Corregido |
| C03 / P1 | Digest formaba parte de la clave de idempotencia, pero se prometía conflicto al repetir key con otra entrada | Otra entrada genera otra clave y evita el 409; una orden duplicada podría seleccionar un push más nuevo y crear otra PR | Clave de namespace estable; digest almacenado para comparar; requestId y selección persistidos. Corregido |
| C04 / P1 | TTL genérico de jobs/reportes de 24 h sin retención distinta de CreationReceipt | Una PR bot abierta varios días deja de ser atribuible al usuario o se intenta reconstruir por labels | Recibo independiente durante PR abierta y plazo posterior propuesto; efectos sin reconciliar no se purgan. Corregido |
| C05 / P1 | Enum común de jobs no incluye espera de usuario/CI; GitHub mezcla etapa, estado y resultado y no define continuación | UI/cancelación no entienden waiting_for_checks; no hay forma segura de responder tras cerrar Alexa | Estados canónicos, stage/result_code separados y continuation_id ligado a usuario/revisión/TTL. Corregido |
| C06 / P1 | API común solo dice sesión administrativa y una credential_ref; GitHub necesita Alexa y varios roles de credencial | Reusar cookie/cabecera de UI como autenticación de voz, confundir bot/humano o revocar a otros usuarios | CommandService interno, credenciales por rol y ownership; sin fallback de user token a instalación. Corregido |
| C07 / P1 | Eventos/notificaciones eran posteriores y polling «suficiente», pero G03 necesita pushes observables | Rama incorrecta tras caída del receptor o falso modo automático completo | BC-07a temprano/durable y BC-07b posterior; huecos de cobertura bloquean selección automática. Corregido; recepción real por validar |
| C08 / P1 | Grant común atado a hash; GitHub permite seguir head nuevo, reanalizar/publicar creación y reparar metadatos sin delimitar los grants | Grant se invalida o se amplía silenciosamente según qué documento se siga | Snapshot default; creación con SHA cambiado termina sin POST, reparación PATCH no implícita; follow_ref explícito para merge y gate nuevo sin ampliar plazo/PR/base. Corregido |
| C09 / P1 | BC-05 pide primer conector real, pero G01 depende de BC-05; BC-06 y Actions también se podían interpretar como completos | Ciclo de implementación o botones dados por listos antes de tener cliente | Núcleo fixture y aceptación real separados; grafo canónico de 25 nodos y CI-READ solo depende de G01. Corregido |
| C10 / P1 | PLAN00 considera IA opcional; creación GitHub exige texto analizado por Antigravity | Publicar PR vacía/inventada o bloquear CI/merge por caída del modelo | Readiness por capacidad; creación exige perfil técnico aislado, lecturas/merge no lo invocan. Corregido; aislamiento real por validar |
| C11 / P1 | Núcleo en RAM y TTL genérico frente a POST/PUT/retarget con efectos duraderos | Caída después del envío duplica PR/merge; cancelación puede figurar exitosa aunque hubo efecto | BC-02b journal/EffectIntent antes de envío, recibo y reconciliación; exclusión con cancelación/revocación. Corregido en contrato |
| C12 / P1 — habilitación bloqueada | Se reconoce que sha condiciona head, pero se podía habilitar merge con solo relectura de base y lock local | Otro actor retargetea entre lectura y merge: squash puede ir a destino distinto; revisión posterior no lo deshace | Guard de destino obligatorio; unverified bloquea antes de PATCH/PUT. No se ha demostrado aún un mecanismo suficiente: condición técnica abierta |

«Corregido» significa que se corrigieron los documentos y sus dependencias; los tests de implementación continúan pendientes. No se marca C12 resuelto por ponerle un flag.

## Comprobaciones de coherencia efectuadas

- La variante selection de ambos documentos no admite project_id ni capacidades de escritura; execution requiere un proyecto real y reautorización.
- Ninguna muestra JSON mantiene un project_id ficticio como sustituto de una selección.
- Antigravity recibe snapshot/redacción, no credenciales GitHub ni control de permisos.
- Una orden duplicada conserva el objetivo original; una orden nueva puede resolver otro objetivo.
- El grant follow_ref no autoriza cambiar PR/base ni extender deadline.
- Espera/continuación/cancelación tiene representación común y no requiere mantener Alexa abierta durante CI.
- Journal y procedencia no desaparecen al vencer el reporte ordinario; se reconoce la pérdida real de evidencia.
- No se necesita conector terminado para construir el núcleo fixture o el lector CI.
- GitHub G08/G09 no dependen de disponibilidad runtime de Antigravity.
- Un guard de destino unverified deja posible diagnóstico, pero no retarget ni merge.

## Escenarios de aceptación que deberá ejecutar la implementación

Estas son expectativas del contrato, **no tests de producto ya ejecutados**:

| Escenario | Resultado exigido |
|---|---|
| 50 repositorios tardan más que la ventana Alexa | Acuse selection temprano; búsqueda posterior sin escritura prematura |
| Repo autorizado sin copia local | ProjectRef remoto válido, sin leer carpeta personal |
| Se repite requestId después de un push nuevo | Mismo padre/candidato/resultado; no segunda selección |
| Misma clave con entrada cambiada | 409 y ningún efecto adicional |
| Usuario B continúa una selección de A | Denegación por principal/scope |
| Continuación vencida o replay | Rechazo; no hijo execution |
| PR bot sigue abierta al tercer día | Recibo permite comprobar procedencia aunque el reporte expiró |
| El recibo fue borrado o pertenece a otra identidad | No adoptar automáticamente la PR |
| Dos usuarios comparten instalación y uno se desconecta | Revocar al primero; no desinstalar/revocar al segundo |
| Webhook falla y nadie pide redelivery | Cobertura desconocida; no presumir evento recibido |
| Head avanza con snapshot grant | Detenerse sin fusionar versión nueva |
| Head cambia durante redacción de creación | stale_snapshot y ningún POST con el grant anterior |
| Carrera cambia head tras POST de creación | Informar metadata_stale; no PATCH de reparación sin capacidad/grant |
| Head avanza con follow_ref explícito | Mismo PR/destino, nuevo snapshot/gate dentro del plazo original |
| Antigravity falla | No publicar PR inventada; permitir diagnósticos independientes |
| Se cancela antes de registrar/enviar EffectIntent | No enviar efecto |
| Se cancela después de enviar PUT | Reconciliar; no afirmar que el merge se deshizo |
| Caída entre POST y guardado de recibo | Reconciliar desde journal antes de repetir |
| Cambia política/permisos durante espera | Invalidar escritura incluso si los tests están verdes |
| Base retargeteada concurrentemente | Guard suficiente demostrado o ningún PATCH/PUT automático |
| Última PR usa mecanismo no soportado | Explicar restricción; no fusionar una anterior |

## Condiciones técnicas aún pendientes

1. **Destino de merge:** demostrar una solución verificable para la precondición requerida. REST expone sha para head y merge_method, no expected_base_sha/ref. No se identificó en esta revisión un mecanismo atómico suficiente para todos los repositorios; el modo desatendido no se libera con una promesa de coordinación informal.
2. **Aislamiento Antigravity:** verificar el perfil técnico en host real. cwd/mode plan y una instrucción «no uses herramientas» no constituyen aislamiento de secretos o filesystem.
3. **Observabilidad de pushes:** validar receptor/journal y recuperación. Sin historial fiable, la interacción debe pedir selección y declarar modo automático no disponible.
4. **Configuración por repositorio:** bases, aliases, fuentes de tests, revisiones y políticas se comprueban en fixtures remotas antes de habilitar escritura. El plan no modifica configuración externa.

Estas condiciones no requieren detener el trabajo de núcleo, lectura, selección o creación independiente. Sí impiden declarar completo lo que aún no tiene evidencia, y C12 impide ejecutar retarget/merge automático.

## Evidencia y límites de verificación

El [resultado de revisión mecánica](PLAN00-GITHUB-CONSISTENCIA.json) comprueba estructura, ejemplos JSON, referencias locales y grafo de dependencias leído de PLAN00. Una ordenación topológica demuestra ausencia de ciclos en ese grafo, no suficiencia de permisos, rendimiento o semántica del API.

Resultado observado: grafo de 25 nodos/46 aristas sin ciclos, 16 correspondencias de contrato verificadas y 27 ejemplos JSON válidos. Los 21 planes mantienen numeración y sus enlaces locales resuelven. Cero errores en el validador documental y en git diff --check. El JSON conserva el alcance y la condición de merge sin verificar; no se borra esa condición para obtener un resultado verde.

No se volvió a ejecutar Playwright sobre una UI sin cambios: no demostraría estos contratos nuevos. No se hicieron pruebas destructivas, PR reales, cambios en Amazon o escrituras GitHub. Las validaciones de esta revisión son documentales y la configuración original permanece intacta.

## Fuentes oficiales revalidadas

- [API de pull requests y merge](https://docs.github.com/en/rest/pulls/pulls#merge-a-pull-request): parámetros de merge y precondición del head.
- [Entregas webhook fallidas](https://docs.github.com/en/webhooks/using-webhooks/handling-failed-webhook-deliveries): GitHub no garantiza reentrega automática.
- [Protecciones de rama](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches): requisitos y posibles bypass de actores, que AlexIA no debe aprovechar.

Los tipos, fases y políticas de retención son propuestas de AlexIA. Revalidar contratos y pruebas reales al implementarlos.
