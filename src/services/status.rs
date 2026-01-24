use crate::db::entities::module_configs;
use crate::services::localization::{ContextL10nExt, L10nProxy};
use crate::{Context, Data, Error};
use chrono::Utc;
use poise::serenity_prelude as serenity;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::sync::atomic::Ordering;

/// Status command
#[poise::command(
    slash_command,
    install_context = "Guild",
    interaction_context = "Guild|BotDm|PrivateChannel",
    ephemeral
)]
pub async fn status(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let end_time = Utc::now();

    let l10n = ctx.l10n_user();

    let components = get_status_components(
        &*ctx.data(),
        &l10n,
        ctx.guild_id().map(|id| id.get() as i64),
        ctx.serenity_context().shard_id,
        ctx.created_at().to_utc(),
        end_time,
        ctx.data().shard_count.load(Ordering::Relaxed),
    )
    .await?;

    ctx.send(
        poise::CreateReply::default()
            .flags(serenity::MessageFlags::IS_COMPONENTS_V2 | serenity::MessageFlags::EPHEMERAL)
            .components(components),
    )
    .await?;

    Ok(())
}

pub async fn get_status_components(
    data: &Data,
    l10n: &L10nProxy,
    guild_id: Option<i64>,
    shard_id: serenity::ShardId,
    interaction_created_at: chrono::DateTime<Utc>,
    end_time: chrono::DateTime<Utc>,
    shard_count: u32,
) -> Result<Vec<serenity::CreateComponent<'static>>, Error> {
    let mut inner_components = vec![];

    // Title
    inner_components.push(serenity::CreateContainerComponent::TextDisplay(
        serenity::CreateTextDisplay::new(format!("## {}", l10n.t("status-title", None))),
    ));

    inner_components.push(serenity::CreateContainerComponent::Separator(
        serenity::CreateSeparator::new(true),
    ));

    // Modules Section
    inner_components.push(serenity::CreateContainerComponent::TextDisplay(
        serenity::CreateTextDisplay::new(format!("### {}", l10n.t("status-modules", None))),
    ));

    // Get enabled modules
    let modules_status = if let Some(gid) = guild_id {
        let db_modules = module_configs::Entity::find()
            .filter(module_configs::Column::GuildId.eq(gid))
            .all(&data.db)
            .await?;

        data.module_definitions
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

    inner_components.push(serenity::CreateContainerComponent::TextDisplay(
        serenity::CreateTextDisplay::new(modules_status),
    ));

    inner_components.push(serenity::CreateContainerComponent::Separator(
        serenity::CreateSeparator::new(false),
    ));

    // Metrics Section
    inner_components.push(serenity::CreateContainerComponent::TextDisplay(
        serenity::CreateTextDisplay::new(format!("### 📈 {}", l10n.t("status-metrics", None))),
    ));

    // Latency
    let total_latency = end_time - interaction_created_at;

    let metrics_text = format!(
        "**{}**: `{} ms`\n**{}**: `{} / {}`",
        l10n.t("status-latency", None),
        total_latency.num_milliseconds(),
        l10n.t("status-shard", None),
        shard_id.get() + 1,
        shard_count
    );

    inner_components.push(serenity::CreateContainerComponent::TextDisplay(
        serenity::CreateTextDisplay::new(metrics_text),
    ));

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

        let sys_info = format!(
            "\n**{}**\nCPU: `{:.2}%`\nMem: `{:.2}%` (`{:.2}` / `{:.2}` GB)",
            l10n.t("status-system", None),
            cpu_usage,
            memory_usage_percent,
            used_gb,
            total_gb
        );
        inner_components.push(serenity::CreateContainerComponent::TextDisplay(
            serenity::CreateTextDisplay::new(sys_info),
        ));
    }

    inner_components.push(serenity::CreateContainerComponent::Separator(
        serenity::CreateSeparator::new(false),
    ));

    // Refresh Button
    let button_row = vec![
        serenity::CreateButton::new("status-refresh")
            .label(l10n.t("status-refresh-btn", None))
            .style(serenity::ButtonStyle::Primary)
            .emoji(serenity::ReactionType::Unicode('🔄'.into())),
    ];

    inner_components.push(serenity::CreateContainerComponent::ActionRow(
        serenity::CreateActionRow::Buttons(button_row.into()),
    ));

    Ok(vec![serenity::CreateComponent::Container(
        serenity::CreateContainer::new(inner_components),
    )])
}

pub async fn handle_interaction(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    data: &Data,
) -> Result<(), Error> {
    if interaction.data.custom_id != "status-refresh" {
        return Ok(());
    }

    let l10n = L10nProxy {
        manager: data.l10n.clone(),
        locale: interaction.locale.to_string(),
    };

    let start_time = interaction.id.created_at().to_utc();

    interaction
        .create_response(&ctx.http, serenity::CreateInteractionResponse::Acknowledge)
        .await?;

    let end_time = Utc::now();

    let components = get_status_components(
        data,
        &l10n,
        interaction.guild_id.map(|id| id.get() as i64),
        ctx.shard_id,
        start_time,
        end_time,
        data.shard_count.load(Ordering::Relaxed),
    )
    .await?;

    interaction
        .edit_response(
            &ctx.http,
            serenity::EditInteractionResponse::new()
                .components(components)
                .flags(serenity::MessageFlags::IS_COMPONENTS_V2 | serenity::MessageFlags::EPHEMERAL),
        )
        .await?;

    Ok(())
}
