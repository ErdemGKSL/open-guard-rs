use dashmap::DashMap;
use poise::serenity_prelude as serenity;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;

pub enum CachedObject {
    Channel(serenity::GuildChannel),
    Role(serenity::Role),
}

pub struct ObjectCacheService {
    // Map: (GuildID, ObjectID) -> (Object, Timestamp)
    cache: Arc<DashMap<(u64, u64), (CachedObject, Instant)>>,
}

impl ObjectCacheService {
    pub fn new() -> Self {
        let cache = Arc::new(DashMap::new());
        let cleaner_cache = Arc::clone(&cache);

        // Spawn cleanup task
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(30)).await;
                let now = Instant::now();
                cleaner_cache.retain(|_, (_, time)| {
                    now.duration_since(*time) < Duration::from_secs(90)
                });
            }
        });

        Self { cache }
    }

    pub fn store_channel(&self, guild_id: serenity::GuildId, channel: serenity::GuildChannel) {
        self.cache.insert(
            (guild_id.get(), channel.id.get()),
            (CachedObject::Channel(channel), Instant::now()),
        );
    }

    pub fn store_role(&self, guild_id: serenity::GuildId, role: serenity::Role) {
        self.cache.insert(
            (guild_id.get(), role.id.get()),
            (CachedObject::Role(role), Instant::now()),
        );
    }

    pub fn take_channel(&self, guild_id: serenity::GuildId, channel_id: serenity::ChannelId) -> Option<serenity::GuildChannel> {
        self.cache.remove(&(guild_id.get(), channel_id.get())).and_then(|(_, (obj, _))| {
            if let CachedObject::Channel(c) = obj {
                Some(c)
            } else {
                None
            }
        })
    }

    pub fn take_role(&self, guild_id: serenity::GuildId, role_id: serenity::RoleId) -> Option<serenity::Role> {
        self.cache.remove(&(guild_id.get(), role_id.get())).and_then(|(_, (obj, _))| {
            if let CachedObject::Role(r) = obj {
                Some(r)
            } else {
                None
            }
        })
    }
}
