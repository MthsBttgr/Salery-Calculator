mod calculate;
mod cli;
mod database;
mod setup;
mod time;
mod wage_bonuses;

use calculate::calculate_salery_from_period;
use chrono::Duration;
use clap::Parser;
use cli::{Cli, Operation};
use database::Database;
use setup::setup_wage_bonuses_if_missing;
use time::{current_salery_period, SQLformat};
use wage_bonuses::WageAndBonuses;

use crate::{
    calculate::duration_worked,
    time::{parse_naivedatetime_from_str, salery_period_from_offset},
};

fn main() {
    let exe_directory = std::env::current_exe().expect("couldn't find the directory of the exe");

    let db_path = exe_directory.with_file_name("DB.db");
    let wage_bonuses_path = exe_directory.with_file_name("Wage_bonuses_map.json");

    setup_wage_bonuses_if_missing(&wage_bonuses_path);

    let cli = Cli::parse();

    let db = Database::open_or_create_db(db_path, "shifts");
    let wage_and_bonuses = WageAndBonuses::open(&wage_bonuses_path).unwrap();

    let Some(op) = cli.operation() else { return };
    match op {
        Operation::List { all, sort, offset } => {
            let salery_period = match offset {
                Some(offset) => salery_period_from_offset(&wage_and_bonuses, *offset),
                None => current_salery_period(&wage_and_bonuses),
            };

            for row in db
                .prepare(format!(
                    "select * from {} {} {}",
                    db.table(),
                    if !all {
                        format!(
                            "where shift_start <= {:#?} and shift_end >= {:#?}",
                            salery_period.end().sql_format(),
                            salery_period.start().sql_format()
                        )
                    } else {
                        String::new()
                    },
                    if *sort {
                        "order by shift_start desc"
                    } else {
                        ""
                    }
                ))
                .unwrap()
                .into_iter()
                .map(|row| row.unwrap())
            {
                println!(
                    "id: {} | shift start: {} | shift end: {}",
                    row.read::<i64, _>("id"),
                    row.read::<&str, _>("shift_start"),
                    row.read::<&str, _>("shift_end")
                );
            }
        }
        Operation::Calculate { offset } => {
            let salery_period = match offset {
                Some(offset) => salery_period_from_offset(&wage_and_bonuses, *offset),
                None => current_salery_period(&wage_and_bonuses),
            };

            let duration_worked = duration_worked(&db, &salery_period);

            println!(
                "You have worked for: {} hours and {} minutes 
                \nYou have earned {:.2} kr.",
                duration_worked.num_hours(),
                duration_worked
                    .checked_sub(&Duration::hours(duration_worked.num_hours()))
                    .unwrap()
                    .num_minutes(),
                calculate_salery_from_period(&db, &wage_and_bonuses, salery_period)
            )
        }

        Operation::Remove { id } => {
            db.remove_shift(*id).unwrap();
            println!("succesfully deleted shift with the id of: {}", id);
        }

        Operation::Add {
            start,
            end,
            break_duration,
        } => {
            let shift_start = parse_naivedatetime_from_str(&start).unwrap();
            let shift_end = parse_naivedatetime_from_str(&end).unwrap();
            db.add_shift(shift_start, shift_end, *break_duration)
                .unwrap();
            println!(
                "Added shift that started at: {} and ended at: {}, break is: {:#?}",
                shift_start, shift_end, break_duration
            );
        }
        Operation::DropDatabase => {
            println!("This action will delete all entries in the database, meaning all data will be lost.\nAre you sure you want to continue? [y/n]");
            let mut response_buffer = String::new();
            std::io::stdin()
                .read_line(&mut response_buffer)
                .expect("couldnt read input");

            if response_buffer.trim().to_uppercase() == "Y".to_string() {
                db.drop_table();
                println!("Succesfully deleted all data");
            } else {
                println!("The data is safe!");
            }
        }
        Operation::EditShift { id, start, end } => {
            let start = match start.as_ref() {
                Some(s) => Some(parse_naivedatetime_from_str(s).unwrap()),
                None => None,
            };
            let end = match end.as_ref() {
                Some(s) => Some(parse_naivedatetime_from_str(s).unwrap()),
                None => None,
            };
            db.edit_shift(*id, &start, &end).unwrap();

            println!(
                "Edit succesfull! \n\nChanges:{}{}",
                if let Some(start) = start {
                    format!("\nshift_start = {}", start)
                } else {
                    "".to_string()
                },
                if let Some(end) = end {
                    format!("\nshift_end = {}", end)
                } else {
                    "".to_string()
                },
            );
        }
    }
}
