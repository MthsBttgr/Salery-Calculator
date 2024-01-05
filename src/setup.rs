use std::{fs, io::BufWriter, path::Path};

use crate::wage_bonuses::{Bonus, Period, WageAndBonuses};

fn get_user_input(query: &str) -> String {
    let mut buffer = String::new();

    println!("{}", query);

    loop {
        match std::io::stdin().read_line(&mut buffer) {
            Ok(_) => break,
            Err(_) => println!("Couldn't read input, try again or press ctrl + c to exit program"),
        };
    }

    buffer.trim().to_string()
}

fn get_parsed_input<T: std::str::FromStr>(query: &str, err_msg: &str) -> T {
    loop {
        let input = get_user_input(query);

        match input.trim().parse() {
            Ok(val) => return val,
            Err(_) => {
                println!("{}\nTry again or press ctrl + c to exit program", err_msg);
                continue;
            }
        }
    }
}

pub fn setup_wage_bonuses_if_missing<P: AsRef<Path>>(path: P) -> Option<()> {
    let config_file = path;

    if Path::new(config_file.as_ref()).exists() {
        return None;
    }

    println!("You are missing: {}. This file is needed for the program to function. Now running setup...", config_file.as_ref().file_name().unwrap().to_str().unwrap());

    let base_rate: f64 = get_parsed_input(
        "\nPlease input your hourly wage. Decimals should be seperated by a period",
        "Something went wrong.",
    );

    println!("\nYour salery period is the period of time your salery is calculated from. 
This can be from the first day of the month to the last day, or it can be special.
Special means that it runs from some date during the middle of the month to just before that date the following month. 
Example:
If the salery period starts the 15th of april, then the end would be the 14th of may
start = 15
end = 14");

    let inp = get_user_input("\nIs your salery period special? [y/n]").to_uppercase();

    let period = 'outer: loop {
        if inp == "Y" {
            loop {
                let start: u32 = get_parsed_input(
                    "\nWhat is start of your salery period?",
                    "Please input a number greater than 0",
                );
                let end = if start < 29 && start > 1 {
                    start - 1
                } else {
                    println!("Start should be between the 2nd and the 29th");
                    continue;
                };
                break 'outer Period::Special {
                    start_day: start,
                    end_day: end,
                };
            }
        } else if inp == "N" {
            break Period::Month;
        } else {
            println!("Please input one of the allowed responses");
            continue;
        }
    };

    println!(
        "\nA general bonus is a bonus that is applied every day during certain hours.
Example
You work night shift and get an extra 23.42 for working at night
start = 00:00
end = 06:00
bonus_pr_hour = 23.42"
    );

    let mut general_time_periods = Vec::new();
    loop {
        let inp = get_user_input("\nDo you want to add a general bonus? [y/n]").to_uppercase();
        if inp == "Y" {
            general_time_periods.push(get_bonus())
        } else if inp == "N" {
            break;
        } else {
            println!("Please input a valic character");
        }
    }

    println!(
        "\nA day-of-the-week bonus is a bonus is a bonus that is applied on certain days during certain hours.
Example
You work during the weekend and get an extra 10.47 for working on sundays from 10:00 to 17:00
start = 10:00
end = 17:00
bonus_pr_hour = 10.47
days = sunday

If a bonus is applied on multiple days, seperate them by a comma"
    );

    let mut day_of_week_rates = Vec::new();

    loop {
        let inp =
            get_user_input("\nDo you want to add a day-of-the-week bonus? [y/n]").to_uppercase();
        if inp == "Y" {
            day_of_week_rates.push(get_day_of_week_bonus())
        } else if inp == "N" {
            break;
        } else {
            println!("Please input a valic character");
        }
    }

    let wage_and_bonuses =
        WageAndBonuses::new(base_rate, period, general_time_periods, day_of_week_rates);

    let file = fs::File::create(config_file).expect("Couldnt create file");
    let writer = BufWriter::new(file);

    serde_json::to_writer(writer, &wage_and_bonuses)
        .expect("couldnt parse the given inputs to json");

    println!("\nSetup is done. If you gave any wrong input, you can edit everything in the json document found in: {}", std::env::current_exe().unwrap().display());

    let _ = get_user_input("\nPress enter to continue the rest of the program");
    Some(())
}

fn get_bonus() -> Bonus {
    let bonus_pr_hour: f64 = get_parsed_input("Please input the added bonus pr hour ie. the amount that will be added to your base salery", "");
    let start: String = get_user_input(
        "Please input what time of day the bonus starts being applied in the format: HH:MM",
    );
    let end: String = get_user_input(
        "Please input what time of day the bonus stops to being applied in the format: HH:MM",
    );

    Bonus::new(bonus_pr_hour, start, end, None)
}

fn get_day_of_week_bonus() -> Bonus {
    let mut bonus = get_bonus();
    let days: String = get_user_input("Please input the days where this bonus applies. The days should be written in english and seperated by a comma");

    let days = days.split(",").map(|day| day.trim().to_string()).collect();

    bonus.add_days(days);
    bonus
}
