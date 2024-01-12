# FLOPPA DATABASE SCHEMA

This is a list of schema used by c in its SQLite DB

## Info

Types in this document are listed as just the equalivent rust type, because of SQLite's type
flexability, **unless** it is the primary key, where the type is `key`, which is a unique
`i64` to rust

## Commands

These all the actual commands, even commands like "help" and other system commands.  

| Name     | Type     | Description                                                                         |
|----------|----------|-------------------------------------------------------------------------------------|
| id       | `key`    | The ID of the command                                                               |
| name     | `String` | The name of the command, e.g. `help` or `about`                                     |
| owner    | `u64`    | The ID of the discord account that owns the command                                 |
| added    | `i64`    | Unix timestamp of when the command was added                                        |
| type     | `String` | The name of the type of the command                                                 |
| data     | `[u8]`   | Binary data in the MessagePack format, used for custom data for the command to save |
| registry | `i64`    | The registry that the command is in, foreign key                                    |

## Registry

These are collections of commands that can have different modification and management permissions

| Name  | Type             | Description                                                                    |
|-------|------------------|--------------------------------------------------------------------------------|
| id    | `key`            | The ID of the registry used by the DB                                          |
| name  | `String`         | The name of the registry, e.g `root` or `custom`                               |
| super | `Option<String>` | Optionally, a name of another registry that this one can inherit commands from |

## Users

This is all the users that have roles (e.g. Admin, Subregistry roles)

| Name  | Type          | Description                                               |
|-------|---------------|-----------------------------------------------------------|
| user  | `key`         | The Id of the user, stored as a i64 by some hacky casting |
| roles | `Vec<Roles>`  | The list of roles that the user has, in msgpack form      |
