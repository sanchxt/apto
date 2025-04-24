use crate::features::habits::models::FrequencyPattern;
use serde_json;

// helper function to convert FrequencyPattern to database format
pub fn serialize_frequency(
    frequency: &FrequencyPattern,
) -> Result<(String, String), serde_json::Error> {
    let frequency_type = match frequency {
        FrequencyPattern::Daily => "daily".to_string(),
        FrequencyPattern::Weekly { .. } => "weekly".to_string(),
        FrequencyPattern::Monthly { .. } => "monthly".to_string(),
        FrequencyPattern::Interval { .. } => "interval".to_string(),
        FrequencyPattern::Custom { .. } => "custom".to_string(),
    };

    let frequency_data = match frequency {
        FrequencyPattern::Daily => "{}".to_string(),
        FrequencyPattern::Weekly { days } => serde_json::to_string(&days)?,
        FrequencyPattern::Monthly { days } => serde_json::to_string(&days)?,
        FrequencyPattern::Interval { days } => serde_json::to_string(days)?,
        FrequencyPattern::Custom { pattern } => serde_json::to_string(pattern)?,
    };

    Ok((frequency_type, frequency_data))
}

// helper function to convert database format to FrequencyPattern
pub fn deserialize_frequency(
    freq_type: &str,
    freq_data: &str,
) -> Result<FrequencyPattern, serde_json::Error> {
    match freq_type {
        "daily" => Ok(FrequencyPattern::Daily),
        "weekly" => {
            let days: Vec<u32> = serde_json::from_str(freq_data)?;
            Ok(FrequencyPattern::Weekly { days })
        }
        "monthly" => {
            let days: Vec<u32> = serde_json::from_str(freq_data)?;
            Ok(FrequencyPattern::Monthly { days })
        }
        "interval" => {
            let days: u32 = serde_json::from_str(freq_data)?;
            Ok(FrequencyPattern::Interval { days })
        }
        "custom" => {
            let pattern: String = serde_json::from_str(freq_data)?;
            Ok(FrequencyPattern::Custom { pattern })
        }
        _ => {
            let msg = format!("Unknown frequency type: {}", freq_type);
            Err(serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                msg,
            )))
        }
    }
}
