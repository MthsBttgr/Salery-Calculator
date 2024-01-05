use clap::{command, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    ///What the program should do
    #[command(subcommand)]
    operation: Option<Operation>,
}

impl Cli {
    pub fn operation(&self) -> Option<&Operation> {
        self.operation.as_ref()
    }
}

#[derive(Subcommand, Clone, Debug)]
pub enum Operation {
    /// adds shift to database
    Add {
        /// YY-MM-DD hh:mm
        start: String,
        /// YY-MM-DD hh:mm
        end: String,
        /// subtracts the break from the end of the shift
        /// the break should be defined in whole minutes
        #[arg(short = 'b', long = "break")]
        break_duration: Option<i64>,
    },
    /// Removes shift from the database
    Remove { id: u32 },
    /// Lists all shifts for this month
    List {
        /// List all documented shifts ever
        #[arg(short, long)]
        all: bool,
        /// Sort chronologically, showing most recent first
        #[arg(short, long)]
        sort: bool,
        #[arg(short, long)]
        offset: Option<u32>,
    },
    /// Calculates this months salery
    Calculate {
        /// calculate the salery period that is equal to the current "minus" the offset
        #[arg(short, long)]
        offset: Option<u32>,
    },
    /// Deletes the database
    DropDatabase,
    /// Edit a shift choosen from it's id, takes atleast one other argument
    EditShift {
        /// Shift id in the database, call "list" to see shifts in the database
        id: u32,
        /// change the shift's start to this
        #[arg(short, long)]
        start: Option<String>,
        /// change the shift's end to this
        #[arg(short, long)]
        end: Option<String>,
    },
}
