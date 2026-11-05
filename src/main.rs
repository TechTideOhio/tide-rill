use std::{env, fs, process::ExitCode};

const USAGE: &str =
    "usage: tide-rill eval <input.json>\n       tide-rill gate <input.json> <thresholds.json>";

fn read(path: &str) -> Result<String, String> {
    fs::read_to_string(path).map_err(|_| "could not read input".to_owned())
}

fn to_json<T: serde::Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_string_pretty(value).map_err(|_| "could not serialize report".to_owned())
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 && (args[1] == "--help" || args[1] == "-h") {
        println!("{USAGE}\nOffline retrieval, faithfulness and gate scoring. Output contains query IDs; protect it.\nExit codes: 0 pass/evaluated, 1 gate failed, 2 input or usage error.");
        return ExitCode::SUCCESS;
    }
    let outcome = match (args.get(1).map(String::as_str), args.len()) {
        (Some("eval"), 3) => read(&args[2])
            .and_then(|i| tide_rill::evaluate_json(&i))
            .and_then(|r| to_json(&r))
            .map(|out| (out, true)),
        (Some("gate"), 4) => read(&args[2])
            .and_then(|i| read(&args[3]).map(|t| (i, t)))
            .and_then(|(i, t)| tide_rill::gate_json(&i, &t))
            .and_then(|v| to_json(&v).map(|out| (out, v.passed))),
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    match outcome {
        Ok((output, passed)) => {
            println!("{output}");
            if passed {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(2)
        }
    }
}
