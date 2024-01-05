use chrono::NaiveTime;
use serde::{Deserialize, Serialize};

use std::error::Error;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct WageAndBonuses {
    base_rate: f64,
    period: Period,
    general_time_periods: Vec<Bonus>,
    day_of_week_rates: Vec<Bonus>,
}

impl WageAndBonuses {
    pub fn new(
        base_rate: f64,
        period: Period,
        general_time_periods: Vec<Bonus>,
        day_of_week_rates: Vec<Bonus>,
    ) -> Self {
        Self {
            base_rate,
            period,
            general_time_periods,
            day_of_week_rates,
        }
    }

    pub fn open<P: AsRef<Path>>(file_path: P) -> Result<WageAndBonuses, Box<dyn Error>> {
        let file = std::fs::File::options()
            // .create(true)
            .read(true)
            .open(file_path)
            .unwrap();
        let reader = std::io::BufReader::new(file);

        Ok(serde_json::from_reader(reader).unwrap())
    }

    pub fn period(&self) -> &Period {
        &self.period
    }

    pub fn base_rate(&self) -> f64 {
        self.base_rate
    }

    pub fn general_time_periods(&self) -> &Vec<Bonus> {
        &self.general_time_periods
    }

    pub fn day_of_week_rates(&self) -> &Vec<Bonus> {
        &self.day_of_week_rates
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum Period {
    Special { start_day: u32, end_day: u32 },
    Month,
}

impl Period {
    pub fn start_day(&self) -> u32 {
        match self {
            Self::Special { start_day, .. } => return *start_day,
            Self::Month => 0,
        }
    }

    pub fn end_day(&self) -> u32 {
        match self {
            Self::Special { end_day, .. } => return *end_day,
            _ => panic!("expected Period to be special"), // need to think of a good way to handle that...
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Bonus {
    bonus_pr_hour: f64,
    start: String,
    end: String,
    days: Option<Vec<String>>,
}

impl Bonus {
    pub fn new(bonus_pr_hour: f64, start: String, end: String, days: Option<Vec<String>>) -> Self {
        Self {
            bonus_pr_hour,
            start,
            end,
            days,
        }
    }

    pub fn add_days(&mut self, days: Vec<String>) {
        self.days = Some(days);
    }

    pub fn start_time(&self) -> NaiveTime {
        NaiveTime::parse_from_str(&self.start, "%H:%M").unwrap()
    }

    pub fn end_time(&self) -> NaiveTime {
        NaiveTime::parse_from_str(&self.end, "%H:%M").unwrap()
    }

    pub fn bonus_pr_hour(&self) -> f64 {
        self.bonus_pr_hour
    }

    pub fn days(&self) -> &Option<Vec<String>> {
        &self.days
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bonus_convert_string_to_time() {
        let bonus = Bonus {
            bonus_pr_hour: 0.0,
            start: "14:00".to_owned(),
            end: "0".to_owned(),
            days: None,
        }
        .start_time();

        let time = NaiveTime::from_hms_opt(14, 0, 0).unwrap();
        assert_eq!(bonus, time);
    }

    #[test]
    fn deserialise_struct_from_json() {
        let expected = WageAndBonuses {
            base_rate: 136.74,
            period: Period::Special {
                start_day: 21,
                end_day: 20,
            },
            general_time_periods: vec![
                Bonus {
                    bonus_pr_hour: 20.77,
                    start: "18:00".to_string(),
                    end: "23:59".to_string(),
                    days: None,
                },
                Bonus {
                    bonus_pr_hour: 28.38,
                    start: "00:00".to_string(),
                    end: "06:00".to_string(),
                    days: None,
                },
            ],
            day_of_week_rates: vec![
                Bonus {
                    bonus_pr_hour: 20.77,
                    start: "14:00".to_string(),
                    end: "24:00".to_string(),
                    days: Some(vec!["lørdag".to_string()]),
                },
                Bonus {
                    bonus_pr_hour: 28.38,
                    start: "06:00".to_string(),
                    end: "24:00".to_string(),
                    days: Some(vec!["søndag".to_string()]),
                },
            ],
        };

        let thing = WageAndBonuses::open("/Wage_bonuses_map.json").unwrap();

        assert_eq!(thing, expected);
    }
}
