use super::super::db::mongo::DB;
use crate::{Context, Error};
use futures::{Stream, StreamExt};
use log::{error, info};
use serenity::utils::Color;

#[poise::command(slash_command, subcommands("add", "remove", "list"))]
pub async fn pool(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

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

/// Create a new banpool
#[poise::command(slash_command)]
pub async fn add(
    ctx: Context<'_>,
    #[description = "Banpool Name"] name: String,
    #[description = "Banpool Description"] description: String,
) -> Result<(), Error> {
    let db = DB::init().await.unwrap();
    match db.add_pool(&name, &description).await {
        Ok(_) => {
            info!("Added pool: {}", name);
            ctx.send(|r| {
                r.embed(|r| {
                    r.title("Create Banpool")
                        .color(Color::DARK_GREEN)
                        .description(format!("Banpool `{}` was created successfully", name))
                })
            })
            .await?;
            Ok(())
        }
        Err(e) => {
            error!("Failed to create banpool: {name}: {description}: {e}");
            ctx.send(|r| {
                r.embed(|r| {
                    r.title("Create Banpool")
                        .color(Color::RED)
                        .description(format!("Banpool `{}` failed to create.\n\n{}", name, e))
                })
            })
            .await?;
            Err(e.into())
        }
    }
}

/// List the existing banpools
#[poise::command(slash_command)]
pub async fn list(ctx: Context<'_>) -> Result<(), Error> {
    let db = DB::init().await.unwrap();
    match db.list_pools().await {
        Ok(pools) => {
            info!("Listed pools");
            // TODO: add Discord feedback
            let mut pool_fields: Vec<_> = Vec::new();

            for pool in pools {
                pool_fields.push((pool.pool_name, pool.pool_desc, false));
            }
            ctx.send(|r| {
                r.embed(|r| {
                    r.title("List Banpools")
                        .color(Color::DARK_GREEN)
                        .fields(pool_fields)
                })
            })
            .await?;
            Ok(())
        }
        Err(e) => {
            ctx.send(|r| {
                r.embed(|r| {
                    r.title("List Banpools")
                        .color(Color::RED)
                        .description(format!("Failed to list Banpools"))
                })
            })
            .await?;
            error!("Error while listing pools: {}", e);
            Err(e.into())
        }
    }
}

/// Remove a target banpool by name
#[poise::command(slash_command)]
pub async fn remove(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_pools"]
    #[description = "Banpool Name"]
    name: String,
) -> Result<(), Error> {
    let db = DB::init().await.unwrap();
    match db.delete_pool(&name).await {
        Ok(_) => {
            info!("Deleted pool: {}", name);
            ctx.send(|r| {
                r.embed(|r| {
                    r.title("Delete Banpool")
                        .color(Color::DARK_GREEN)
                        .description(format!("Banpool `{}` was deleted successfully", name))
                })
            })
            .await?;
            Ok(())
        }
        Err(e) => {
            error!("Failed to delete banpool: {name}: {e}");
            ctx.send(|r| {
                r.embed(|r| {
                    r.title("Delete Banpool")
                        .color(Color::RED)
                        .description(format!("Banpool `{}` failed to delete.\n\n{}", name, e))
                })
            })
            .await?;
            Err(e.into())
        }
    }
}
