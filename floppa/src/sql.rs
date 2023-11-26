use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use serenity::{futures::TryStreamExt, model::id::UserId};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    FromRow, Pool, Sqlite,
};

use crate::{
    command::{self, Command},
    Cli, FlopResult,
};

#[derive(Debug)]
pub struct FlopDB {
    pool: Pool<Sqlite>,
    commands: HashMap<(String, String), Arc<Mutex<CommandEntry>>>,
    registries: HashMap<String, RegistryRow>,
}

impl FlopDB {
    pub async fn init(cli: &Cli) -> FlopResult<Self> {
        let db_file = cli.get_path("flop.db");

        let pool = SqlitePoolOptions::new()
            .connect(&format!("sqlite://{}", db_file.display()))
            .await?;
        pool.set_connect_options(
            SqliteConnectOptions::new()
                .create_if_missing(true)
                .foreign_keys(true),
        );

        // Include the table schema from a seperate file
        sqlx::query_file!("assets/schema.sql")
            .execute(&pool)
            .await?;

        let mut commands: HashMap<(String, String), Arc<Mutex<CommandEntry>>> = HashMap::new();
        let mut rows = sqlx::query_file!("assets/get_commands.sql").fetch(&pool);

        while let Some(row) = rows.try_next().await? {
            // Get raw data from the row
            // Parse and "transform" the values
            let owner = UserId::from(row.owner as u64);
            let added = row.added.unwrap_or_default() as u64;
            let data = if let Some(actual) = row.data {
                rmp_serde::from_slice(&actual)?
            } else {
                rmpv::Value::Nil
            };

            // TODO maybe move to a seperate class?
            let key = (row.registry.clone(), row.name.clone());

            let cmd = CommandEntry {
                id: row.id,
                name: row.name,
                owner,
                inner: command::construct(&row.ty, data, cli),
                ty: row.ty,
                added,
                registry: row.registry,
            };

            commands.insert(key, Arc::new(Mutex::new(cmd)));
        }

        // Drop rows to get rid of a borrow on pool
        drop(rows);

        let registries: Vec<RegistryRow> = sqlx::query_as!(
            RegistryRow,
            "SELECT id, name, super as parent FROM registries;"
        )
        .fetch_all(&pool)
        .await?;

        Ok(Self {
            pool,
            commands,
            registries: registries
                .into_iter()
                .map(|x| (x.name.clone(), x))
                .collect(),
        })
    }
}

#[derive(Debug)]
pub struct CommandEntry {
    id: i64,
    name: String,
    owner: UserId,
    ty: String,
    added: u64,
    registry: String,
    inner: Box<dyn Command + Send + Sync>,
}

#[derive(Debug, FromRow)]
pub struct RegistryRow {
    id: i64,
    name: String,
    #[sqlx(rename = "super")]
    parent: Option<String>,
}
