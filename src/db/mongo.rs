// ref: https://github.com/mongodb/mongo-rust-driver#example-usage
// ref: https://www.mongodb.com/developer/quickstart/rust-crud-tutorial/
// ref: https://github.com/zupzup/rust-web-mongodb-example/blob/main/src/db.rs
// ref: https://blog.logrocket.com/using-mongodb-in-a-rust-web-service/

use anyhow::{anyhow, Result};
use chrono::Utc;
use log::info;
use mongodb::bson::doc;
use mongodb::{options::ClientOptions, Client};
use serde::{Deserialize, Serialize};
use serenity::futures::TryStreamExt;
use std::env;

#[derive(Clone, Debug)]
pub struct DB {
    pub client: Client,
    pub db_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BanPool {
    pub pool_name: String,
    pub pool_desc: String,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub timestamp: chrono::DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Ban {
    pub user_id: String,
    pub pool_name: String,
    pub reason: String,
    pub creator_id: String,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub timestamp: chrono::DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BanException {
    pub user_id: String,
    pub server_id: String,
    pub creator_id: String,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub timestamp: chrono::DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Subscription {
    pub pool_name: String,
    pub subscription_level: String,
    pub server_id: String,
    pub creator_id: String,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub timestamp: chrono::DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerConfig {
    pub server_id: String,
    pub announce_channel_id: String,
    pub admin_role_id: String,
    pub author_id: String,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub timestamp: chrono::DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AdminRole {
    pub server_id: String,
    pub role_id: String,
    pub author_id: String,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub timestamp: chrono::DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NotificationChannel {
    pub server_id: String,
    pub channel_id: String,
    pub author_id: String,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub timestamp: chrono::DateTime<Utc>,
}

impl DB {
    pub async fn init() -> Result<Self> {
        let mongo_uri = env::var("MONGODB_URI").expect("MONGODB_URI isn't set!");
        let db_name = env::var("MONGODB_DB").expect("MONGODB_DB isn't set!");
        let client_options = ClientOptions::parse(mongo_uri).await?;

        Ok(Self {
            client: Client::with_options(client_options)?,
            db_name: db_name,
        })
    }

    pub async fn add_pool(&self, pool_name: &str, pool_desc: &str) -> Result<()> {
        // get the banpools collection
        let banpools = self
            .client
            .database(&self.db_name)
            .collection::<BanPool>("banpools");
        // check to see if the pool exists
        let pool = banpools
            .find_one(doc! {"pool_name": pool_name}, None)
            .await
            .unwrap();

        // if it doesnt exist already, create it
        match pool {
            Some(pool) => Err(anyhow!("Pool already exists.")),
            None => {
                info!("Creating {} with the description {}", pool_name, pool_desc);
                let new_pool = BanPool {
                    pool_name: pool_name.to_string(),
                    pool_desc: pool_desc.to_string(),
                    timestamp: Utc::now(),
                };
                let insert_result = banpools.insert_one(new_pool, None).await.unwrap();
                Ok(())
            }
        }
    }

    pub async fn delete_pool(&self, pool_name: &str) -> Result<()> {
        // get the banpools collection
        let banpools = self
            .client
            .database(&self.db_name)
            .collection::<BanPool>("banpools");
        // check to see if the pool exists
        let pool = banpools
            .delete_one(doc! {"pool_name": pool_name}, None)
            .await
            .unwrap();

        // if we deleted a pool, return ok, otherwise return an error
        if pool.deleted_count > 0 {
            Ok(())
        } else {
            Err(anyhow!(
                "Unable to delete {}, pool does not exist.",
                pool_name
            ))
        }
    }

    pub async fn list_pools(&self) -> Result<Vec<BanPool>> {
        // get the banpools collection
        let banpools = self
            .client
            .database(&self.db_name)
            .collection::<BanPool>("banpools");

        let pool_query = banpools.find(None, None).await.unwrap();

        let pools: Vec<BanPool> = pool_query.try_collect().await.unwrap();

        Ok(pools)
    }

    pub async fn add_ban(
        &self,
        user_id: &str,
        pool_name: &str,
        reason: &str,
        author_id: &str,
    ) -> Result<()> {
        // get the banpools collection
        let bans = self
            .client
            .database(&self.db_name)
            .collection::<Ban>("bans");
        let banpools = self
            .client
            .database(&self.db_name)
            .collection::<BanPool>("banpools");

        let ban = bans
            .find_one(doc! {"user_id": user_id, "pool_name": pool_name}, None)
            .await
            .unwrap();

        match ban {
            Some(ban) => Err(anyhow!("Ban already exists")),
            None => {
                let target_pool = banpools
                    .find_one(doc! {"pool_name": pool_name}, None)
                    .await
                    .unwrap();

                match target_pool {
                    Some(pool) => {
                        let new_ban = Ban {
                            user_id: user_id.to_string(),
                            pool_name: pool_name.to_string(),
                            creator_id: author_id.to_string(),
                            reason: reason.to_string(),
                            timestamp: Utc::now(),
                        };
                        bans.insert_one(new_ban, None).await.unwrap();
                        Ok(())
                    }
                    None => Err(anyhow!("This pool does not exist")),
                }
            }
        }
    }

    pub async fn delete_ban(&self, user_id: &str, pool_name: &str) -> Result<()> {
        // get the banpools collection
        let bans = self
            .client
            .database(&self.db_name)
            .collection::<Ban>("bans");

        let ban = bans
            .delete_one(doc! {"pool_name": pool_name, "user_id": user_id}, None)
            .await
            .unwrap();

        // if we deleted a pool, return ok, otherwise return an error
        if ban.deleted_count > 0 {
            Ok(())
        } else {
            Err(anyhow!(
                "Unable to delete {} from {}, ban does not exist.",
                user_id,
                pool_name
            ))
        }
    }

    pub async fn list_bans(&self) -> Result<Vec<Ban>> {
        let bans = self
            .client
            .database(&self.db_name)
            .collection::<Ban>("bans");

        let ban_query = bans.find(None, None).await.unwrap();

        let bans: Vec<Ban> = ban_query.try_collect().await.unwrap();

        Ok(bans)
    }

    pub async fn get_ban_from_pool(&self, user_id: &str, pool_name: &str) -> Result<Vec<Ban>> {
        let bans = self
            .client
            .database(&self.db_name)
            .collection::<Ban>("bans");
        let ban_query = bans
            .find(doc! {"user_id": user_id, "pool_name": pool_name}, None)
            .await
            .unwrap();

        let bans: Vec<Ban> = ban_query.try_collect().await.unwrap();

        Ok(bans)
    }

    pub async fn get_user_bans(&self, user_id: &str) -> Result<Vec<Ban>> {
        // TODO: combine get_user_bans and get_pool_bans
        let bans = self
            .client
            .database(&self.db_name)
            .collection::<Ban>("bans");

        let ban_query = bans.find(doc! {"user_id": user_id}, None).await.unwrap();

        let bans: Vec<Ban> = ban_query.try_collect().await.unwrap();

        Ok(bans)
    }

    pub async fn get_pool_bans(&self, pool_name: &str) -> Result<Vec<Ban>> {
        // TODO: combine get_user_bans and get_pool_bans
        let bans = self
            .client
            .database(&self.db_name)
            .collection::<Ban>("bans");

        let ban_query = bans
            .find(doc! {"pool_name": pool_name}, None)
            .await
            .unwrap();

        let bans: Vec<Ban> = ban_query.try_collect().await.unwrap();

        Ok(bans)
    }

    pub async fn add_exception(
        &self,
        user_id: &str,
        server_id: &str,
        creator_id: &str,
    ) -> Result<()> {
        let exceptions = self
            .client
            .database(&self.db_name)
            .collection::<BanException>("exceptions");

        let exception_query = exceptions
            .find_one(doc! {"user_id": user_id, "server_id": server_id}, None)
            .await
            .unwrap();

        match exception_query {
            Some(_) => Err(anyhow!("Ban exception already exists.")),
            None => {
                let new_exception = BanException {
                    user_id: user_id.to_string(),
                    server_id: server_id.to_string(),
                    creator_id: creator_id.to_string(),
                    timestamp: Utc::now(),
                };
                exceptions.insert_one(new_exception, None).await.unwrap();
                Ok(())
            }
        }
    }

    pub async fn delete_exception(&self, user_id: &str, server_id: &str) -> Result<()> {
        // get the banpools collection
        let exceptions = self
            .client
            .database(&self.db_name)
            .collection::<BanException>("exceptions");

        let exception = exceptions
            .delete_one(doc! {"server_id": server_id, "user_id": user_id}, None)
            .await
            .unwrap();

        // if we deleted a pool, return ok, otherwise return an error
        if exception.deleted_count > 0 {
            Ok(())
        } else {
            Err(anyhow!(
                "Unable to delete {} from {}, exception does not exist.",
                user_id,
                server_id
            ))
        }
    }

    pub async fn list_exceptions(&self, server_id: &str) -> Result<Vec<BanException>> {
        // get the banpools collection
        let exceptions = self
            .client
            .database(&self.db_name)
            .collection::<BanException>("exceptions");

        let exception_query = exceptions
            .find(doc! {"server_id": server_id}, None)
            .await
            .unwrap();

        let exceptions: Vec<BanException> = exception_query.try_collect().await.unwrap();

        Ok(exceptions)
    }

    pub async fn is_user_exception(&self, server_id: &str, user_id: &str) -> bool {
        let exceptions = self
            .client
            .database(&self.db_name)
            .collection::<BanException>("exceptions");

        let exception_query = exceptions
            .find_one(doc! {"server_id": server_id, "user_id": user_id}, None)
            .await
            .unwrap();

        match exception_query {
            Some(_) => true,
            None => false,
        }
    }

    pub async fn add_subscription(
        &self,
        pool_name: &str,
        server_id: &str,
        creator_id: &str,
        subscription_level: &str,
    ) -> Result<()> {
        let subscriptions = self
            .client
            .database(&self.db_name)
            .collection::<Subscription>("subscriptions");

        let banpools = self
            .client
            .database(&self.db_name)
            .collection::<BanPool>("banpools");

        let subscription_query = subscriptions
            .find_one(doc! {"pool_name": pool_name, "server_id": server_id}, None)
            .await
            .unwrap();

        match subscription_query {
            Some(_) => Err(anyhow!("Subscription already exists.")),
            None => {
                let target_pool = banpools
                    .find_one(doc! {"pool_name": pool_name}, None)
                    .await
                    .unwrap();

                match target_pool {
                    Some(_) => {
                        let new_subscription = Subscription {
                            pool_name: pool_name.to_string(),
                            server_id: server_id.to_string(),
                            subscription_level: subscription_level.to_string(),
                            creator_id: creator_id.to_string(),
                            timestamp: Utc::now(),
                        };
                        subscriptions
                            .insert_one(new_subscription, None)
                            .await
                            .unwrap();
                        Ok(())
                    }
                    None => Err(anyhow!(
                        "Error adding subscription: This pool does not exist: {pool_name}"
                    )),
                }
            }
        }
    }

    pub async fn delete_subscription(&self, pool_name: &str, server_id: &str) -> Result<()> {
        // get the banpools collection
        let subscriptions = self
            .client
            .database(&self.db_name)
            .collection::<Subscription>("subscriptions");

        let subscription = subscriptions
            .delete_one(doc! {"server_id": server_id, "pool_name": pool_name}, None)
            .await
            .unwrap();

        // if we deleted a pool, return ok, otherwise return an error
        if subscription.deleted_count > 0 {
            Ok(())
        } else {
            Err(anyhow!(
                "Unable to unsubscribe {} from {}, subscription does not exist.",
                pool_name,
                server_id
            ))
        }
    }

    pub async fn list_subscriptions(&self, server_id: &str) -> Result<Vec<Subscription>> {
        let subscriptions = self
            .client
            .database(&self.db_name)
            .collection::<Subscription>("subscriptions");

        let subscription_query = subscriptions
            .find(doc! {"server_id": server_id}, None)
            .await
            .unwrap();

        let sub_list: Vec<Subscription> = subscription_query.try_collect().await.unwrap();

        Ok(sub_list)
    }

    pub async fn list_subscribed_servers(&self, pool_name: &str) -> Result<Vec<Subscription>> {
        let subscriptions = self
            .client
            .database(&self.db_name)
            .collection::<Subscription>("subscriptions");

        let subscription_query = subscriptions
            .find(doc! {"pool_name": pool_name}, None)
            .await
            .unwrap();

        let sub_list: Vec<Subscription> = subscription_query.try_collect().await.unwrap();

        Ok(sub_list)
    }

    pub async fn add_notification_channel(
        &self,
        server_id: &str,
        channel_id: &str,
        author_id: &str,
    ) -> Result<()> {
        let notifications = self
            .client
            .database(&self.db_name)
            .collection::<NotificationChannel>("notifications");

        let notification = notifications
            .find_one(doc! {"server_id": server_id}, None)
            .await
            .unwrap();

        match notification {
            Some(_) => Err(anyhow!("Notification channel already set")),
            None => {
                info!(
                    "Setting {} as notification Channel for {}",
                    channel_id, server_id
                );
                let new_notification = NotificationChannel {
                    server_id: server_id.to_string(),
                    channel_id: channel_id.to_string(),
                    author_id: author_id.to_string(),
                    timestamp: Utc::now(),
                };
                let insert_result = notifications
                    .insert_one(new_notification, None)
                    .await
                    .unwrap();
                Ok(())
            }
        }
    }

    pub async fn delete_notification_channel(&self, server_id: &str) -> Result<()> {
        // get the banpools collection
        let notifications = self
            .client
            .database(&self.db_name)
            .collection::<NotificationChannel>("notifications");

        let notification = notifications
            .delete_one(doc! {"server_id": server_id}, None)
            .await
            .unwrap();

        // if we deleted a pool, return ok, otherwise return an error
        if notification.deleted_count > 0 {
            Ok(())
        } else {
            Err(anyhow!(
                "Unable to remove notification channel. No channel is set",
            ))
        }
    }

    pub async fn list_notification_channel(&self, server_id: &str) -> Result<NotificationChannel> {
        let notifications = self
            .client
            .database(&self.db_name)
            .collection::<NotificationChannel>("notifications");

        let notification_channel = notifications
            .find_one(doc! {"server_id": server_id}, None)
            .await
            .unwrap();

        match notification_channel {
            Some(channel) => Ok(channel),
            None => Err(anyhow!("No notification channel has been set")),
        }
    }

    pub async fn add_admin_role(
        &self,
        server_id: &str,
        role_id: &str,
        author_id: &str,
    ) -> Result<()> {
        let admin_roles = self
            .client
            .database(&self.db_name)
            .collection::<AdminRole>("adminroles");

        let admin_role = admin_roles
            .find_one(doc! {"server_id": server_id}, None)
            .await
            .unwrap();

        match admin_role {
            Some(_) => Err(anyhow!("Admin role already set")),
            None => {
                info!("Setting {} as Admin role for {}", role_id, server_id);
                let new_role = AdminRole {
                    server_id: server_id.to_string(),
                    role_id: role_id.to_string(),
                    author_id: author_id.to_string(),
                    timestamp: Utc::now(),
                };
                let insert_result = admin_roles.insert_one(new_role, None).await.unwrap();
                Ok(())
            }
        }
    }

    pub async fn delete_admin_role(&self, server_id: &str) -> Result<()> {
        let admin_roles = self
            .client
            .database(&self.db_name)
            .collection::<AdminRole>("adminroles");

        let admin_role = admin_roles
            .delete_one(doc! {"server_id": server_id}, None)
            .await
            .unwrap();

        // if we deleted a pool, return ok, otherwise return an error
        if admin_role.deleted_count > 0 {
            Ok(())
        } else {
            Err(anyhow!("Unable to remove admin role. No role is set",))
        }
    }

    pub async fn list_admin_role(&self, server_id: &str) -> Result<AdminRole> {
        let admin_roles = self
            .client
            .database(&self.db_name)
            .collection::<AdminRole>("adminroles");

        let admin_role = admin_roles
            .find_one(doc! {"server_id": server_id}, None)
            .await
            .unwrap();

        match admin_role {
            Some(role) => Ok(role),
            None => Err(anyhow!("No admin role has been set")),
        }
    }
}
