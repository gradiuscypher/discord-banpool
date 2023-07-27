use super::super::db::mongo::DB;
use crate::{Context, Error};
use futures::{Stream, StreamExt};
use log::{error, info};
use serenity::utils::Color;

async fn autocomplete_pools<'a>(
    _: Context<'_>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    let db = DB::init().await.unwrap();
    let pools = db.list_pools().await.unwrap_or(vec![]);

    futures::stream::iter(pools)
        .filter(move |pool| futures::future::ready(pool.pool_name.starts_with(partial)))
        .map(|pool| pool.pool_name)
}

#[poise::command(slash_command, subcommands("add", "get", "remove"))]
pub async fn ban(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Add a new ban to a pool
#[poise::command(slash_command)]
pub async fn add(
    ctx: Context<'_>,
    #[description = "Target User ID"] user_id: String,
    #[description = "Banpool Name"]
    #[autocomplete = "autocomplete_pools"]
    pool: String,
    #[description = "Ban Reason"] reason: String,
) -> Result<(), Error> {
    let db = DB::init().await.unwrap();
    let author_id = ctx.author().id;
    match db
        .add_ban(&user_id, &pool, &reason, &author_id.to_string())
        .await
    {
        Ok(_) => {
            info!("Added pool: {}", user_id);
            ctx.send(|r| {
                r.embed(|r| {
                    r.title("Create Ban")
                        .color(Color::DARK_GREEN)
                        .description(format!("{user_id} was added to {pool} successfully"))
                })
            })
            .await?;
            Ok(())
        }
        Err(e) => {
            error!("Failed to add {} to {} banpool", user_id, pool);
            ctx.send(|r| {
                r.embed(|r| {
                    r.title("Create Ban")
                        .color(Color::RED)
                        .description(format!("Failed to add {user_id} to {pool}:\n{e}"))
                })
            })
            .await?;
            Err(e.into())
        }
    }
}

#[poise::command(slash_command)]
pub async fn get(
    ctx: Context<'_>,
    #[description = "Target User ID"] user_id: String,
) -> Result<(), Error> {
    let db = DB::init().await.unwrap();
    match db.get_user_bans(&user_id).await {
        Ok(bans) => {
            let mut ban_string = String::new();

            for ban in bans {
                ban_string.push_str(format!("{}\n", ban.pool_name).as_str());
            }

            info!("Added pool: {}", user_id);
            ctx.send(|r| {
                r.embed(|r| {
                    r.title("Get Bans")
                        .color(Color::DARK_GREEN)
                        .description(format!("User is in these pools:\n {ban_string}"))
                })
            })
            .await?;
            Ok(())
        }
        Err(e) => {
            error!("Unable to fetch bans for user ID {}: {}", user_id, e);
            ctx.send(|r| {
                r.embed(|r| {
                    r.title("Get Bans")
                        .color(Color::RED)
                        .description(format!("Unable to fetch bans for User ID {user_id}\n{e}"))
                })
            })
            .await?;
            Err(e.into())
        }
    }
}

/// Remove a target banpool by name
#[poise::command(slash_command)]
pub async fn remove(
    ctx: Context<'_>,
    #[description = "Target User ID"] user_id: String,
    #[description = "Banpool Name"]
    #[autocomplete = "autocomplete_pools"]
    pool: String,
) -> Result<(), Error> {
    let db = DB::init().await.unwrap();
    match db.delete_ban(&user_id, &pool).await {
        Ok(_) => {
            info!("Added pool: {}", user_id);
            ctx.send(|r| {
                r.embed(|r| {
                    r.title("Remove Ban")
                        .color(Color::DARK_GREEN)
                        .description(format!("{user_id} was removed from {pool} successfully"))
                })
            })
            .await?;
            Ok(())
        }
        Err(e) => {
            error!("Failed to add {} to {} banpool", user_id, pool);
            ctx.send(|r| {
                r.embed(|r| {
                    r.title("Remove Ban")
                        .color(Color::RED)
                        .description(format!("Failed to remove {user_id} from {pool}:\n{e}"))
                })
            })
            .await?;
            Err(e.into())
        }
    }
}
