use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};
use console::Term;
use log::{error, info};
use serde::{Deserialize, Serialize};
use simple_logger;
use std::fs::File;
use std::path::PathBuf;
use std::process::exit;

fn main() {
    simple_logger::init().unwrap();
    println!("CBCT preset selector");
    println!("--------------------");
    let mut app = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("CSV file")
                .help("CSV table with the CBCT presets per pathology and linac.\n\
                If the argument is ommited the application will look for the CSV file [data.csv] in the user local data folder.\n\
                - Windows: %USERPROFILE%\\AppData\\Local\\cbct_preset_selector\n\
                - Linux: $HOME/.local/share/cbct_preset_selector\n\
                - macOS: $HOME/Library/Application Support/cbct_preset_selector")
                .index(1)
                .required(false),
        );
    let matches = app.clone().get_matches();

    let filename_csv: PathBuf;
    let opt_filename_csv = matches.value_of("CSV file");
    match opt_filename_csv {
        None => {
            let opt_local_dir = dirs::data_local_dir();
            if let None = opt_local_dir {
                error!("Unable to determine the local data directory for the current user.");
                if let Err(err) = app.print_long_help() {
                    error!("{}", err.to_string());
                }
                wait_exit(1);
            }
            let mut pb_dir = PathBuf::from(opt_local_dir.unwrap());
            pb_dir.push("cbct_preset_selector");
            let mut pb_file = pb_dir.clone();
            pb_file.push("data.csv");
            if !pb_dir.is_dir() {
                if let Err(e) = std::fs::create_dir_all(&pb_dir) {
                    error!(
                        "Unable to create missing directory {:#?} [error message: {}]",
                        pb_dir,
                        e.to_string()
                    );
                    println!();
                    if let Err(err) = app.print_long_help() {
                        error!("{}", err.to_string());
                    }
                    wait_exit(1);
                }
                info!("Created missing directory: {:#?}", pb_dir.clone());
            }
            if !pb_file.is_file() {
                error!(
                    "Missing CSV file in {:#?}. Copy the data.csv file into this directory: {:#?}",
                    pb_file.clone(),
                    pb_dir.clone()
                );
                println!();
                if let Err(err) = app.print_long_help() {
                    error!("{}", err.to_string());
                }
                wait_exit(1);
            }
            filename_csv = pb_file;
        }
        Some(s) => {
            filename_csv = PathBuf::from(s.to_string());
        }
    }

    let res_entries = read_csv(&filename_csv);
    let mut entries = vec![];
    match res_entries {
        Ok(ve) => {
            entries = ve;
        }
        Err(e) => {
            error!("{}", e.to_string());
            wait_exit(1);
        }
    }
    let mut machines = vec![];
    for entry in &entries {
        if !machines.contains(&entry.machine) {
            machines.push(entry.machine.clone());
        }
    }
    machines.sort();

    let term = Term::stdout();
    let mut work = true;
    while work {
        if let Err(e) = find_preset(&term, &machines, &entries) {
            error!("{}", e.to_string());
            wait_exit(1);
        }
        let res_ans = question_yes_no(&term, "Do you want to obtain another preset?");
        match res_ans {
            Ok(ans) => {
                work = ans;
            }
            Err(e) => {
                error!("{}", e.to_string());
                wait_exit(1);
            }
        }
    }

    if let Err(e) = term.write_line("Bye") {
        error!("{}", e.to_string());
        wait_exit(1);
    }
}

#[derive(Debug, Clone)]
enum AppError {
    Terminal(String),
    IO(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self {
            AppError::Terminal(msg) => write!(f, "Terminal error: {}", msg),
            AppError::IO(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Entry {
    machine: String,
    pathology: String,
    preset: String,
}

impl Entry {
    fn new() -> Self {
        Self {
            machine: "".to_owned(),
            pathology: "".to_owned(),
            preset: "".to_owned(),
        }
    }
}

impl std::fmt::Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "machine: {}\npathology: {}\npreset: {}",
            &self.machine, &self.pathology, &self.preset
        )
    }
}

fn wait_exit(code: i32) {
    let term = Term::stdout();
    if code != 0 {
        if let Err(e) = term.write_line(&format!("Exit code: {}.", code)) {
            error!("{}", e.to_string());
        }
    }
    if cfg!(windows) {
        if let Err(e) = term.write_str("Press any key to continue ...") {
            error!("{}", e.to_string());
        }
    }
    let _ = term.read_line();
    exit(code);
}

fn question(term: &Term, msg: &str) -> Result<String, AppError> {
    loop {
        if let Err(e) = term.write_str(&format!("{}: ", msg)) {
            return Err(AppError::Terminal(e.to_string()));
        }
        let res_ans = term.read_line();
        if let Err(e) = res_ans {
            return Err(AppError::Terminal(e.to_string()));
        }
        let ans_str = res_ans.unwrap();
        let ans = ans_str.trim();
        if !ans.is_empty() {
            return Ok(ans.to_string());
        }
        if let Err(e) = term.write_line("No answer was given.\n") {
            return Err(AppError::Terminal(e.to_string()));
        }
    }
}

fn question_yes_no(term: &Term, msg: &str) -> Result<bool, AppError> {
    let msg_yn = msg.to_string() + "[y/n]";
    let ans = question(term, &msg_yn)?;
    let ans = ans.to_lowercase();
    let ans = ans.trim();
    if ans == "y" || ans == "yes" {
        return Ok(true);
    }
    return Ok(false);
}

fn question_with_options<T: std::fmt::Display>(
    term: &Term,
    msg: &str,
    options: &Vec<T>,
) -> Result<usize, AppError> {
    loop {
        let mut i: usize = 0;
        for opt in options {
            if let Err(e) = term.write_line(&format!("{}. {}", i + 1, opt)) {
                return Err(AppError::Terminal(e.to_string()));
            }
            i = i + 1;
        }
        if let Err(e) = term.write_str(&format!("{}: ", msg)) {
            return Err(AppError::Terminal(e.to_string()));
        }
        let res_ans = term.read_line();
        if let Err(e) = res_ans {
            return Err(AppError::Terminal(e.to_string()));
        }
        let ans_str = res_ans.unwrap();
        let res_ans_int = ans_str.parse::<usize>();
        if let Err(e) = res_ans_int {
            return Err(AppError::Terminal(e.to_string()));
        }
        let ans = res_ans_int.unwrap();
        if ans >= 1 || ans <= options.len() {
            return Ok(ans - 1);
        }
    }
}

fn read_csv(filename: &PathBuf) -> Result<Vec<Entry>, AppError> {
    let res_file = File::open(filename);
    if let Err(e) = res_file {
        return Err(AppError::IO(e.to_string()));
    }
    let mut rdr = csv::Reader::from_reader(res_file.unwrap());
    if let Err(e) = rdr.headers() {
        return Err(AppError::IO(e.to_string()));
    }
    let mut v = vec![];
    for result in rdr.records() {
        if let Err(e) = result {
            return Err(AppError::IO(e.to_string()));
        }
        let record = result.unwrap();
        if !record.is_empty() {
            if record.len() != 3 {
                return Err(AppError::IO(format!(
                    "Expected each line to contain 3 columns but instead were detected: {}",
                    record.len()
                )));
            }
            let entry = Entry {
                machine: (&record[0]).to_string(),
                pathology: (&record[1]).to_string(),
                preset: (&record[2]).to_string(),
            };
            v.push(entry);
        }
    }
    return Ok(v);
}

fn find_preset(term: &Term, machines: &Vec<String>, entries: &Vec<Entry>) -> Result<(), AppError> {
    let mut result = Entry::new();
    if let Err(e) = term.write_line("\nMachine:") {
        return Err(AppError::Terminal(e.to_string()));
    }
    let machine_idx = question_with_options(&term, "Select", &machines)?;
    result.machine = machines.get(machine_idx).unwrap().clone();

    let mut filtered_entries = vec![];
    for entry in entries {
        if entry.machine == result.machine {
            filtered_entries.push(entry.clone());
        }
    }
    filtered_entries.sort_by_key(|entry| (*entry).pathology.clone());
    let mut filtered_pathologies = vec![];
    for entry in &filtered_entries {
        filtered_pathologies.push(entry.pathology.clone());
    }
    if let Err(e) = term.write_line("\nPathology:") {
        return Err(AppError::Terminal(e.to_string()));
    }
    let pathology_idx = question_with_options(&term, "Select", &filtered_pathologies)?;
    result.pathology = filtered_entries
        .get(pathology_idx)
        .unwrap()
        .pathology
        .clone();
    result.preset = filtered_entries.get(pathology_idx).unwrap().preset.clone();
    assert_eq!(
        &result.pathology,
        filtered_pathologies.get(pathology_idx).unwrap()
    );

    println!("\nSelected parameters and preset:\n{}\n", result);
    return Ok(());
}
