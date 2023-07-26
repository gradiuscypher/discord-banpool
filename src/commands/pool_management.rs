use super::super::db::mongo;
use crate::{Context, Error};

#[poise::command(slash_command, subcommands("add", "remove"))]
pub async fn pool(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command)]
pub async fn add(ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command)]
pub async fn remove(ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}
