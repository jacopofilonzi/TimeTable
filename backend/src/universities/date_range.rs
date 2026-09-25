use chrono::{DateTime, Datelike, Days, NaiveDate, NaiveTime, TimeZone};
use chrono_tz::Tz;

/// Time window for lessons: from Monday 00:00 of the current week, for `weeks` weeks.
#[derive(Debug, Clone)]
pub struct DateRange {
    pub from: DateTime<Tz>,
    /// Exclusive.
    pub to: DateTime<Tz>,
}

impl DateRange {
    pub fn current_weeks<T: TimeZone>(now: DateTime<T>, tz: Tz, weeks: u8) -> Self {
        let today = now.with_timezone(&tz).date_naive();
        let monday = today - Days::new(u64::from(today.weekday().num_days_from_monday()));
        let end = monday + Days::new(7 * u64::from(weeks));
        let at_midnight = |d: NaiveDate| {
            tz.from_local_datetime(&d.and_time(NaiveTime::MIN))
                .earliest()
                .expect("midnight exists in every supported timezone")
        };
        Self {
            from: at_midnight(monday),
            to: at_midnight(end),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn range_starts_on_monday_local_time() {
        let now = Utc.with_ymd_and_hms(2026, 9, 27, 23, 30, 0).unwrap(); // Monday 28th 01:30 in Rome
        let range = DateRange::current_weeks(now, chrono_tz::Europe::Rome, 2);
        assert_eq!(range.from.to_rfc3339(), "2026-09-28T00:00:00+02:00");
        assert_eq!(range.to.to_rfc3339(), "2026-10-12T00:00:00+02:00");
    }

    #[test]
    fn range_across_dst_change() {
        let now = Utc.with_ymd_and_hms(2026, 10, 21, 12, 0, 0).unwrap();
        let range = DateRange::current_weeks(now, chrono_tz::Europe::Rome, 1);
        assert_eq!(range.from.to_rfc3339(), "2026-10-19T00:00:00+02:00");
        assert_eq!(range.to.to_rfc3339(), "2026-10-26T00:00:00+01:00");
    }
}
