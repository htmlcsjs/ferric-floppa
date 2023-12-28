INSERT into commands(name, owner, type, registry, added, data) 
VALUES(?, ?, ?, ?, ?, ?)
RETURNING id;