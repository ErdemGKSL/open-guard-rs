use fluent::{FluentArgs, FluentResource};
use fluent_bundle::bundle::FluentBundle;
use std::collections::HashMap;
use std::fs;
use tracing::{error, info};
use unic_langid::LanguageIdentifier;

// We use the concurrent memoizer to ensure thread safety (Sync + Send)
type ConcurrentBundle = FluentBundle<FluentResource, intl_memoizer::concurrent::IntlLangMemoizer>;

pub struct LocalizationManager {
    bundles: HashMap<LanguageIdentifier, ConcurrentBundle>,
}

impl LocalizationManager {
    pub fn new() -> Self {
        let mut bundles = HashMap::new();

        // Load all locales from the locales directory
        if let Ok(entries) = fs::read_dir("locales") {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        let locale_name = entry.file_name().to_string_lossy().into_owned();
                        if let Ok(lang_id) = locale_name.parse::<LanguageIdentifier>() {
                            let mut bundle =
                                ConcurrentBundle::new_concurrent(vec![lang_id.clone()]);

                            // Load all .ftl files in the directory
                            let path = entry.path();
                            if let Ok(files) = fs::read_dir(path) {
                                for file in files.flatten() {
                                    if file.path().extension().map_or(false, |ext| ext == "ftl") {
                                        if let Ok(content) = fs::read_to_string(file.path()) {
                                            match FluentResource::try_new(content) {
                                                Ok(resource) => {
                                                    if let Err(errors) =
                                                        bundle.add_resource(resource)
                                                    {
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
                                    }
                                }
                            }
                            info!("Loaded locale: {}", locale_name);
                            bundles.insert(lang_id, bundle);
                        }
                    }
                }
            }
        }

        Self { bundles }
    }

    pub fn translate(&self, locale: &str, key: &str, args: Option<&FluentArgs>) -> String {
        let lang_id = locale
            .parse::<LanguageIdentifier>()
            .unwrap_or_else(|_| "en-US".parse().unwrap());

        let bundle = self.bundles.get(&lang_id).or_else(|| {
            // Fallback to en-US if the requested locale is not available
            self.bundles.get(&"en-US".parse().unwrap())
        });

        if let Some(bundle) = bundle {
            if let Some(msg) = bundle.get_message(key) {
                if let Some(pattern) = msg.value() {
                    let mut errors = vec![];
                    return bundle
                        .format_pattern(pattern, args, &mut errors)
                        .into_owned();
                }
            }
        }

        key.to_string()
    }
}

/// A proxy for translation that holds a reference to the manager and a specific locale
pub struct L10nProxy<'a> {
    manager: &'a LocalizationManager,
    locale: String,
}

impl<'a> L10nProxy<'a> {
    pub fn t(&self, key: &str, args: Option<&FluentArgs>) -> String {
        self.manager.translate(&self.locale, key, args)
    }
}

/// Holds localization proxies for both the user and the guild
#[allow(unused)]
pub struct LocalizationContext<'a> {
    pub user: Option<L10nProxy<'a>>,
    pub guild: Option<L10nProxy<'a>>,
}

#[allow(unused)]
/// Helper trait to add localization to the Poise context
pub trait ContextL10nExt {
    fn l10n(&self) -> LocalizationContext<'_>;
    fn l10n_guild(&self) -> L10nProxy<'_>;
    fn l10n_user(&self) -> L10nProxy<'_>;
    fn l10n_user_option(&self) -> Option<L10nProxy<'_>>;
}

impl ContextL10nExt for crate::Context<'_> {
    fn l10n(&self) -> LocalizationContext<'_> {
        // User locale is usually only available in interactions
        let user = self.locale().map(|locale| L10nProxy {
            manager: &self.data().l10n,
            locale: locale.to_string(),
        });

        // Guild locale is available if we are in a guild and have its data
        let guild = self.guild().map(|guild| L10nProxy {
            manager: &self.data().l10n,
            locale: guild.preferred_locale.clone(),
        });

        LocalizationContext { user, guild }
    }
    fn l10n_guild(&self) -> L10nProxy<'_> {
        // if guild does not exists fall back to user with raw coding, since it will be infinite loop if we directly use l10n_user, if this also is not exists fall back to en-US
        self.guild()
            .map(|guild| L10nProxy {
                manager: &self.data().l10n,
                locale: guild.preferred_locale.clone(),
            })
            .or_else(|| self.l10n_user_option())
            .unwrap_or_else(|| L10nProxy {
                manager: &self.data().l10n,
                locale: "en-US".to_string(),
            })
    }

    fn l10n_user_option(&self) -> Option<L10nProxy<'_>> {
        self.locale().map(|locale| L10nProxy {
            manager: &self.data().l10n,
            locale: locale.to_string(),
        })
    }

    fn l10n_user(&self) -> L10nProxy<'_> {
        self.locale()
            .map(|locale| L10nProxy {
                manager: &self.data().l10n,
                locale: locale.to_string(),
            })
            .unwrap_or_else(|| self.l10n_guild())
    }
}
