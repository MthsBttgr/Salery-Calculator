use chrono::{Datelike, Duration, Months, NaiveDate, NaiveDateTime, NaiveTime};

use crate::wage_bonuses::WageAndBonuses;

pub fn calculate_shift_time(shift_start: NaiveDateTime, shift_end: NaiveDateTime) -> Duration {
    let dur = shift_end.signed_duration_since(shift_start);

    dur
}

#[derive(Debug)]
pub struct SaleryPeriod {
    start: NaiveDateTime,
    end: NaiveDateTime,
}

impl SaleryPeriod {
    pub fn new(start: NaiveDateTime, end: NaiveDateTime) -> Self {
        Self { start, end }
    }

    pub fn start(&self) -> NaiveDateTime {
        self.start
    }

    pub fn end(&self) -> NaiveDateTime {
        self.end
    }
}

pub fn current_salery_period(wage_bonuses: &WageAndBonuses) -> SaleryPeriod {
    let today = chrono::Local::now();

    let current_month_period_end =
        NaiveDate::from_ymd_opt(today.year(), today.month(), wage_bonuses.period().end_day())
            .expect("the end day of the salery period is not valid");
    let previous_month_period_start = NaiveDate::from_ymd_opt(
        today.year(),
        today.month(),
        wage_bonuses.period().start_day(),
    )
    .expect("the start day of the salery period is not valid")
    .checked_sub_months(Months::new(1))
    .unwrap();

    let mut salery_period_end = current_month_period_end;
    let mut salery_period_start = previous_month_period_start;

    if today.date_naive() > current_month_period_end {
        salery_period_end = salery_period_end
            .checked_add_months(Months::new(1))
            .unwrap();
        salery_period_start = salery_period_start
            .checked_add_months(Months::new(1))
            .unwrap();
    }

    let salery_period_start =
        salery_period_start.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
    let salery_period_end = salery_period_end.and_time(NaiveTime::from_hms_opt(23, 59, 0).unwrap());

    SaleryPeriod::new(salery_period_start, salery_period_end)
}

pub fn salery_period_from_offset(wage_bonuses: &WageAndBonuses, offset: u32) -> SaleryPeriod {
    let mut salery_period = current_salery_period(wage_bonuses);
    salery_period.start = salery_period
        .start
        .checked_sub_months(Months::new(offset))
        .unwrap();

    salery_period.end = salery_period
        .end
        .checked_sub_months(Months::new(offset))
        .unwrap();

    salery_period
}

pub fn parse_naivedatetime_from_str(date_time: &str) -> Result<NaiveDateTime, Vec<String>> {
    // formats can be switched around to change priority. First match is returned
    let formats = &[
        "%d-%m-%Y %H:%M",    // danish time format
        "%Y-%d-%m %H:%M",    // danish time format with year in front
        "%d-%m-%Y %H:%M:%s", // danish time format with seconds
        "%Y-%d-%m %H:%M:%s", // danish time format with year in front and seconds
        "%Y-%m-%d %H:%M",    // american time format
        "%Y-%m-%d %H:%M:%S", // american time format with seconds
    ];
    let mut errors = Vec::new();

    for format in formats {
        match NaiveDateTime::parse_from_str(date_time, format) {
            Ok(result) => return Ok(result),
            Err(err) => {
                let error_message = format!(
                    "encountered error: ({}), when trying to format input: \"{}\", according format: {}",
                    err, date_time, format
                );
                errors.push(error_message);
            }
        }
    }

    // try adding current year to the front of input string and testing the formats again
    // this allows adding shifts without specifying the year, which would be redundant user experience
    let today = chrono::Local::now();
    let date = today.date_naive();

    let input = format!("{}-{}", date.year(), date_time);
    for format in formats {
        match NaiveDateTime::parse_from_str(&input, format) {
            Ok(result) => return Ok(result),
            Err(err) => {
                let error_message = format!(
                    "encountered error: ({}), when trying to format input: \"{}\", according format: {}",
                    err, input, format
                );
                errors.push(error_message);
            }
        }
    }

    // If none of the formats matched
    Err(errors)
}

pub trait SQLformat {
    fn sql_format(&self) -> String;
}

impl SQLformat for NaiveDateTime {
    fn sql_format(&self) -> String {
        format!("{} {}", self.date(), self.time())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn different_str_formats_for_parsing_naivedatetime() {
        let input = "2023-12-23 23:59";
        let result = parse_naivedatetime_from_str(input).unwrap();
        let expected = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2023, 12, 23).unwrap(),
            NaiveTime::from_hms_opt(23, 59, 00).unwrap(),
        );

        assert_eq!(result, expected);
    }
}
