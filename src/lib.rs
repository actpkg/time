use act_sdk::prelude::*;
use chrono::{DateTime, Datelike, Duration, NaiveDate, NaiveDateTime, Timelike, Utc};
use chrono_tz::Tz;

#[act_component(
    name = "time",
    version = "0.1.0",
    description = "Time and timezone utilities",
)]
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

    /// Convert a datetime string between timezones.
    #[act_tool(description = "Convert time between timezones", read_only)]
    fn convert_timezone(
        #[doc = "ISO 8601 / RFC 3339 datetime string"] time: String,
        #[doc = "Source IANA timezone (used if datetime has no offset)"] from_timezone: Option<String>,
        #[doc = "Target IANA timezone"] to_timezone: String,
    ) -> ActResult<String> {
        let to_tz: Tz = to_timezone
            .parse()
            .map_err(|_| ActError::invalid_args(format!("Unknown target timezone: {to_timezone}")))?;

        // Try parsing as RFC 3339 first (has offset)
        if let Ok(dt) = DateTime::parse_from_rfc3339(&time) {
            return Ok(dt.with_timezone(&to_tz).to_rfc3339());
        }

        // Parse as naive datetime + source timezone
        let from_tz_str = from_timezone
            .as_deref()
            .ok_or_else(|| ActError::invalid_args("Datetime has no offset; provide from_timezone"))?;
        let from_tz: Tz = from_tz_str
            .parse()
            .map_err(|_| ActError::invalid_args(format!("Unknown source timezone: {from_tz_str}")))?;

        let naive = NaiveDateTime::parse_from_str(&time, "%Y-%m-%dT%H:%M:%S")
            .or_else(|_| NaiveDateTime::parse_from_str(&time, "%Y-%m-%d %H:%M:%S"))
            .map_err(|e| ActError::invalid_args(format!("Cannot parse datetime: {e}")))?;

        let src_dt = naive
            .and_local_timezone(from_tz)
            .single()
            .ok_or_else(|| {
                ActError::invalid_args("Ambiguous or invalid datetime in source timezone")
            })?;

        Ok(src_dt.with_timezone(&to_tz).to_rfc3339())
    }

    /// Add or subtract a duration from a datetime.
    #[act_tool(description = "Add or subtract duration from a datetime", read_only)]
    fn date_arithmetic(
        #[doc = "ISO 8601 datetime string"] datetime: String,
        #[doc = "Operation: 'add' or 'subtract'"] operation: String,
        #[doc = "Number of years to add/subtract"] years: Option<i32>,
        #[doc = "Number of months to add/subtract"] months: Option<i32>,
        #[doc = "Number of days to add/subtract"] days: Option<i64>,
        #[doc = "Number of hours to add/subtract"] hours: Option<i64>,
        #[doc = "Number of minutes to add/subtract"] minutes: Option<i64>,
        #[doc = "Number of seconds to add/subtract"] seconds: Option<i64>,
    ) -> ActResult<String> {
        let dt = DateTime::parse_from_rfc3339(&datetime)
            .map_err(|e| ActError::invalid_args(format!("Cannot parse datetime: {e}")))?;

        let sign: i64 = match operation.as_str() {
            "add" => 1,
            "subtract" => -1,
            _ => return Err(ActError::invalid_args("operation must be 'add' or 'subtract'")),
        };

        let mut result = dt.with_timezone(&Utc);

        // Handle years and months via chrono's date manipulation
        if let Some(y) = years {
            let target_year = result.year() + (y as i32 * sign as i32);
            result = result
                .with_year(target_year)
                .ok_or_else(|| ActError::internal("Invalid year result"))?;
        }
        if let Some(m) = months {
            let total_months =
                result.year() * 12 + result.month0() as i32 + (m as i32 * sign as i32);
            let target_year = total_months.div_euclid(12);
            let target_month0 = total_months.rem_euclid(12) as u32;
            result = result
                .with_year(target_year)
                .and_then(|d| d.with_month0(target_month0))
                .ok_or_else(|| ActError::internal("Invalid month result"))?;
        }

        // Handle days/hours/minutes/seconds via Duration
        let dur = Duration::days(days.unwrap_or(0) * sign)
            + Duration::hours(hours.unwrap_or(0) * sign)
            + Duration::minutes(minutes.unwrap_or(0) * sign)
            + Duration::seconds(seconds.unwrap_or(0) * sign);
        result = result + dur;

        Ok(result.to_rfc3339())
    }

    /// List available IANA timezone names, optionally filtered.
    #[act_tool(description = "List available IANA timezone names", read_only)]
    fn list_timezones(
        #[doc = "Optional filter string (case-insensitive substring match)"] filter: Option<String>,
    ) -> ActResult<String> {
        use chrono_tz::TZ_VARIANTS;
        let names: Vec<&str> = TZ_VARIANTS
            .iter()
            .map(|tz| tz.name())
            .filter(|name| {
                filter
                    .as_ref()
                    .map_or(true, |f| name.to_lowercase().contains(&f.to_lowercase()))
            })
            .collect();
        serde_json::to_string_pretty(&names)
            .map_err(|e| ActError::internal(format!("JSON error: {e}")))
    }

    /// Parse a datetime string and return structured components.
    #[act_tool(description = "Parse a datetime string into structured components", read_only)]
    fn parse_datetime(
        #[doc = "Datetime string to parse (ISO 8601, RFC 3339, or common formats)"] input: String,
        #[doc = "IANA timezone for context (used if input has no offset)"] timezone: Option<String>,
    ) -> ActResult<String> {
        // Try RFC 3339
        if let Ok(dt) = DateTime::parse_from_rfc3339(&input) {
            let utc = dt.with_timezone(&Utc);
            return Ok(format_parsed(&utc));
        }

        // Try common formats
        let formats = [
            "%Y-%m-%dT%H:%M:%S",
            "%Y-%m-%d %H:%M:%S",
            "%Y-%m-%d %H:%M",
            "%Y-%m-%d",
            "%d/%m/%Y %H:%M:%S",
            "%d/%m/%Y",
            "%m/%d/%Y %H:%M:%S",
            "%m/%d/%Y",
        ];

        for fmt in &formats {
            if let Ok(naive) = NaiveDateTime::parse_from_str(&input, fmt) {
                let tz: Tz = timezone
                    .as_deref()
                    .unwrap_or("UTC")
                    .parse()
                    .map_err(|_| ActError::invalid_args("Unknown timezone"))?;
                if let Some(dt) = naive.and_local_timezone(tz).single() {
                    let utc = dt.with_timezone(&Utc);
                    return Ok(format_parsed(&utc));
                }
            }
            // Try as date only
            if let Ok(naive_date) = NaiveDate::parse_from_str(&input, fmt) {
                let naive = naive_date.and_hms_opt(0, 0, 0).unwrap();
                let tz: Tz = timezone
                    .as_deref()
                    .unwrap_or("UTC")
                    .parse()
                    .map_err(|_| ActError::invalid_args("Unknown timezone"))?;
                if let Some(dt) = naive.and_local_timezone(tz).single() {
                    let utc = dt.with_timezone(&Utc);
                    return Ok(format_parsed(&utc));
                }
            }
        }

        Err(ActError::invalid_args(format!(
            "Cannot parse datetime: {input}"
        )))
    }
}

fn format_parsed(dt: &DateTime<Utc>) -> String {
    serde_json::json!({
        "iso8601": dt.to_rfc3339(),
        "unix_timestamp": dt.timestamp(),
        "year": dt.year(),
        "month": dt.month(),
        "day": dt.day(),
        "hour": dt.hour(),
        "minute": dt.minute(),
        "second": dt.second(),
        "day_of_week": dt.weekday().to_string(),
        "day_of_year": dt.ordinal(),
    })
    .to_string()
}
