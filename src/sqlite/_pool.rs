use crate::resource::{Resource, Runnable, SyncPool};

pub use rusqlite;
use rusqlite::{Connection, Error, OpenFlags};
use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Debug)]
enum Source {
    File(PathBuf),
    Memory,
}

type InitFn = dyn Fn(&mut Connection) -> Result<(), rusqlite::Error> + Send + Sync + 'static;

/// An `r2d2::ManageConnection` for `rusqlite::Connection`s.
pub struct SqliteConnectionConfig {
    source: Source,
    flags: OpenFlags,
    init: Option<Box<InitFn>>,
}

impl fmt::Debug for SqliteConnectionConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut builder = f.debug_struct("SqliteConnectionConfig");
        let _ = builder.field("source", &self.source);
        let _ = builder.field("flags", &self.source);
        let _ = builder.field("init", &self.init.as_ref().map(|_| "InitFn"));
        builder.finish()
    }
}

impl SqliteConnectionConfig {
    /// Creates a new `SqliteConnectionManager` from file.
    ///
    /// See `rusqlite::Connection::open`
    pub fn file<P: AsRef<Path>>(path: P) -> Self {
        Self {
            source: Source::File(path.as_ref().to_path_buf()),
            flags: OpenFlags::default(),
            init: None,
        }
    }

    /// Creates a new `SqliteConnectionManager` from memory.
    pub fn memory() -> Self {
        Self {
            source: Source::Memory,
            flags: OpenFlags::default(),
            init: None,
        }
    }

    /// Converts `SqliteConnectionManager` into one that sets OpenFlags upon
    /// connection creation.
    ///
    /// See `rustqlite::OpenFlags` for a list of available flags.
    pub fn with_flags(self, flags: OpenFlags) -> Self {
        Self { flags, ..self }
    }

    /// Converts `SqliteConnectionManager` into one that calls an initialization
    /// function upon connection creation. Could be used to set PRAGMAs, for
    /// example.
    ///
    /// ### Example
    ///
    /// Make a `SqliteConnectionManager` that sets the `foreign_keys` pragma to
    /// true for every connection.
    ///
    /// ```rust,no_run
    /// # use r2d2_sqlite::{SqliteConnectionManager};
    /// let manager = SqliteConnectionManager::file("app.db")
    ///     .with_init(|c| c.execute_batch("PRAGMA foreign_keys=1;"));
    /// ```
    pub fn with_init<F>(self, init: F) -> Self
    where
        F: Fn(&mut Connection) -> Result<(), rusqlite::Error> + Send + Sync + 'static,
    {
        let init: Option<Box<InitFn>> = Some(Box::new(init));
        Self { init, ..self }
    }
}

pub struct SqliteConnection;

impl Resource for SqliteConnection {
    type Config = SqliteConnectionConfig;
    type Handle = Connection;
    type Item = Connection;
    type Error = rusqlite::Error;
    type ApplyError = rusqlite::Error;

    fn create(config: &Self::Config) -> Result<Connection, Error> {
        match config.source {
            Source::File(ref path) => Connection::open_with_flags(path, config.flags),
            Source::Memory => Connection::open_in_memory_with_flags(config.flags),
        }
        .map_err(Into::into)
        .and_then(|mut c| match config.init {
            None => Ok(c),
            Some(ref init) => init(&mut c).map(|_| c),
        })
    }

    fn apply(
        conn: Connection,
        f: impl FnOnce(Connection) -> Result<Connection, Error>,
    ) -> Result<Connection, Error> {
        f(conn)
    }

    fn validate(conn: Connection) -> Result<Connection, Error> {
        conn.execute_batch("")?;
        Ok(conn)
    }
}