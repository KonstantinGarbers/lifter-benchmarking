use regex::Regex;
use std::process::{Command, Output};
use std::time::Instant;
use xlsxwriter::Workbook;

#[derive(Debug, Clone)]
struct TestMetrics {
    name: String,
    duration: f64,
    instructions: usize,
    blocks: usize,
}

struct TestProcessor {
    workbook: Workbook,
    project_path: std::path::PathBuf,
}

impl TestProcessor {
    fn new(project_path: std::path::PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(TestProcessor {
            workbook: Workbook::new(EXCEL_PATH)?,
            project_path,
        })
    }

    fn write_metrics(&mut self, metrics: &[TestMetrics]) -> Result<(), Box<dyn std::error::Error>> {
        let mut sheet = self.workbook.add_worksheet(None)?;

        // Write headers
        sheet.write_string(0, 0, "Test Name", None)?;
        sheet.write_string(0, 1, "Duration (s)", None)?;
        sheet.write_string(0, 2, "Blocks", None)?;
        sheet.write_string(0, 3, "Instructions", None)?;

        // Write data
        for (i, metric) in metrics.iter().enumerate() {
            let row = (i + 1) as u32;
            let cleaned_test_name = metric.name.split("::").last().unwrap();
            sheet.write_string(row, 0, &cleaned_test_name, None)?;
            sheet.write_number(row, 1, metric.duration, None)?;
            sheet.write_number(row, 2, metric.blocks as f64, None)?;
            sheet.write_number(row, 3, metric.instructions as f64, None)?;
        }

        Ok(())
    }

    fn get_test_list(&self) -> Result<Output, std::io::Error> {
        Command::new("cargo")
            .current_dir(self.project_path.clone())
            .arg("test")
            .arg("--")
            .arg("--list")
            .output()
    }

    fn run_single_test(&self, test_name: &str) -> String {
        let cmd = Command::new("cargo")
            .current_dir(self.project_path.clone())
            .args([
                "test",
                "--",
                "--exact",
                test_name,
                "--format=terse",
                "--nocapture",
            ])
            .output()
            .expect("Failed to run test");

        String::from_utf8_lossy(&cmd.stdout).to_string()
    }

    fn process_tests(&self) -> Result<Vec<TestMetrics>, Box<dyn std::error::Error>> {
        let mut metrics = Vec::new();

        let test_list = self.get_test_list()?.stdout;
        let test_list = String::from_utf8_lossy(&test_list);
        let tests = test_list
            .lines()
            .filter(|line| line.contains("test"))
            .map(|line| line.trim().strip_suffix(": test").unwrap_or(line.trim()))
            .filter(|line| line.ends_with("1"))
            .collect::<Vec<_>>();

        // Used to retrieve the block and instruction count from a string as below
        // Blocks: 1, Instructions: 6
        let re =
            Regex::new(r"Blocks: (?P<block_count>\d+), Instructions: (?P<instruction_count>\d+)")
                .unwrap();
        for test in tests {
            let test_name = test.split_whitespace().next().unwrap();
            let start = Instant::now();
            let output = self.run_single_test(test_name);
            let test_name = test_name.strip_suffix("_1").unwrap_or(test_name);
            let duration = start.elapsed().as_secs_f64();
            if let Some(captures) = re.captures(&output) {
                println!("Processing test: {}", test_name);
                let blocks: usize = captures
                    .name("block_count")
                    .unwrap()
                    .as_str()
                    .parse()
                    .unwrap();

                let instructions: usize = captures
                    .name("instruction_count")
                    .unwrap()
                    .as_str()
                    .parse()
                    .unwrap();

                let metric = TestMetrics {
                    name: test_name.to_string(),
                    duration,
                    instructions,
                    blocks,
                };
                metrics.push(metric);
            } else {
                continue;
            }
        }

        Ok(metrics)
    }

    fn combine_metrics(&self, metrics: &[Vec<TestMetrics>]) -> Vec<TestMetrics> {
        let mut combined_metrics: Vec<TestMetrics> = Vec::new();
        for test_metrics in metrics {
            for metric in test_metrics {
                let existing_metric = combined_metrics.iter_mut().find(|m| m.name == metric.name);
                match existing_metric {
                    Some(existing_metric) => {
                        existing_metric.duration += metric.duration;
                        existing_metric.instructions += metric.instructions;
                        existing_metric.blocks += metric.blocks;
                    }
                    None => {
                        combined_metrics.push(metric.clone());
                    }
                }
            }
        }

        combined_metrics
            .iter()
            .map(|m| TestMetrics {
                name: m.name.clone(),
                duration: m.duration / metrics.len() as f64,
                instructions: m.instructions / metrics.len(),
                blocks: m.blocks / metrics.len(),
            })
            .collect()
    }
}

const EXCEL_PATH: &str = "test_metrics.xlsx";
const PROJECT_NAME: &str = "aarch64-air-lifter"; // Replace with the actual project folder name
const WARMUP_RUNS: usize = 5;
const TEST_RUNS: usize = 20;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = std::env::current_dir()?;
    let parent_dir = current_dir
        .parent()
        .ok_or("Failed to get parent directory")?;

    // set variables
    let project_path = parent_dir.join(PROJECT_NAME);

    if !project_path.exists() {
        return Err(format!("Project folder '{}' does not exist", PROJECT_NAME).into());
    }

    let mut processor = TestProcessor::new(project_path)?;

    println!("Total warmup runs: {}", WARMUP_RUNS);
    for i in 0..WARMUP_RUNS {
        println!("Warmup run: {}", i + 1);
        processor.process_tests()?;
    }

    let mut benchmark_metrics = Vec::new();
    println!("Total test runs: {}", TEST_RUNS);
    for i in 0..TEST_RUNS {
        println!("Test run: {}", i);
        let metrics = processor.process_tests()?;
        benchmark_metrics.push(metrics);
    }
    let metrics = processor.combine_metrics(&benchmark_metrics);
    processor.write_metrics(&metrics)?;
    Ok(())
}
