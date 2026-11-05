export function hasChanges(saved, values, metaprompt) {
  if (!saved) return false;
  const original = new Map(saved.environment.map(field => [field.key, field.value]));
  return metaprompt.replace(/\r\n?/g, '\n') !== saved.metaprompt.replace(/\r\n?/g, '\n') || values.size !== original.size
    || [...values].some(([key, value]) => !original.has(key) || original.get(key) !== value);
}
