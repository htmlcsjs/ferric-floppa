use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use serde::{Deserialize, Serialize};
use serenity::{
    futures::TryStreamExt,
    model::{id::UserId, Timestamp},
};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    FromRow, Pool, Sqlite,
};
use tokio::{sync::Mutex, time::Instant};
use tracing::{error, info, warn};

use crate::{
    command::{self, ExtendedCommand},
    Cli, FlopResult,
};

const COMMAND_SEARCH_DEPTH_LIMIT: usize = 64;

#[derive(Debug)]
pub struct FlopDB {
    /// SQL pool for making db edits with
    pool: Pool<Sqlite>,
    /// The core command DB for floppa
    commands: HashMap<(String, String), Arc<Mutex<CommandEntry>>>,
    /// list of all commands that are dirty and need to be synced
    dirty_commands: HashSet<(String, String)>,
    /// Map of name to registry data
    registries: HashMap<String, RegistryRow>,
    /// List of the IDs of commands that have been removed
    removed_commands: Vec<i64>,
    /// List of users with roles
    user_roles: HashMap<UserId, (Vec<FlopRole>, SyncState)>,
}

impl FlopDB {
    pub async fn init(cli: &Cli) -> FlopResult<Self> {
        let db_file = cli.get_path("flop.db");

        let pool = SqlitePoolOptions::new()
            .connect_with(
                SqliteConnectOptions::new()
                    .create_if_missing(true)
                    .foreign_keys(true)
                    .filename(db_file),
            )
            .await?;

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
            let added = row.added.unwrap_or_default();
            let data = &row.data.unwrap_or_default();
            // TODO maybe move to a seperate class?
            let key = (row.registry.clone(), row.name.clone());
            let cmd_obj = match CmdNode::load(&row.ty, data, cli) {
                Ok(cmd) => cmd,
                Err(e) => {
                    error!(
                        "Error constructing command: {} in registry {}\n{e}",
                        row.name, row.registry
                    );
                    continue;
                }
            };

            let cmd = CommandEntry {
                id: Some(row.id),
                name: row.name,
                owner,
                node: cmd_obj,
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

        let roles = sqlx::query!("SELECT id, roles FROM users;")
            .fetch_all(&pool)
            .await?;
        let mut user_roles = HashMap::with_capacity(roles.len());
        for user in roles {
            let Some(role_data) = user.roles else {
                continue;
            };
            let roles: Vec<FlopRole> = match rmp_serde::from_slice(&role_data) {
                Ok(r) => r,
                Err(e) => {
                    warn!("Error decoding roles for <@{}>\n{e}", user.id);
                    continue;
                }
            };
            user_roles.insert(UserId::from(user.id as u64), (roles, SyncState::Clean));
        }

        Ok(Self {
            pool,
            commands,
            registries: registries
                .into_iter()
                .map(|x| (x.name.clone(), x))
                .collect(),
            removed_commands: Vec::new(),
            dirty_commands: HashSet::new(),
            user_roles,
        })
    }

    pub fn get_command(&self, registry: String, name: String) -> Option<Arc<Mutex<CommandEntry>>> {
        self.commands.get(&(registry, name.to_lowercase())).cloned()
    }

    pub fn add_command(
        &mut self,
        registry: String,
        name: String,
        owner: impl Into<UserId>,
        ty: String,
        cmd: CmdNode,
    ) -> Option<Arc<Mutex<CommandEntry>>> {
        let name = name.to_lowercase();
        let entry = CommandEntry {
            id: None,
            name: name.clone(),
            owner: owner.into(),
            ty,
            added: Timestamp::now().unix_timestamp(),
            registry: registry.clone(),
            node: cmd,
        };
        self.dirty_commands.insert((registry.clone(), name.clone()));
        self.commands
            .insert((registry, name), Arc::new(Mutex::new(entry)))
    }

    pub async fn remove_command(&mut self, registry: String, name: String) -> bool {
        if let Some(entry) = self.commands.remove(&(registry, name)) {
            if let Some(id) = entry.lock().await.id {
                self.removed_commands.push(id)
            }
            true
        } else {
            false
        }
    }

    pub async fn sync(
        &self,
        dirty: HashSet<(String, String)>,
        delete: Vec<i64>,
        roles: Vec<(UserId, SyncState)>,
    ) -> FlopResult<()> {
        if dirty.is_empty() && delete.is_empty() {
            // No point doing all of this if there is nothing to act on
            info!("Nothing to sync");
            return Ok(());
        }
        let start = Instant::now();
        // Start a transaction
        let mut tx = self.pool.begin().await?;

        // every command to be synced
        for key in dirty {
            let Some(cmd) = self.commands.get(&key) else {
                continue;
            };

            // Get a lock on the command
            let mut cmd_lock = cmd.lock().await;

            if let Some(id) = cmd_lock.id {
                // If the command was editied (previously had an id)

                // Process and get data
                let reg_id = self.registries.get(&cmd_lock.registry).map_or(1, |x| x.id);
                let data = cmd_lock.node.save();
                let owner = cmd_lock.owner.get() as i64;
                // Construct the actual query
                let res = sqlx::query_file!(
                    "assets/update_command.sql",
                    cmd_lock.name,
                    owner,
                    cmd_lock.ty,
                    reg_id,
                    data,
                    id
                )
                .execute(&mut *tx)
                .await;
                // Handle Error
                if let Err(e) = res {
                    error!(
                        "Error saving command {}:{}```rust\n{e}```",
                        cmd_lock.registry, cmd_lock.name
                    )
                }
            } else {
                // If the command doesnt have an id (is new)

                // Process data
                let reg_id = self.registries.get(&cmd_lock.registry).map_or(1, |x| x.id);
                let data = cmd_lock.node.save();
                let owner = cmd_lock.owner.get() as i64;
                // Construct the query
                let res = sqlx::query_file!(
                    "assets/add_command.sql",
                    cmd_lock.name,
                    owner,
                    cmd_lock.ty,
                    reg_id,
                    cmd_lock.added,
                    data,
                )
                .fetch_one(&mut *tx)
                .await;
                // Update the command entry with the id
                match res {
                    Ok(id) => cmd_lock.id = Some(id.id),
                    Err(e) => error!(
                        "Error saving command {}:{}```rust\n{e}```",
                        cmd_lock.registry, cmd_lock.name
                    ),
                }
            }
        }

        // For commands to be deleted
        for id in delete {
            // Construct the query
            let res = sqlx::query!("DELETE FROM commands WHERE id = ?;", id)
                .execute(&mut *tx)
                .await;
            // handle errors
            if let Err(e) = res {
                error!("Error deleting command {id}```rust\n{e}```");
            }
        }

        // Sync roles
        for (user, state) in roles {
            match state {
                SyncState::Dirty => {
                    let Some((roles, _)) = self.user_roles.get(&user) else {
                        continue;
                    };
                    let data = rmp_serde::to_vec(roles)?;
                    let id = user.get() as i64;
                    let res = sqlx::query!("UPDATE users SET roles = ? WHERE id = ?;", data, id)
                        .execute(&mut *tx)
                        .await;
                    if let Err(e) = res {
                        error!("Error modifying user roles{id}```rust\n{e}```");
                    }
                }
                SyncState::New => {
                    let Some((roles, _)) = self.user_roles.get(&user) else {
                        continue;
                    };
                    let data = rmp_serde::to_vec(roles)?;
                    let id = user.get() as i64;
                    let res = sqlx::query!("INSERT INTO users(id, roles) VALUES(?, ?);", id, data)
                        .execute(&mut *tx)
                        .await;
                    if let Err(e) = res {
                        error!("Error adding user roles{id}```rust\n{e}```");
                    }
                }
                SyncState::Clean => (),
                SyncState::Deleted => {
                    let id = user.get() as i64;
                    let res = sqlx::query!("DELETE FROM users WHERE id = ?;", id)
                        .execute(&mut *tx)
                        .await;
                    if let Err(e) = res {
                        error!("Error deleting user roles{id}```rust\n{e}```");
                    }
                }
            }
        }

        // Commit the changes to the DB
        tx.commit().await?;

        // Write info out
        info!("Synced to DB in {:?}", start.elapsed());

        Ok(())
    }

    /// Marks a command to be synced on next cycle
    pub fn mark_dirty(&mut self, registry: String, name: String) {
        self.dirty_commands.insert((registry, name));
    }

    /// Drains all of the dirty commands out of cache
    #[must_use]
    pub fn drain_dirty(&mut self) -> HashSet<(String, String)> {
        let ret = self.dirty_commands.clone();
        self.dirty_commands.clear();
        ret
    }

    /// Drains all of the dirty commands out of cache
    #[must_use]
    pub fn drain_removed(&mut self) -> Vec<i64> {
        let ret = self.removed_commands.clone();
        self.removed_commands.clear();
        ret
    }

    /// Drains all of the dirty commands out of cache
    #[must_use]
    pub fn drain_roles(&mut self) -> Vec<(UserId, SyncState)> {
        let ret = self
            .user_roles
            .iter()
            .filter(|(_, (_, state))| *state != SyncState::Clean)
            .map(|(id, (_, state))| (*id, *state))
            .collect();

        self.user_roles
            .retain(|_, (_, state)| *state != SyncState::Deleted);

        self.user_roles
            .values_mut()
            .for_each(|(_, status)| *status = SyncState::Clean);

        ret
    }

    /// Function to follow symlink/subregistries to find the actual command to call
    pub async fn canonicalise_command(
        &self,
        mut registry: String,
        name: String,
    ) -> CanonicalsedResult {
        let mut result = CanonicalsedResult::default();
        let mut words = name.split_whitespace();
        let Some(mut search_name) = words.next().map(|x| x.to_owned()) else {
            return result;
        };
        result.stack.push((registry.clone(), search_name.clone()));

        for _ in 0..COMMAND_SEARCH_DEPTH_LIMIT {
            if let Some(cmd) = self.get_command(registry.clone(), search_name.clone()) {
                let mut cmd_lock = cmd.lock().await;
                let node = cmd_lock.get_node();
                match node {
                    CmdNode::Cmd(_) => {
                        result.call += " ";
                        result.call += &search_name;
                        result.call = result.call.trim().to_owned();
                        result.status = CanonicalisedStatus::Success;
                        return result;
                    }
                    CmdNode::Subregistry(reg) => {
                        registry = reg.to_owned();
                        result.call += " ";
                        result.call += &search_name;
                        result.call = result.call.trim().to_owned();
                        if let Some(name) = words.next() {
                            search_name = name.to_owned()
                        } else {
                            result.status = CanonicalisedStatus::FailedSubcommand;
                            return result;
                        };
                    }
                    CmdNode::Symlink { reg, name } => {
                        registry = reg.to_owned();
                        search_name = name.to_owned();
                    }
                }
            } else if let Some(reg_entry) = self.registries.get(&registry) {
                if let Some(parent) = &reg_entry.parent {
                    registry = parent.to_owned()
                } else {
                    return result;
                }
            } else {
                return result;
            }
            let new_val = (registry.clone(), search_name.clone());
            if result.stack.contains(&new_val) {
                result.stack.push(new_val);
                result.status = CanonicalisedStatus::Recursive;
                return result;
            } else {
                result.stack.push(new_val);
            }
        }
        result.status = CanonicalisedStatus::Overflow;
        result
    }

    /// Checks if a command exists
    pub fn command_exists(&self, registry: String, name: &str) -> bool {
        self.commands.contains_key(&(registry, name.to_lowercase()))
    }

    /// Check if a user has a role, or a valid parent
    pub fn user_has_role(&self, user: UserId, role: &FlopRole) -> bool {
        if let Some((roles, sync)) = self.user_roles.get(&user) {
            // Dont check roles that have been "deleted"
            if *sync == SyncState::Deleted {
                return false;
            }

            // Specal case banned users
            if roles.contains(&FlopRole::Banned) {
                return false;
            }
            let mut role_found = roles.contains(role);
            while !role_found {
                let parent = role.get_parent();
                if let Some(role) = parent {
                    role_found = roles.contains(&role)
                } else {
                    return role_found;
                }
            }
            role_found
        } else {
            false
        }
    }

    /// Give a user a role
    pub fn give_role(&mut self, user: UserId, role: FlopRole) {
        let Some(inner) = self.user_roles.get_mut(&user) else {
            self.user_roles.insert(user, (vec![role], SyncState::New));
            return;
        };

        let roles = &mut inner.0;
        inner.1 = SyncState::Dirty;

        // Check if the user is banned, if so they dont get a role
        if roles.contains(&FlopRole::Banned) {
            *roles = vec![FlopRole::Banned]
        } else {
            roles.push(role);
        }
    }

    /// Remove a role from a user
    pub fn remove_role(&mut self, user: UserId, role: FlopRole) {
        let Some(inner) = self.user_roles.get_mut(&user) else {
            return;
        };

        let roles = &mut inner.0;
        inner.1 = SyncState::Dirty;

        // Check if the user is banned, if so they dont get a role
        roles.retain(|x| x != &role);

        if roles.is_empty() {
            inner.1 = SyncState::Deleted;
        }
    }
}

#[derive(Debug, Default)]
/// The result from [`canonicalise_command`]
pub struct CanonicalsedResult {
    /// The stack of all command names searched
    pub stack: Vec<(String, String)>,
    /// What the end call of the command was
    pub call: String,
    /// Why it was exited
    pub status: CanonicalisedStatus,
}

#[derive(Debug, PartialEq, Eq, Default)]
pub enum CanonicalisedStatus {
    /// The loop was terminated due to depth issues
    Overflow,
    /// The command was successfully found
    Success,
    /// The command was not found
    #[default]
    NotFound,
    /// The search entered a recursive loop
    Recursive,
    /// The registry failed to get a command to search
    FailedSubcommand,
}

#[derive(Debug)]
pub struct CommandEntry {
    id: Option<i64>,
    name: String,
    owner: UserId,
    ty: String,
    added: i64,
    registry: String,
    node: CmdNode,
}

impl CommandEntry {
    /// a helper to execute the inner command
    pub fn get_node(&mut self) -> &mut CmdNode {
        &mut self.node
    }

    /// Gets the owner of the command
    pub fn get_owner(&self) -> &UserId {
        &self.owner
    }

    /// Gets the type of the command
    pub fn get_type(&self) -> &str {
        &self.ty
    }

    /// Gets when the command was added, in unix time
    pub fn get_added(&self) -> i64 {
        self.added
    }

    /// Gets the registry the command is in
    pub fn get_registry(&self) -> &str {
        &self.registry
    }

    ///Gets the name of the command
    pub fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug)]
/// The actual type of the command node
pub enum CmdNode {
    /// An actual command that can be executed
    Cmd(Box<dyn ExtendedCommand + Send + Sync>),
    /// A seperate subregistry
    Subregistry(String),
    /// A symlink to anothe command
    Symlink { reg: String, name: String },
}

impl CmdNode {
    pub const SUBREG_ID: &'static str = "Subregistry";
    pub const SYMLINK_ID: &'static str = "Symlink";

    pub fn save(&self) -> Option<Vec<u8>> {
        match self {
            Self::Cmd(cmd) => cmd.save(),
            Self::Subregistry(reg) => Some(reg.clone().into_bytes()),
            Self::Symlink { reg, name } => {
                Some(rmp_serde::to_vec(&(reg, name)).unwrap_or_default())
            }
        }
    }

    pub fn load(ty: &str, data: &[u8], cli: &Cli) -> FlopResult<Self> {
        Ok(match ty {
            Self::SUBREG_ID => Self::Subregistry(String::from_utf8(data.to_vec())?),
            Self::SYMLINK_ID => {
                let data: SymlinkData = rmp_serde::from_slice(data)?;
                Self::Symlink {
                    reg: data.registry,
                    name: data.name,
                }
            }
            _ => Self::Cmd(command::construct(ty, data, cli)?),
        })
    }
}

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct SymlinkData {
    registry: String,
    name: String,
}

impl From<Box<dyn ExtendedCommand + Send + Sync>> for CmdNode {
    fn from(value: Box<dyn ExtendedCommand + Send + Sync>) -> Self {
        Self::Cmd(value)
    }
}

#[derive(Debug, FromRow)]
pub struct RegistryRow {
    id: i64,
    name: String,
    #[sqlx(rename = "super")]
    parent: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[non_exhaustive]
pub enum FlopRole {
    /// The user effectivly can do anything
    Admin,
    /// The user can moderate a registry
    /// i.e. (delete commands, change command owners)
    RegMod(String),
    /// The user can add a command to a registry
    RegAdd(String),
    /// The user can moderate all the registries
    GlobalMod,
    /// The user is banned from using commands or adding to public registries
    Banned,
}

impl FlopRole {
    pub fn get_parent(&self) -> Option<FlopRole> {
        match self {
            Self::Admin => None,
            Self::RegMod(_) => Some(Self::GlobalMod),
            Self::RegAdd(s) => Some(Self::RegAdd(s.clone())),
            Self::GlobalMod => Some(Self::Admin),
            Self::Banned => None,
        }
    }

    pub fn from_str(txt: &str) -> Option<FlopRole> {
        let ilovetheborrowchecker = txt.to_lowercase();
        let lower = &ilovetheborrowchecker;
        if lower == "admin" {
            Some(Self::Admin)
        } else if lower == "globalmod" {
            Some(Self::GlobalMod)
        } else if lower == "banned" {
            Some(Self::Banned)
        } else {
            let Some((name, arg)) = lower
                .split_once('(')
                .map(|(a, x)| (a, x.trim_end_matches(')')))
            else {
                return None;
            };

            if name == "regmod" {
                Some(Self::RegMod(arg.to_string()))
            } else if name == "regadd" {
                Some(Self::RegAdd(arg.to_string()))
            } else {
                None
            }
        }
    }
}

/// An enum to represent the state somthing is regarding being synced
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum SyncState {
    /// Changed
    Dirty,
    /// New item
    #[default]
    New,
    /// No change
    Clean,
    /// To be deleted
    Deleted,
}
