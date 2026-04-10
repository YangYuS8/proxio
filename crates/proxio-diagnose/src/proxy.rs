use proxio_core::ProxySettings;
use url::Url;

use crate::model::{EffectiveProxy, TransportMode};

pub fn select_effective_proxy(
    url: &str,
    settings: &ProxySettings,
) -> Result<EffectiveProxy, String> {
    let url = Url::parse(url).map_err(|err| err.to_string())?;
    let value = match url.scheme() {
        "https" => settings
            .https_proxy
            .as_ref()
            .filter(|value| !value.trim().is_empty())
            .cloned()
            .or_else(|| {
                settings
                    .all_proxy
                    .as_ref()
                    .filter(|value| !value.trim().is_empty())
                    .cloned()
            }),
        "http" => settings
            .http_proxy
            .as_ref()
            .filter(|value| !value.trim().is_empty())
            .cloned()
            .or_else(|| {
                settings
                    .all_proxy
                    .as_ref()
                    .filter(|value| !value.trim().is_empty())
                    .cloned()
            }),
        scheme => return Err(format!("unsupported URL scheme: {scheme}")),
    };

    Ok(match value {
        Some(value) => EffectiveProxy {
            mode: TransportMode::Proxied,
            value: Some(value),
        },
        None => EffectiveProxy {
            mode: TransportMode::Direct,
            value: None,
        },
    })
}
