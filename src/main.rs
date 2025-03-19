use rusqlite::Result;

mod database;
use database::start_db;
fn main() -> Result<()> {
    let result = start_db();
    result
    
}

