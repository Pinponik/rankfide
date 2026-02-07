use std::error::Error;
use std::fs::File;
use csv::ReaderBuilder;

struct ProbabilityRecord {
    difference_from: u16,
    difference_to: u16,
    probability_when_me_bigger: u8,
    probability_when_me_smaller: u8,
}

impl ProbabilityRecord {
    fn new() -> Self {
        Self {
            difference_from: 0,
            difference_to: 0,
            probability_when_me_bigger: 0,
            probability_when_me_smaller: 0,
        }
    }
    
    fn new_from(difference_from: u16, difference_to: u16, probability_when_me_bigger: u8, probability_when_me_smaller: u8) -> Self {
        Self {
            difference_from,
            difference_to,
            probability_when_me_bigger,
            probability_when_me_smaller,
        }
    }
    
    fn from_csv_record(record: &csv::StringRecord) -> Result<Self, Box<dyn Error>> {
        if record.len() != 4 {
            return Err("Invalid record length".into());
        }
        Ok(Self {
            difference_from: record[0].parse()?,
            difference_to: record[1].parse()?,
            probability_when_me_bigger: record[2].parse()?,
            probability_when_me_smaller: record[3].parse()?,
        })
    }
}

fn load_from_csv(file: &str) -> Result<Vec<ProbabilityRecord>, Box<dyn Error>> {
    let file = File::open(file)?;
    let mut rdr = ReaderBuilder::new()
        .delimiter(b',') 
        .has_headers(false)
        .from_reader(file);

    let mut records = Vec::new();
    for result in rdr.records() {
        let record = result?;
        records.push(ProbabilityRecord::from_csv_record(&record)?);
    }
    Ok(records)
}

fn main() -> Result<(), Box<dyn Error>> {
    let probability = load_from_csv(r"src\table.csv")?;
    
}
