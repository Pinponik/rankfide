use std::error::Error;
use std::fs::File;
use csv::ReaderBuilder;

fn main() -> Result<(), Box<dyn Error>> {
    let file = File::open(r"src\tabela.csv")?;
    let mut rdr = ReaderBuilder::new()
        .delimiter(b',') 
        .has_headers(false)
        .from_reader(file);

    for result in rdr.records() {
        let record = result?;
        println!("{:?}", record);
    }
    Ok(())
}
