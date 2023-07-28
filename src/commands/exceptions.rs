use super::super::db::mongo::DB;
use crate::{Context, Error};
use log::{error, info};
use serenity::utils::Color;

#[poise::command(slash_command, subcommands("add", "list", "remove"))]
pub async fn exception(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Create a new exception
#[poise::command(slash_command)]
pub async fn add(
    ctx: Context<'_>,
    #[description = "Target User ID"] user_id: String,
) -> Result<(), Error> {
    let db = DB::init().await.unwrap();
    let guild_id = ctx.guild_id().unwrap().to_string();
    match db
        .add_exception(&user_id, &guild_id, &ctx.author().id.to_string())
        .await
    {
        Ok(_) => {
            info!("Added {} as exception for {}", user_id, guild_id);
            ctx.send(|r| {
                r.embed(|r| {
                    r.title("Create Exception")
                        .color(Color::DARK_GREEN)
                        .description(format!(
                            "Exception for `{}` was created successfully",
                            user_id
                        ))
                })
            })
            .await?;
            Ok(())
        }
        Err(e) => {
            error!("Failed to create exception for {user_id}: {e}");
            ctx.send(|r| {
                r.embed(|r| {
                    r.title("Create Exception")
                        .color(Color::RED)
                        .description(format!(
                            "Failed to create exception for `{}`\n\n{}",
                            user_id, e
                        ))
                })
            })
            .await?;
            Err(e.into())
        }
    }
}

/// List the existing exceptions for this guild
#[poise::command(slash_command)]
pub async fn list(ctx: Context<'_>) -> Result<(), Error> {
    let db = DB::init().await.unwrap();
    let guild_id = ctx.guild_id().unwrap().to_string();
    match db.list_exceptions(&guild_id).await {
        Ok(exceptions) => {
            info!("Listed Exceptions on guild {}", guild_id);
            let mut exception_string = String::new();

            for exception in exceptions {
                exception_string.push_str(&exception.user_id);
            }

            ctx.send(|r| {
                r.embed(|r| {
                    r.title("List Exceptions")
                        .color(Color::DARK_GREEN)
                        .description(format!("User exceptions:\n {exception_string}"))
                })
            })
            .await?;
            Ok(())
        }
        Err(e) => {
            ctx.send(|r| {
                r.embed(|r| {
                    r.title("List Exceptions")
                        .color(Color::RED)
                        .description(format!("Failed to list exceptions: \n{}", e))
                })
            })
            .await?;
            error!("Error while listing exceptions: {}", e);
            Err(e.into())
        }
    }
}

/// Remove an exception by User ID
#[poise::command(slash_command)]
pub async fn remove(
    ctx: Context<'_>,
    #[description = "Target User ID"] user_id: String,
) -> Result<(), Error> {
    let db = DB::init().await.unwrap();
    let guild_id = ctx.guild_id().unwrap().to_string();
    match db.delete_exception(&user_id, &guild_id).await {
        Ok(_) => {
            info!("Removed {} as exception from {}", user_id, guild_id);
            ctx.send(|r| {
                r.embed(|r| {
                    r.title("Remove Exception")
                        .color(Color::DARK_GREEN)
                        .description(format!(
                            "Exception {user_id} was removed from {guild_id} as an exception"
                        ))
                })
            })
            .await?;
            Ok(())
        }
        Err(e) => {
            info!(
                "Failed to remove {} as exception from {}",
                user_id, guild_id
            );
            ctx.send(|r| {
                r.embed(|r| {
                    r.title("Remove Exception")
                        .color(Color::RED)
                        .description(format!(
                            "Failed to remove exception for {user_id} from {guild_id}"
                        ))
                })
            })
            .await?;
            Err(e.into())
        }
    }
}
