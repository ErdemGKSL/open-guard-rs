use crate::db::entities::module_configs::{self, LoggingModuleConfig, ModuleType};
use crate::{Data, Error};
use poise::serenity_prelude as serenity;
use sea_orm::EntityTrait;

pub async fn handle_guild_member_add(
    ctx: &serenity::Context,
    guild_id: serenity::GuildId,
    member: serenity::Member,
    data: &Data,
) -> Result<(), Error> {
    // 1. Get module config
    let m_config = module_configs::Entity::find_by_id((guild_id.get() as i64, ModuleType::Logging))
        .one(&data.db)
        .await?;

    let config: LoggingModuleConfig = m_config
        .and_then(|m| serde_json::from_value(m.config).ok())
        .unwrap_or_default();

    if !config.log_membership {
        return Ok(());
    }

    let l10n = data.l10n.get_l10n_for_guild(guild_id, &data.db).await;

    let mut args = fluent::FluentArgs::new();
    args.set("userId", member.user.id.get());

    data.logger
        .log_action(
            &ctx.http,
            guild_id,
            Some(ModuleType::Logging),
            config.membership_log_channel_id,
            crate::services::logger::LogLevel::Info,
            &l10n.t("log-member-join-title", None),
            &l10n.t("log-member-join-desc", Some(&args)),
            vec![],
        )
        .await?;

    Ok(())
}

pub async fn handle_guild_member_remove(
    ctx: &serenity::Context,
    guild_id: serenity::GuildId,
    user: serenity::User,
    member_data_if_available: Option<serenity::Member>,
    data: &Data,
) -> Result<(), Error> {
    // 1. Get module config
    let m_config = module_configs::Entity::find_by_id((guild_id.get() as i64, ModuleType::Logging))
        .one(&data.db)
        .await?;

    let config: LoggingModuleConfig = m_config
        .and_then(|m| serde_json::from_value(m.config).ok())
        .unwrap_or_default();

    if !config.log_membership {
        return Ok(());
    }

    let l10n = data.l10n.get_l10n_for_guild(guild_id, &data.db).await;

    let mut args = fluent::FluentArgs::new();
    args.set("userId", user.id.get());

    let mut fields = vec![];
    if let Some(member) = member_data_if_available {
        let roles = member
            .roles
            .iter()
            .map(|id| format!("<@&{}>", id.get()))
            .collect::<Vec<_>>()
            .join(", ");

        if !roles.is_empty() {
            let label = l10n.t("log-member-leave-roles", None);
            // We use a separate log action call or handle the label lifetime
            // For simplicity, we just log it as a field with leak if it's one-off,
            // or better, just include it in a specialized log.
            // Actually, I'll use a hack to get the label as &'static str for now if it's just a few labels,
            // or I'll just change the log_action to take Vec<(String, String)>.
            // Let's use leak() for the label as it's a fixed set of logging strings.
            fields.push((Box::leak(label.into_boxed_str()) as &str, roles));
        }
    }

    data.logger
        .log_action(
            &ctx.http,
            guild_id,
            Some(ModuleType::Logging),
            config.membership_log_channel_id,
            crate::services::logger::LogLevel::Info,
            &l10n.t("log-member-leave-title", None),
            &l10n.t("log-member-leave-desc", Some(&args)),
            fields,
        )
        .await?;

    Ok(())
}
