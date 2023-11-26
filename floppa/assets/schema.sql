CREATE TABLE IF NOT EXISTS registries(
    id        INTEGER  PRIMARY KEY AUTOINCREMENT,
    name      TEXT     NOT NULL UNIQUE,
    super     TEXT     NULL
);
CREATE TABLE IF NOT EXISTS commands(
    id        INTEGER  PRIMARY KEY AUTOINCREMENT,
    name      TEXT     NOT NULL UNIQUE,
    owner     INTEGER  NOT NULL,
    type      TEXT     NOT NULL,
    registry  INTEGER  NOT NULL UNIQUE,
    added     INTEGER  NULL,
    data      BLOB     NULL,
    FOREIGN KEY(registry) REFERENCES registries(id)
);