use crate::models::FrequencyPattern;
use chrono::{DateTime, Datelike, NaiveDate, Utc};

pub fn is_habit_due(
    frequency: &FrequencyPattern,
    reference_date: NaiveDate,
    last_completed: Option<DateTime<Utc>>,
) -> bool {
    let today = reference_date;

    match frequency {
        FrequencyPattern::Daily => {
            // for daily habits, it should be completed every day
            match last_completed {
                Some(last) => {
                    let last_date = last.date_naive();
                    // it's due if the last completion was before today
                    last_date < today
                }
                None => true, // never completed, so it's due
            }
        }
        FrequencyPattern::Weekly { days } => {
            // for weekly habits, check if today is one of the specified days
            let weekday = today.weekday().number_from_monday();
            if !days.contains(&weekday) {
                return false; // not due on this day of the week
            }

            // if it's one of the specified days, check if it was already completed today
            match last_completed {
                Some(last) => last.date_naive() < today,
                None => true,
            }
        }
        FrequencyPattern::Monthly { days } => {
            // for monthly habits, check if today is one of the specified days
            let day_of_month = today.day();
            if !days.contains(&day_of_month) {
                return false; // not due on this day of the month
            }

            // if it's one of the specified days, check if it was already completed today
            match last_completed {
                Some(last) => last.date_naive() < today,
                None => true,
            }
        }
        FrequencyPattern::Interval { days } => {
            // for interval habits, check if enough days have passed since the last completion
            match last_completed {
                Some(last) => {
                    let last_date = last.date_naive();
                    let days_since_last = today.signed_duration_since(last_date).num_days();
                    days_since_last >= *days as i64
                }
                None => true, // never completed, so it's due
            }
        }
        FrequencyPattern::Custom { pattern: _ } => {
            // for now, always assume it's due
            true
        }
    }
}

pub fn breaks_streak(
    frequency: &FrequencyPattern,
    previous_completion: DateTime<Utc>,
    current_date: NaiveDate,
) -> bool {
    let prev_date = previous_completion.date_naive();

    match frequency {
        FrequencyPattern::Daily => {
            // for daily habits, streak breaks if more than 1 day passed
            let days_diff = current_date.signed_duration_since(prev_date).num_days();
            days_diff > 1
        }
        FrequencyPattern::Weekly { days } => {
            // for weekly, check if any expected days were missed
            let mut check_date = prev_date;
            while check_date < current_date {
                check_date = check_date.succ_opt().unwrap(); // move to next day
                let weekday = check_date.weekday().number_from_monday();

                // ff this day is in the frequency pattern and it's not the current date,
                // then we missed a day that breaks the streak
                if days.contains(&weekday) && check_date < current_date {
                    return true;
                }
            }
            false
        }
        FrequencyPattern::Monthly { days } => {
            // for monthly, check if any expected days were missed
            let mut check_date = prev_date;
            while check_date < current_date {
                check_date = check_date.succ_opt().unwrap(); // move to next day
                let day_of_month = check_date.day();

                // ff this day is in the frequency pattern and it's not the current date,
                // then we missed a day that breaks the streak
                if days.contains(&day_of_month) && check_date < current_date {
                    return true;
                }
            }
            false
        }
        FrequencyPattern::Interval { days } => {
            // for interval habits, streak breaks if more than specified interval passed
            let days_diff = current_date.signed_duration_since(prev_date).num_days();
            days_diff > *days as i64
        }
        FrequencyPattern::Custom { pattern: _ } => {
            // for now, assume no streak break
            false
        }
    }
}
