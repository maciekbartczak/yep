use std::fs::File;
use std::io::{BufRead, Read};
use std::process::Command;
use std::{env, fs};

fn main() {
    let mut run_build = true;

    let args: Vec<String> = env::args().collect();
    if let Some(flag) = try_consume_flag(args) {
        match flag.as_str() {
            "skipBuild" => run_build = false,
            _ => panic!("Unsupported flag provided: {}", flag),
        }
    }

    if run_build {
        Command::new("cargo")
            .args(["build", "--release", "--bin", "yep"])
            .output()
            .expect("failed to build yep");
        println!("Build successful!");
    }

    let programs = get_files_with_extension("./tests/programs", "yep");

    for program in programs {
        // read the expected result
        let result_filename = program.replace(".yep", ".result");
        let mut result_file = File::open(&result_filename)
            .expect(format!("Cannot open {}, make sure it exists.", result_filename).as_str());

        let mut expected_contents = String::new();
        result_file.read_to_string(&mut expected_contents).unwrap();

        let expected_lines: Vec<String> =
            expected_contents.lines().map(|s| s.to_string()).collect();

        // compile the program
        Command::new("target/release/yep")
            .arg(format!("{}", program))
            .output()
            .expect("failed to execute yep");

        // TEMP: This eventually will be integrated into the yep executable
        Command::new("bash")
            .arg("./compile.sh")
            .output()
            .expect("failed to execute compile.sh");

        let program_output = Command::new("./program")
            .output()
            .expect("failed to execute program");

        let std_output = String::from_utf8(program_output.stdout).unwrap();

        let program_lines: Vec<String> = std_output.lines().map(String::from).collect();

        println!(
            "{}... {}",
            program,
            if program_lines == expected_lines {
                "OK"
            } else {
                "FAIL"
            }
        );
        assert_eq!(expected_lines, program_lines);
    }
}
fn get_files_with_extension(directory: &str, extension: &str) -> Vec<String> {
    match fs::read_dir(directory) {
        Ok(entries) => entries
            .filter_map(|entry| {
                entry.ok().and_then(|dir_entry| {
                    let path = dir_entry.path();

                    if path.is_file() && path.extension().map_or(false, |ext| ext == extension) {
                        path.to_str().map(|s| s.to_string())
                    } else {
                        None
                    }
                })
            })
            .collect(),
        Err(e) => {
            eprintln!("Error reading directory: {}", e);
            Vec::new()
        }
    }
}

fn try_consume_flag(args: Vec<String>) -> Option<String> {
    if args.len() == 1 {
        return None;
    }

    let flag = &args[1];
    if !flag.starts_with("--") {
        panic!("Expected flag to start with '--'");
    }

    let value = &flag[2..];
    Some(value.to_string())
}
