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

pub fn translate(ctx: &crate::Context<'_>, key: &str, args: Option<&fluent::FluentArgs>) -> String {
    let locale = ctx.locale().unwrap_or("en-US");
    ctx.data().l10n.translate(locale, key, args)
}
