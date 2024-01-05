use crate::time::SQLformat;
use std::path::Path;

use chrono::{Duration, NaiveDateTime};
use sqlite::{self, Connection, Error, Statement};

pub struct Database {
    connection: Connection,
    table: String,
}

impl Database {
    pub fn open_or_create_db<P: AsRef<Path>>(path: P, table_name: &str) -> Self {
        let db = Connection::open(path).expect("Couldnt open/create connection");

        let query = format!(
            "create table if not exists {}(
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                shift_start DATETIME NOT NULL,
                shift_end DATETIME NOT NULL
            )",
            table_name
        );

        db.execute(query).expect("couldnt execute statement");

        Database {
            connection: db,
            table: table_name.to_owned(),
        }
    }

    pub fn drop_table(&self) {
        let query = format!("drop table if exists {}", self.table);

        self.connection.execute(query).unwrap();
    }

    pub fn edit_shift(
        &self,
        shift_id: u32,
        start: Option<NaiveDateTime>,
        end: Option<NaiveDateTime>,
    ) -> Result<(), Error> {
        if let (Some(s), Some(e)) = (&start, &end) {
            if s > e {
                panic!("The end of the shift should be after the start");
            }
        } else if start.is_none() && end.is_none() {
            panic!("Edit the start and/or the end of the shift");
        }

        let mut query = format!("update {} set", self.table);

        match start {
            Some(ndt) => query = format!("{} shift_start = {:#?},", query, ndt.sql_format()),
            None => {}
        }

        match end {
            Some(ndt) => query = format!("{} shift_end = {:#?},", query, ndt.sql_format()),
            None => {}
        }

        // remove the trailing comma
        query.pop();

        query = format!("{} where id = {}", query, shift_id);

        self.connection.execute(query)
    }

    pub fn remove_shift(&self, shift_id: u32) -> Result<(), Error> {
        self.connection.execute(format!(
            "delete from {} where id = {}",
            self.table, shift_id
        ))
    }

    pub fn add_shift(
        &self,
        start: NaiveDateTime,
        mut end: NaiveDateTime,
        break_duration: Option<i64>,
    ) -> Result<(), Error> {
        if start > end {
            panic!("the end of the shift should be after the start");
        }

        if let Some(dur) = break_duration {
            end = end
                .checked_sub_signed(Duration::minutes(dur))
                .expect("break is out of range");
        }

        self.connection.execute(format!(
            "INSERT into {} (shift_start, shift_end) VALUES ({:#?}, {:#?})",
            self.table,
            start.sql_format(),
            end.sql_format()
        ))
    }

    pub fn table(&self) -> &str {
        &self.table
    }

    pub fn prepare<T: AsRef<str>>(&self, statement: T) -> sqlite::Result<Statement<'_>> {
        self.connection.prepare(statement)
    }
}

#[cfg(test)]
mod tests {

    // this test needs to be redisigned, along with database struct

    // #[test]
    // fn insert_into_db() {
    //     let db = Database::open_or_create_db("/DB.db");

    //     let date = NaiveDate::from_ymd_opt(2023, 10, 30).unwrap();
    //     let time = NaiveTime::from_hms_opt(23, 0, 0).unwrap();
    //     let start = NaiveDateTime::new(date, time);

    //     let date = NaiveDate::from_ymd_opt(2023, 10, 31).unwrap();
    //     let time = NaiveTime::from_hms_opt(3, 0, 0).unwrap();
    //     let end = NaiveDateTime::new(date, time);

    //     db.add_shift(start, end).unwrap();
    // }
}
