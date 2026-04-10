use std::error::Error;
use std::fmt::{Display, Formatter};

use url::Url;

use crate::config::ProxySettings;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub field: &'static str,
    pub message: String,
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

impl Error for ValidationError {}

pub fn validate_proxy_settings(settings: &ProxySettings) -> Result<(), ValidationError> {
    validate_url("http_proxy", settings.http_proxy.as_deref())?;
    validate_url("https_proxy", settings.https_proxy.as_deref())?;
    validate_url("all_proxy", settings.all_proxy.as_deref())?;
    Ok(())
}

fn validate_url(field: &'static str, value: Option<&str>) -> Result<(), ValidationError> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(());
    };

    Url::parse(value).map_err(|error| ValidationError {
        field,
        message: error.to_string(),
    })?;

    Ok(())
}
