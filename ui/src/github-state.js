export const githubErrors = {
  disconnect_cleanup_pending: 'El acceso está revocado. Reintenta la limpieza para terminar de eliminar la credencial local.',
  app_installation_required: 'Tu cuenta está autorizada, pero falta instalar esta GitHub App en tu cuenta y seleccionar repositorios. Después pulsa Verificar conexión.',
  github_authorization_incomplete: 'La autorización anterior no se completó. Pulsa Desconectar y vuelve a Autorizar en GitHub después de instalar la App.',
  vault_unavailable: 'No se pudo acceder a la credencial guardada en Windows. Revisa que AlexIA se ejecute con tu usuario; si no existe una autorización guardada, desconecta y vuelve a autorizar.',
  authorization_failed: 'GitHub no completó la autorización. Desconecta y vuelve a autorizar la App.',
  connection_expired: 'La autorización de GitHub caducó. Desconecta y vuelve a autorizar.',
  device_flow_unavailable: 'GitHub no inició la autorización. Comprueba el Client ID y activa Enable Device Flow en tu App.',
  github_unavailable: 'No se pudo contactar con GitHub. Revisa la conexión y vuelve a intentarlo.',
};

export function githubConnectionState(connection, authorizationFlowVersion = 2) {
  const cleanupPending = connection?.last_error === 'disconnect_cleanup_pending';
  const canDisconnect = Boolean(connection && (connection.status !== 'revoked' || cleanupPending));
  if (authorizationFlowVersion < 2) {
    return {
      connected: false, canVerify: false, canDisconnect, cleanupPending, requiresRestart: true,
      label: 'Reinicia AlexIA',
      description: 'El servicio de AlexIA sigue usando una versión anterior de la autorización. Detén el lanzador con Ctrl+C y ejecuta de nuevo scripts/start-alexa.ps1. Después vuelve a desbloquear las integraciones.',
    };
  }
  const hasIdentity = Number.isSafeInteger(connection?.github_user_id) && connection.github_user_id > 0;
  const connected = connection?.status === 'connected';
  const pendingInstall = connection?.last_error === 'app_installation_required';
  const incomplete = connection && !hasIdentity && ['degraded', 'configured_unverified'].includes(connection.status);
  return {
    connected, canDisconnect, cleanupPending,
    canVerify: hasIdentity && !['connecting', 'revoked', 'expired'].includes(connection.status),
    label: cleanupPending ? 'Limpieza pendiente' : pendingInstall ? 'Falta instalar la App' : connected ? `Conectado · ${connection.login}`
      : ({ connecting: 'Esperando autorización', revoked: 'Desconectado', expired: 'Autorización caducada', degraded: 'Revisar conexión', configured_unverified: 'Pendiente de verificar' }[connection?.status] || 'Sin conectar'),
    description: pendingInstall ? githubErrors.app_installation_required
      : connection?.last_error ? githubErrors[connection.last_error] || 'No se completó la conexión. Revisa la configuración y vuelve a autorizar.'
      : incomplete ? githubErrors.github_authorization_incomplete
      : connected ? `Cuenta verificada: ${connection.login} · ${connection.repositories?.length || 0} repositorio(s) configurado(s)`
      : 'Autoriza tu cuenta mediante una GitHub App propia.',
  };
}
