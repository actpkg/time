use act_sdk::prelude::*;
use chrono::Utc;
use chrono_tz::Tz;

#[act_component]
mod component {
    use super::*;

    /// Get current time in a timezone (IANA name, e.g. "America/New_York"). Defaults to UTC.
    #[act_tool(description = "Get current time in a specified timezone", read_only)]
    fn get_current_time(
        #[doc = "IANA timezone name (e.g. 'Europe/Moscow', 'America/New_York'). Defaults to UTC."]
        timezone: Option<String>,
    ) -> ActResult<String> {
        let now = Utc::now();
        match timezone {
            Some(tz_name) => {
                let tz: Tz = tz_name
                    .parse()
                    .map_err(|_| ActError::invalid_args(format!("Unknown timezone: {tz_name}")))?;
                Ok(now.with_timezone(&tz).to_rfc3339())
            }
            None => Ok(now.to_rfc3339()),
        }
    }
}
