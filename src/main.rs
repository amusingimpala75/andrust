mod args;
mod model;

use std::collections::HashMap;
use std::error::Error;

fn main() {
    match run() {
        Ok(_) => {}
        Err(e) => println!("{}", e),
    };
}

fn run() -> Result<(), Box<dyn Error>> {
    let arguments = args::Args::parse()?;

    let mut model = model::Model {
        degree: arguments.degree,
        token_type: arguments.token_type,
        freq_table: HashMap::new(),
    };

    model.train(arguments.files)?;

    let generated = model.generate(arguments.max_count)?;

    println!("{}", generated);

    Ok(())
}
