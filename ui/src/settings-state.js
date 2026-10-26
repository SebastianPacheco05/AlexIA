export function hasChanges(saved, values, metaprompt) {
  if (!saved) return false;
  const original = new Map(saved.environment.map(field => [field.key, field.value]));
  return metaprompt !== saved.metaprompt || values.size !== original.size
    || [...values].some(([key, value]) => !original.has(key) || original.get(key) !== value);
}
