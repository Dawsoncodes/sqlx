use std::io;
use std::time::Duration;

use anyhow::Result;
use futures::{Future, TryFutureExt};

use sqlx::{AnyConnection, Connection};

use crate::opt::{Command, ConnectOpts, DatabaseCommand, MigrateCommand};

mod database;
mod metadata;
// mod migration;
// mod migrator;
#[cfg(feature = "completions")]
mod completions;
mod config;
mod migrate;
mod opt;
mod prepare;

pub use crate::opt::Opt;
pub use config::SqlxConfig;

pub async fn run(opt: Opt) -> Result<()> {
    match opt.command {
        Command::Migrate(migrate) => match migrate.command {
            MigrateCommand::Add {
                source,
                description,
                reversible,
                sequential,
                timestamp,
            } => migrate::add(&source, &description, reversible, sequential, timestamp).await?,
            MigrateCommand::Run {
                source,
                dry_run,
                ignore_missing,
                mut connect_opts,
                target_version,
            } => {
                migrate::run(
                    &source,
                    &mut connect_opts,
                    dry_run,
                    *ignore_missing,
                    target_version,
                )
                .await?
            }
            MigrateCommand::Revert {
                source,
                dry_run,
                ignore_missing,
                mut connect_opts,
                target_version,
            } => {
                migrate::revert(
                    &source,
                    &mut connect_opts,
                    dry_run,
                    *ignore_missing,
                    target_version,
                )
                .await?
            }
            MigrateCommand::Info {
                source,
                mut connect_opts,
            } => migrate::info(&source, &mut connect_opts).await?,
            MigrateCommand::BuildScript { source, force } => migrate::build_script(&source, force)?,
        },

        Command::Database(database) => match database.command {
            DatabaseCommand::Create { mut connect_opts } => {
                database::create(&mut connect_opts).await?
            }
            DatabaseCommand::Drop {
                confirmation,
                mut connect_opts,
                force,
            } => database::drop(&mut connect_opts, !confirmation.yes, force).await?,
            DatabaseCommand::Reset {
                confirmation,
                source,
                mut connect_opts,
                force,
            } => database::reset(&source, &mut connect_opts, !confirmation.yes, force).await?,
            DatabaseCommand::Setup {
                source,
                mut connect_opts,
            } => database::setup(&source, &mut connect_opts).await?,
        },

        Command::Prepare {
            check,
            workspace,
            connect_opts,
            args,
        } => prepare::run(check, workspace, connect_opts, args).await?,

        #[cfg(feature = "completions")]
        Command::Completions { shell } => completions::run(shell),
    };

    Ok(())
}

/// Attempt to connect to the database server, retrying up to `ops.connect_timeout`.
async fn connect(opts: &mut ConnectOpts) -> anyhow::Result<AnyConnection> {
    retry_connect_errors(opts, AnyConnection::connect).await
}

/// Attempt an operation that may return errors like `ConnectionRefused`,
/// retrying up until `ops.connect_timeout`.
///
/// The closure is passed `&ops.database_url` for easy composition.
async fn retry_connect_errors<'a, F, Fut, T>(
    opts: &'a mut ConnectOpts,
    mut connect: F,
) -> anyhow::Result<T>
where
    F: FnMut(&'a str) -> Fut,
    Fut: Future<Output = sqlx::Result<T>> + 'a,
{
    sqlx::any::install_default_drivers();

    let opts_clone = opts.clone();

    let db_url = opts.required_db_url()?;

    backoff::future::retry(
        backoff::ExponentialBackoffBuilder::new()
            .with_max_elapsed_time(Some(Duration::from_secs(opts_clone.connect_timeout)))
            .build(),
        || {
            connect(&db_url).map_err(|e| -> backoff::Error<anyhow::Error> {
                match e {
                    sqlx::Error::Io(ref ioe) => match ioe.kind() {
                        io::ErrorKind::ConnectionRefused
                        | io::ErrorKind::ConnectionReset
                        | io::ErrorKind::ConnectionAborted => {
                            return backoff::Error::transient(e.into());
                        }
                        _ => (),
                    },
                    _ => (),
                }

                backoff::Error::permanent(e.into())
            })
        },
    )
    .await
}
