use crate::db::entities::module_configs;
use crate::services::localization::ContextL10nExt;
use crate::{Context, Error};
use chrono::Utc;
use poise::serenity_prelude as serenity;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

/// Status command
#[poise::command(
    slash_command,
    install_context = "Guild",
    interaction_context = "Guild|BotDm|PrivateChannel",
    ephemeral
)]
pub async fn status(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;
    let end_time = Utc::now();

    let l10n = ctx.l10n_user();

    let guild_id = ctx.guild_id().map(|id| id.get() as i64);

    // Get enabled modules
    let modules_status = if let Some(gid) = guild_id {
        let db_modules = module_configs::Entity::find()
            .filter(module_configs::Column::GuildId.eq(gid))
            .all(&ctx.data().db)
            .await?;

        ctx.data()
            .module_definitions
            .iter()
            .map(|def| {
                let is_enabled = db_modules
                    .iter()
                    .find(|m| m.module_type.to_string() == def.id)
                    .map(|m| m.enabled)
                    .unwrap_or(false);
                let status_emoji = if is_enabled { "✅" } else { "❌" };
                let name = l10n.t(def.name_key, None);
                format!("{} **{}**", status_emoji, name)
            })
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        l10n.t("status-no-guild", None)
    };

    // Latency
    let interaction_created_at = ctx.created_at().to_utc();
    let total_latency = end_time - interaction_created_at;

    // Shard info
    let shard_id = ctx.serenity_context().shard_id;
    let shard_count = ctx.framework().serenity_context.runners.len();

    let mut embed = serenity::CreateEmbed::default()
        .title(l10n.t("status-title", None))
        .field(l10n.t("status-modules", None), modules_status, false)
        .field(
            l10n.t("status-latency", None),
            format!("{} ms", total_latency.num_milliseconds()),
            true,
        )
        .field(
            l10n.t("status-shard", None),
            format!("{} / {}", shard_id, shard_count),
            true,
        );

    #[cfg(feature = "system-info")]
    {
        use sysinfo::System;
        let mut sys = System::new_all();
        sys.refresh_all();

        let cpu_usage = sys.global_cpu_usage();
        let total_memory = sys.total_memory();
        let used_memory = sys.used_memory();
        let memory_usage_percent = (used_memory as f32 / total_memory as f32) * 100.0;

        // Convert to GB
        let total_gb = total_memory as f32 / 1024.0 / 1024.0 / 1024.0;
        let used_gb = used_memory as f32 / 1024.0 / 1024.0 / 1024.0;

        embed = embed.field(
            l10n.t("status-system", None),
            format!(
                "CPU: {:.2}%\nMem: {:.2}% ({:.2} / {:.2} GB)",
                cpu_usage, memory_usage_percent, used_gb, total_gb
            ),
            false,
        );
    }

    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}
