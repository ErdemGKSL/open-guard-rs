use fluent::{FluentArgs, FluentResource};
use fluent_bundle::bundle::FluentBundle;
use include_dir::{Dir, include_dir};
use poise::serenity_prelude as serenity;
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info};
use unic_langid::LanguageIdentifier;

// We use the concurrent memoizer to ensure thread safety (Sync + Send)
type ConcurrentBundle = FluentBundle<FluentResource, intl_memoizer::concurrent::IntlLangMemoizer>;

// Embed the locales directory at compile time
static LOCALES_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/locales");

#[derive(Debug, Deserialize)]
pub struct CommandLocale {
    pub name: Option<String>,
    pub desc: Option<String>,
    #[serde(default)]
    pub options: HashMap<String, OptionLocale>,
    #[serde(default)]
    pub subcommands: HashMap<String, CommandLocale>,
}

#[derive(Debug, Deserialize)]
pub struct OptionLocale {
    pub name: Option<String>,
    pub desc: Option<String>,
    #[serde(default)]
    pub choices: HashMap<String, String>,
}

pub struct LocalizationManager {
    bundles: HashMap<LanguageIdentifier, ConcurrentBundle>,
    command_locales: HashMap<LanguageIdentifier, HashMap<String, CommandLocale>>,
}

impl LocalizationManager {
    pub fn new() -> Self {
        let mut bundles = HashMap::new();
        let mut command_locales = HashMap::new();

        // Iterate over subdirectories in the embedded locales directory
        for entry in LOCALES_DIR.dirs() {
            let locale_name = entry.path().to_string_lossy();

            if let Ok(lang_id) = locale_name.parse::<LanguageIdentifier>() {
                let mut bundle = ConcurrentBundle::new_concurrent(vec![lang_id.clone()]);
                let mut commands = HashMap::new();

                // Load files in the locale directory
                for file in entry.files() {
                    let path = file.path();
                    let extension = path.extension().and_then(|e| e.to_str());
                    let file_name = path.file_name().and_then(|n| n.to_str());

                    if extension == Some("ftl") {
                        if let Some(content) = file.contents_utf8() {
                            match FluentResource::try_new(content.to_string()) {
                                Ok(resource) => {
                                    if let Err(errors) = bundle.add_resource(resource) {
                                        for err in errors {
                                            error!(
                                                "Error adding resource for {}: {:?}",
                                                locale_name, err
                                            );
                                        }
                                    }
                                }
                                Err((_, errors)) => {
                                    for err in errors {
                                        error!(
                                            "Error parsing resource for {}: {:?}",
                                            locale_name, err
                                        );
                                    }
                                }
                            }
                        }
                    } else if file_name == Some("commands.yaml")
                        || file_name == Some("commands.yml")
                    {
                        if let Some(content) = file.contents_utf8() {
                            match serde_yaml::from_str::<HashMap<String, CommandLocale>>(content) {
                                Ok(yaml_commands) => {
                                    commands.extend(yaml_commands);
                                }
                                Err(err) => {
                                    error!(
                                        "Error parsing commands.yaml for {}: {:?}",
                                        locale_name, err
                                    );
                                }
                            }
                        }
                    }
                }

                info!("Loaded embedded locale: {}", locale_name);
                bundles.insert(lang_id.clone(), bundle);
                command_locales.insert(lang_id, commands);
            }
        }

        Self {
            bundles,
            command_locales,
        }
    }

    pub async fn get_l10n_for_guild(
        self: &Arc<Self>,
        _guild_id: serenity::GuildId,
        _db: &DatabaseConnection,
    ) -> L10nProxy {
        // Since preferred_locale is not in guild_configs table yet,
        // and fetching guild from HTTP every time is expensive,
        // we default to en-US for system-wide module logs.
        self.get_proxy("en-US")
    }

    pub fn get_proxy(self: &Arc<Self>, locale: &str) -> L10nProxy {
        L10nProxy {
            manager: self.clone(),
            locale: locale.to_string(),
        }
    }

    pub fn translate(&self, locale: &str, key: &str, args: Option<&FluentArgs>) -> String {
        let lang_id = locale
            .parse::<LanguageIdentifier>()
            .unwrap_or_else(|_| "en-US".parse().unwrap());

        // 1. Try to get the message from the requested locale
        if let Some(bundle) = self.bundles.get(&lang_id) {
            if let Some(msg) = bundle.get_message(key) {
                if let Some(pattern) = msg.value() {
                    let mut errors = vec![];
                    return bundle
                        .format_pattern(pattern, args, &mut errors)
                        .into_owned();
                }
            }
        }

        // 2. Fallback to en-US if requested locale failed and it wasn't already en-US
        let en_us: LanguageIdentifier = "en-US".parse().unwrap();
        if lang_id != en_us {
            if let Some(en_bundle) = self.bundles.get(&en_us) {
                if let Some(msg) = en_bundle.get_message(key) {
                    if let Some(pattern) = msg.value() {
                        let mut errors = vec![];
                        return en_bundle
                            .format_pattern(pattern, args, &mut errors)
                            .into_owned();
                    }
                }
            }
        }

        key.to_string()
    }

    pub fn apply_translations<U, E>(&self, commands: &mut [poise::Command<U, E>]) {
        for (lang_id, locales) in &self.command_locales {
            let locale_str = lang_id.to_string();

            for cmd in commands.iter_mut() {
                self.apply_to_command(cmd, locales, &locale_str);
            }
        }
    }

    fn apply_to_command<U, E>(
        &self,
        cmd: &mut poise::Command<U, E>,
        locales: &HashMap<String, CommandLocale>,
        locale_str: &str,
    ) {
        if let Some(loc) = locales.get(cmd.name.as_ref()) {
            // Command name and description localizations
            if let Some(name) = &loc.name {
                cmd.name_localizations
                    .to_mut()
                    .push((locale_str.to_string().into(), name.clone().into()));
                // We also update the BASE name/description if it's en-US (fallback)
                if locale_str == "en-US" {
                    cmd.name = name.clone().into();
                }
            }
            if let Some(desc) = &loc.desc {
                cmd.description_localizations
                    .to_mut()
                    .push((locale_str.to_string().into(), desc.clone().into()));
                if locale_str == "en-US" {
                    cmd.description = Some(desc.clone().into());
                }
            }

            // Options (parameters)
            for param in cmd.parameters.iter_mut() {
                if let Some(opt_loc) = loc.options.get(param.name.as_ref()) {
                    if let Some(name) = &opt_loc.name {
                        param
                            .name_localizations
                            .to_mut()
                            .push((locale_str.to_string().into(), name.clone().into()));
                        if locale_str == "en-US" {
                            param.name = name.clone().into();
                        }
                    }
                    if let Some(desc) = &opt_loc.desc {
                        param
                            .description_localizations
                            .to_mut()
                            .push((locale_str.to_string().into(), desc.clone().into()));
                        if locale_str == "en-US" {
                            param.description = Some(desc.clone().into());
                        }
                    }

                    // Choices
                    for choice in param.choices.to_mut().iter_mut() {
                        if let Some(choice_name) = opt_loc.choices.get(choice.name.as_ref()) {
                            choice
                                .localizations
                                .to_mut()
                                .push((locale_str.to_string().into(), choice_name.clone().into()));
                            if locale_str == "en-US" {
                                choice.name = choice_name.clone().into();
                            }
                        }
                    }
                }
            }

            // Subcommands (recursive search within the current command's locale config)
            for subcommand in cmd.subcommands.iter_mut() {
                self.apply_to_command(subcommand, &loc.subcommands, locale_str);
            }
        }
    }
}

/// A proxy for translation that holds a reference to the manager and a specific locale
pub struct L10nProxy {
    pub manager: Arc<LocalizationManager>,
    pub locale: String,
}

impl L10nProxy {
    pub fn t(&self, key: &str, args: Option<&FluentArgs>) -> String {
        self.manager.translate(&self.locale, key, args)
    }
}

/// Holds localization proxies for both the user and the guild
#[allow(unused)]
pub struct LocalizationContext {
    pub user: Option<L10nProxy>,
    pub guild: Option<L10nProxy>,
}

#[allow(unused)]
/// Helper trait to add localization to the Poise context
pub trait ContextL10nExt {
    fn l10n(&self) -> LocalizationContext;
    fn l10n_guild(&self) -> L10nProxy;
    fn l10n_user(&self) -> L10nProxy;
    fn l10n_user_option(&self) -> Option<L10nProxy>;
}

impl ContextL10nExt for crate::Context<'_> {
    fn l10n(&self) -> LocalizationContext {
        let manager = self.data().l10n.clone();

        // User locale is usually only available in interactions
        let user = self.locale().map(|locale| L10nProxy {
            manager: manager.clone(),
            locale: locale.to_string(),
        });

        // Guild locale is available if we are in a guild and have its data
        let guild = self.guild().map(|guild| L10nProxy {
            manager: manager.clone(),
            locale: guild.preferred_locale.to_string(),
        });

        LocalizationContext { user, guild }
    }

    fn l10n_guild(&self) -> L10nProxy {
        let manager = self.data().l10n.clone();
        if let Some(guild) = self.guild() {
            L10nProxy {
                manager,
                locale: guild.preferred_locale.to_string(),
            }
        } else if let Some(locale) = self.locale() {
            L10nProxy {
                manager,
                locale: locale.to_string(),
            }
        } else {
            L10nProxy {
                manager,
                locale: "en-US".to_string(),
            }
        }
    }

    fn l10n_user_option(&self) -> Option<L10nProxy> {
        let manager = self.data().l10n.clone();
        self.locale().map(|locale| L10nProxy {
            manager,
            locale: locale.to_string(),
        })
    }

    fn l10n_user(&self) -> L10nProxy {
        let manager = self.data().l10n.clone();
        if let Some(locale) = self.locale() {
            L10nProxy {
                manager,
                locale: locale.to_string(),
            }
        } else {
            self.l10n_guild()
        }
    }
}

