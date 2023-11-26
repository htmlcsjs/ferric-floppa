SELECT commands.name, commands.id, owner, type as ty, data, added, registries.name AS registry
FROM commands, registries
WHERE registries.id = commands.registry;