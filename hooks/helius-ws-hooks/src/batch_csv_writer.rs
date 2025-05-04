use csv::WriterBuilder;
use serde::{Deserialize, Serialize};
use std::{fs::File, path::Path, time::Instant};

pub struct BatchCsvWriter {
    base_dir: String,
    start: Instant,
    record_interval: u64,
    current_record_interval: u64,
    seconds_interval: u64,
    csv_writer: csv::Writer<File>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BatchCsvRecord {
    pub action: String,
    pub slot: u64,
    pub signature: String,
    pub mint_pk: Option<String>,
    pub user_pk: Option<String>,

    pub name: Option<String>,
    pub symbol: Option<String>,
    pub uri: Option<String>,

    pub amount_buy: Option<u64>,
    pub max_sol_cost: Option<u64>,

    pub amount_sell: Option<u64>,
    pub min_sol_output: Option<u64>,

    pub bonding_curve: Option<String>,
    pub associated_bonding_curve: Option<String>,
}

impl BatchCsvWriter {
    pub fn new(
        base_dir: String,
        record_interval: u64,
        seconds_interval: u64,
    ) -> anyhow::Result<Self> {
        assert!(Path::new(&base_dir).exists());

        let file = Self::create_csv_file(&base_dir)?;

        Ok(Self {
            start: Instant::now(),
            record_interval,
            current_record_interval: 0,
            seconds_interval,
            base_dir,
            csv_writer: WriterBuilder::new().from_writer(file),
        })
    }

    pub fn write(&mut self, record: BatchCsvRecord) -> anyhow::Result<()> {
        self.current_record_interval += 1;
        self.csv_writer.serialize(record)?;

        // check every 100 records
        if self.current_record_interval > self.record_interval {
            if self.start.elapsed().as_secs() > self.seconds_interval {
                self.reset()?;
            }
            self.current_record_interval = 0;
        }

        Ok(())
    }

    // New helper function to create a CSV file
    fn create_csv_file(base_dir: &str) -> anyhow::Result<File> {
        let file_name = format!("{}.csv", chrono::Local::now().format("%Y%m%d-%H%M%S"));
        let file_path = Path::new(base_dir).join(file_name);
        let file = File::create(file_path)?;
        Ok(file)
    }

    fn reset(&mut self) -> anyhow::Result<()> {
        let _ = self.csv_writer.flush();

        self.start = Instant::now();
        self.current_record_interval = 0;

        let file = Self::create_csv_file(&self.base_dir)?;
        self.csv_writer = WriterBuilder::new().from_writer(file);

        Ok(())
    }

    // get the current record count
    pub fn current_count(&self) -> u64 {
        self.current_record_interval
    }
}
