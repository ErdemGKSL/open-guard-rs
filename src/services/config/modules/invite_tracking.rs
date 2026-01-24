use crate::Data;
use crate::db::entities::module_configs::{self, InviteTrackingModuleConfig, ModuleType};
use crate::services::localization::L10nProxy;
use poise::serenity_prelude as serenity;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};

pub fn build_ui(
    config: &InviteTrackingModuleConfig,
    l10n: &L10nProxy,
) -> Vec<serenity::CreateContainerComponent<'static>> {
    let mut components = vec![];

    // Track Vanity URL Toggle
    let vanity_label = if config.track_vanity {
        l10n.t("config-btn-enabled", None)
    } else {
        l10n.t("config-btn-disabled", None)
    };

    let vanity_button = serenity::CreateButton::new("config_it_vanity_toggle")
        .label(vanity_label)
        .style(if config.track_vanity {
            serenity::ButtonStyle::Success
        } else {
            serenity::ButtonStyle::Secondary
        });

    components.push(serenity::CreateContainerComponent::Section(
        serenity::CreateSection::new(
            vec![serenity::CreateSectionComponent::TextDisplay(
                serenity::CreateTextDisplay::new(l10n.t("config-it-vanity-label", None)),
            )],
            serenity::CreateSectionAccessory::Button(vanity_button),
        ),
    ));

    components.push(serenity::CreateContainerComponent::Separator(
        serenity::CreateSeparator::new(false),
    ));

    // Ignore Bots Toggle
    let bots_label = if config.ignore_bots {
        l10n.t("config-btn-enabled", None)
    } else {
        l10n.t("config-btn-disabled", None)
    };

    let bots_button = serenity::CreateButton::new("config_it_ignore_bots_toggle")
        .label(bots_label)
        .style(if config.ignore_bots {
            serenity::ButtonStyle::Success
        } else {
            serenity::ButtonStyle::Secondary
        });

    components.push(serenity::CreateContainerComponent::Section(
        serenity::CreateSection::new(
            vec![serenity::CreateSectionComponent::TextDisplay(
                serenity::CreateTextDisplay::new(l10n.t("config-it-ignore-bots-label", None)),
            )],
            serenity::CreateSectionAccessory::Button(bots_button),
        ),
    ));

    components.push(serenity::CreateContainerComponent::Separator(
        serenity::CreateSeparator::new(true),
    ));

    // Minimum Account Age
    let mut args = fluent_bundle::FluentArgs::new();
    args.set("count", config.minimum_account_age_days);
    components.push(serenity::CreateContainerComponent::TextDisplay(
        serenity::CreateTextDisplay::new(l10n.t("config-it-min-age-label", Some(&args))),
    ));

    components.push(serenity::CreateContainerComponent::ActionRow(
        serenity::CreateActionRow::Buttons(
            vec![
                serenity::CreateButton::new("config_it_min_age_dec")
                    .label("-")
                    .style(serenity::ButtonStyle::Secondary),
                serenity::CreateButton::new("config_it_min_age_inc")
                    .label("+")
                    .style(serenity::ButtonStyle::Secondary),
            ]
            .into(),
        ),
    ));

    components.push(serenity::CreateContainerComponent::Separator(
        serenity::CreateSeparator::new(false),
    ));

    // Fake Threshold
    let mut args = fluent_bundle::FluentArgs::new();
    args.set("count", config.fake_threshold_hours);
    components.push(serenity::CreateContainerComponent::TextDisplay(
        serenity::CreateTextDisplay::new(l10n.t("config-it-fake-threshold-label", Some(&args))),
    ));

    components.push(serenity::CreateContainerComponent::ActionRow(
        serenity::CreateActionRow::Buttons(
            vec![
                serenity::CreateButton::new("config_it_fake_dec")
                    .label("-")
                    .style(serenity::ButtonStyle::Secondary),
                serenity::CreateButton::new("config_it_fake_inc")
                    .label("+")
                    .style(serenity::ButtonStyle::Secondary),
            ]
            .into(),
        ),
    ));

    components.push(serenity::CreateContainerComponent::Separator(
        serenity::CreateSeparator::new(false),
    ));

    // Leaderboard Limit
    let mut args = fluent_bundle::FluentArgs::new();
    args.set("count", config.leaderboard_limit);
    components.push(serenity::CreateContainerComponent::TextDisplay(
        serenity::CreateTextDisplay::new(l10n.t("config-it-leaderboard-limit-label", Some(&args))),
    ));

    components.push(serenity::CreateContainerComponent::ActionRow(
        serenity::CreateActionRow::Buttons(
            vec![
                serenity::CreateButton::new("config_it_limit_dec")
                    .label("-")
                    .style(serenity::ButtonStyle::Secondary),
                serenity::CreateButton::new("config_it_limit_inc")
                    .label("+")
                    .style(serenity::ButtonStyle::Secondary),
            ]
            .into(),
        ),
    ));

    components
}

pub async fn handle_interaction(
    _ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    data: &Data,
    guild_id: serenity::GuildId,
) -> Result<bool, crate::Error> {
    let custom_id = &interaction.data.custom_id;

    if custom_id == "config_it_vanity_toggle" {
        let (config_active, mut config) = get_config(data, guild_id).await?;
        config.track_vanity = !config.track_vanity;
        save_config(data, config_active, config).await?;
        return Ok(true);
    }

    if custom_id == "config_it_ignore_bots_toggle" {
        let (config_active, mut config) = get_config(data, guild_id).await?;
        config.ignore_bots = !config.ignore_bots;
        save_config(data, config_active, config).await?;
        return Ok(true);
    }

    if custom_id == "config_it_min_age_inc" {
        let (config_active, mut config) = get_config(data, guild_id).await?;
        config.minimum_account_age_days += 1;
        save_config(data, config_active, config).await?;
        return Ok(true);
    }

    if custom_id == "config_it_min_age_dec" {
        let (config_active, mut config) = get_config(data, guild_id).await?;
        config.minimum_account_age_days = config.minimum_account_age_days.saturating_sub(1);
        save_config(data, config_active, config).await?;
        return Ok(true);
    }

    if custom_id == "config_it_fake_inc" {
        let (config_active, mut config) = get_config(data, guild_id).await?;
        config.fake_threshold_hours += 1;
        save_config(data, config_active, config).await?;
        return Ok(true);
    }

    if custom_id == "config_it_fake_dec" {
        let (config_active, mut config) = get_config(data, guild_id).await?;
        config.fake_threshold_hours = config.fake_threshold_hours.saturating_sub(1);
        save_config(data, config_active, config).await?;
        return Ok(true);
    }

    if custom_id == "config_it_limit_inc" {
        let (config_active, mut config) = get_config(data, guild_id).await?;
        config.leaderboard_limit = (config.leaderboard_limit + 5).min(100);
        save_config(data, config_active, config).await?;
        return Ok(true);
    }

    if custom_id == "config_it_limit_dec" {
        let (config_active, mut config) = get_config(data, guild_id).await?;
        config.leaderboard_limit = config.leaderboard_limit.saturating_sub(5).max(5);
        save_config(data, config_active, config).await?;
        return Ok(true);
    }

    Ok(false)
}

async fn get_config(
    data: &Data,
    guild_id: serenity::GuildId,
) -> Result<(module_configs::ActiveModel, InviteTrackingModuleConfig), crate::Error> {
    let db = &data.db;
    let m_config =
        module_configs::Entity::find_by_id((guild_id.get() as i64, ModuleType::InviteTracking))
            .one(db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Config not found"))?;

    let config: InviteTrackingModuleConfig =
        serde_json::from_value(m_config.config.clone()).unwrap_or_default();
    Ok((m_config.into(), config))
}

async fn save_config(
    data: &Data,
    mut config_active: module_configs::ActiveModel,
    config: InviteTrackingModuleConfig,
) -> Result<(), crate::Error> {
    config_active.config = Set(serde_json::to_value(config)?);
    config_active.update(&data.db).await?;
    Ok(())
}
