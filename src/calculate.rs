use chrono::{Datelike, Duration, NaiveDateTime, NaiveTime, Weekday};

use crate::{
    database::Database,
    time::{calculate_shift_time, parse_naivedatetime_from_str, SaleryPeriod},
    wage_bonuses::WageAndBonuses,
};

pub struct SaleryEntry {
    duration: Duration,
    bonus_pr_hour: f64,
}

impl SaleryEntry {
    fn new(duration: Duration, bonus: f64) -> Self {
        Self {
            duration,
            bonus_pr_hour: bonus,
        }
    }
}

pub fn salery_entries_from_period(
    database: &Database,
    wage_and_bonus: &WageAndBonuses,
    salery_period: &SaleryPeriod,
) -> Vec<SaleryEntry> {
    let query = format!(
        "Select * from {} where shift_start <= :shift_end and shift_end > :shift_start",
        database.table()
    );

    let mut salery_entries = Vec::new();

    for row in database
        .prepare(query)
        .unwrap()
        .into_iter()
        .bind(
            &[
                (
                    ":shift_start",
                    format!("{}", salery_period.start()).as_str(),
                ),
                (":shift_end", format!("{}", salery_period.end()).as_str()), //"2023-11-21"
            ][..],
        ) // change this, so it is the current salery period
        .expect("Couldn't prepare statement for gettign shifts when calculating salery")
        .map(|row| row.unwrap())
    {
        let shift_start = parse_naivedatetime_from_str(row.read::<&str, _>("shift_start")).unwrap();
        let shift_end = parse_naivedatetime_from_str(row.read::<&str, _>("shift_end")).unwrap();

        // checking if shift crosses midnight
        if shift_start.date() != shift_end.date() {
            let shift1_start = shift_start;
            let shift1_end = NaiveDateTime::new(
                shift_start.date(),
                NaiveTime::from_hms_opt(23, 59, 59).unwrap(),
            );
            let shift2_start =
                NaiveDateTime::new(shift_end.date(), NaiveTime::from_hms_opt(0, 0, 0).unwrap());
            let shift2_end = shift_end;

            if shift1_start >= salery_period.start() {
                salery_entries.extend(salery_entries_from_shift(
                    &wage_and_bonus,
                    shift1_start,
                    shift1_end,
                ));
            }
            if shift2_end <= salery_period.end() {
                salery_entries.extend(salery_entries_from_shift(
                    &wage_and_bonus,
                    shift2_start,
                    shift2_end,
                ));
            }
        } else {
            salery_entries.extend(salery_entries_from_shift(
                &wage_and_bonus,
                shift_start,
                shift_end,
            ));
        }
    }

    salery_entries
}

pub fn calculate_salery_from_period(
    database: &Database,
    wage_and_bonus: &WageAndBonuses,
    salery_period: SaleryPeriod,
) -> f64 {
    salery_entries_from_period(database, wage_and_bonus, &salery_period)
        .iter()
        .map(|entry| entry.duration.num_minutes() as f64 * entry.bonus_pr_hour / 60.0)
        .sum()
}

pub fn duration_worked(database: &Database, salery_period: &SaleryPeriod) -> Duration {
    let query = format!(
        "Select * from {} where shift_start <= :shift_end and shift_end > :shift_start",
        database.table()
    );

    let mut duration = Duration::weeks(0);

    for row in database
        .prepare(query)
        .unwrap()
        .into_iter()
        .bind(
            &[
                (
                    ":shift_start",
                    format!("{}", salery_period.start()).as_str(),
                ),
                (":shift_end", format!("{}", salery_period.end()).as_str()), //"2023-11-21"
            ][..],
        ) // change this, so it is the current salery period
        .expect("Couldn't prepare statement for gettign shifts when calculating salery")
        .map(|row| row.unwrap())
    {
        let shift_start = parse_naivedatetime_from_str(row.read::<&str, _>("shift_start")).unwrap();
        let shift_end = parse_naivedatetime_from_str(row.read::<&str, _>("shift_end")).unwrap();

        duration = duration
            .checked_add(&shift_end.signed_duration_since(shift_start))
            .unwrap();
    }

    duration
    // salery_entries_from_period(database, wage_and_bonus, &salery_period)
    //     .iter()
    //     .map(|entry| entry.duration)
    //     .sum()
}

// this function can be made way prettier
/// Checks if shift overlaps any "bonus periods" in wage_and_bonuses and returns the produces salery entries
fn salery_entries_from_shift(
    wage_and_bonus: &WageAndBonuses,
    shift_start: NaiveDateTime,
    shift_end: NaiveDateTime,
) -> Vec<SaleryEntry> {
    let total_shift_duration = calculate_shift_time(shift_start, shift_end);

    let mut salery_entries: Vec<SaleryEntry> = wage_and_bonus
        .general_time_periods()
        .iter()
        .filter_map(|bonus| {
            let beginning = shift_start.time().max(bonus.start_time());
            let ending = shift_end.time().min(bonus.end_time());

            if ending > beginning {
                return Some(SaleryEntry::new(
                    ending.signed_duration_since(beginning),
                    bonus.bonus_pr_hour(),
                ));
            }
            None
        })
        .collect();

    salery_entries.extend(
        wage_and_bonus
            .day_of_week_rates()
            .iter()
            .filter_map(|bonus| {
                let shift_weekday = shift_start.weekday();

                if let Some(bonus_weekday) = bonus.days() {
                    let bonus_weekday: Vec<Weekday> = bonus_weekday
                        .iter()
                        .filter_map(|s| s.parse::<Weekday>().ok())
                        .collect();

                    if bonus_weekday.contains(&shift_weekday) {
                        let beginning = shift_start.time().max(bonus.start_time());
                        let ending = shift_end.time().min(bonus.end_time());

                        if ending > beginning {
                            return Some(SaleryEntry::new(
                                ending.signed_duration_since(beginning),
                                bonus.bonus_pr_hour(),
                            ));
                        }
                    }
                }

                None
            })
            .collect::<Vec<SaleryEntry>>(),
    );

    salery_entries.push(SaleryEntry::new(
        total_shift_duration,
        wage_and_bonus.base_rate(),
    ));

    salery_entries
}

// #[cfg(test)]
// mod test {
//     use chrono::NaiveDate;

//     use super::*;
//     #[test]
//     fn calculate() {
//         std::fs::remove_file("test.db").ok();
//         let db = Database::open_or_create_db("test.db", "test");

//         for day in 2..=4 {
//             let date = NaiveDate::from_ymd_opt(2023, 10, day).unwrap();
//             let time_start = NaiveTime::from_hms_opt(12, 0, 0).unwrap();
//             let time_stop = NaiveTime::from_hms_opt(13, 0, 0).unwrap();
//             let shift_start = NaiveDateTime::new(date, time_start);
//             let shift_end = NaiveDateTime::new(date, time_stop);

//             db.add_shift(shift_start, shift_end, None).unwrap();
//         }

//         let wage_and_bonuses = WageAndBonuses::open("Wage_bonuses_map.json").unwrap();
//         let salery = _calculate_salery(db, &wage_and_bonuses);
//         assert!(salery > 410.0 && salery < 411.0);
//     }
// }
