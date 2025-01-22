use regex::Regex;
use std::process::Command;
use std::time::Instant;
use xlsxwriter::Workbook;

struct TestMetrics {
    name: String,
    duration: f64,
    result: String,
    processed_lines: Vec<String>,
}

struct TestProcessor {
    workbook: Workbook,
    search_word: String,
    project_path: std::path::PathBuf,
}

impl TestProcessor {
    fn new(
        project_path: std::path::PathBuf,
        excel_path: &str,
        search_word: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(TestProcessor {
            workbook: Workbook::new(excel_path)?,
            search_word: search_word.to_string(),
            project_path,
        })
    }

    fn write_metrics(&mut self, metrics: &[TestMetrics]) -> Result<(), Box<dyn std::error::Error>> {
        let mut sheet = self.workbook.add_worksheet(None)?;

        // Write headers
        sheet.write_string(0, 0, "Test Name", None)?;
        sheet.write_string(0, 1, "Duration (s)", None)?;
        sheet.write_string(0, 2, "Result", None)?;
        sheet.write_string(0, 3, "Processed Lines", None)?;
        sheet.write_string(0, 4, "Timestamp", None)?;

        // Write data
        for (i, metric) in metrics.iter().enumerate() {
            let row = (i + 1) as u32;

            sheet.write_string(row, 0, &metric.name, None)?;
            sheet.write_number(row, 1, metric.duration, None)?;
            sheet.write_string(row, 2, &metric.result, None)?;
            sheet.write_string(row, 3, &metric.processed_lines.join("; "), None)?;

            // sheet.write_datetime(row, 4, timestamp, None)?;
        }

        Ok(())
    }

    fn run_tests(&mut self) -> Result<Vec<TestMetrics>, Box<dyn std::error::Error>> {
        let output = Command::new("cargo")
            .current_dir(&self.project_path)
            .arg("test")
            .arg("--")
            .arg("--nocapture")
            .output()?;

        let stdout = String::from_utf8(output.stdout)?;
        self.process_test_output(&stdout)
    }

    fn process_test_output(
        &self,
        output: &str,
    ) -> Result<Vec<TestMetrics>, Box<dyn std::error::Error>> {
        let test_pattern = Regex::new(r"test (.*) \.\.\. (\w+)")?;
        let mut metrics = Vec::new();

        for line in output.lines() {
            if let Some(caps) = test_pattern.captures(line) {
                let start = Instant::now();
                let test_name = caps.get(1).unwrap().as_str().to_string();
                let result = caps.get(2).unwrap().as_str().to_string();

                let processed_lines = self.process_test_lines(output, &test_name);

                metrics.push(TestMetrics {
                    name: test_name,
                    duration: start.elapsed().as_secs_f64(),
                    result,
                    processed_lines,
                });
            }
        }

        Ok(metrics)
    }

    fn process_test_lines(&self, output: &str, test_name: &str) -> Vec<String> {
        let mut processed_lines = Vec::new();

        for line in output.lines() {
            if line.contains(&self.search_word) {
                // Replace this with your custom processing logic
                let processed_line = self.process_line(line);
                processed_lines.push(processed_line);
            }
        }

        processed_lines
    }

    fn process_line(&self, line: &str) -> String {
        // Implement your custom processing logic here
        line.to_string()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut processor = TestProcessor::new(
        std::env::current_dir().unwrap(),
        "test_metrics.xlsx",
        "error",
    )?;

    let metrics = processor.run_tests()?;
    processor.write_metrics(&metrics)?;
    Ok(())
}
