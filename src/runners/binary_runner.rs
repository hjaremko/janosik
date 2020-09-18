use crate::runners::runner_error::RunnerError;
use serenity::futures::io::Error;
use std::fs;
use std::fs::File;
use std::io::{BufReader, Read};
use std::process::{Child, ChildStdout, Command, Stdio};
use std::time::Duration;
use tracing::{error, info};
use wait_timeout::ChildExt;

type RunnerResult = Result<String, RunnerError>;

pub struct BinaryRunner {}

impl BinaryRunner {
    pub(crate) fn run(program_name: &str, input: &str) -> RunnerResult {
        if input.is_empty() {
            return Err(RunnerError::NoInput);
        }

        let file = BinaryRunner::create_tmp_file(input)?;
        let mut process = BinaryRunner::find_binary(program_name, file)?;

        info!("Program {} started", program_name);
        let status_code = BinaryRunner::wait_timeout(&mut process)?;
        info!("{} returned {:?}", program_name, status_code);

        if status_code != 0 {
            error!("{} crashed!", program_name);
            return Err(RunnerError::Crash);
        }

        BinaryRunner::read_output(process)
    }

    fn create_tmp_file(input: &str) -> Result<File, RunnerError> {
        const TMP_FILENAME: &str = "tmp.txt";

        if fs::write(TMP_FILENAME, input).is_err() {
            return Err(RunnerError::Other("cannot write to tmp file".to_string()));
        }

        match File::open(TMP_FILENAME) {
            Ok(file) => Ok(file),
            Err(_) => Err(RunnerError::Other("cannot open tmp file".to_string())),
        }
    }

    fn find_binary(program_name: &str, file: File) -> Result<Child, RunnerError> {
        let child = BinaryRunner::spawn_process(program_name, file);

        if child.is_err() {
            return Err(RunnerError::NotFound);
        }

        Ok(child.unwrap())
    }

    fn spawn_process(program_name: &str, file: File) -> Result<Child, Error> {
        Command::new(format!("bin/{}", program_name))
            .stdin(Stdio::from(file))
            .stdout(Stdio::piped())
            .spawn()
    }

    fn wait_timeout(child: &mut Child) -> Result<i32, RunnerError> {
        let timeout = Duration::from_secs(30);

        let status_code = match child.wait_timeout(timeout) {
            Ok(c) => c,
            Err(_) => return Err(RunnerError::Timeout),
        };

        match status_code {
            None => {
                child.kill().expect("Command wasn't running");
                Err(RunnerError::Timeout)
            }
            Some(status) => Ok(status.code().unwrap()),
        }
    }

    fn read_output(child: Child) -> RunnerResult {
        let mut reader = BufReader::new(BinaryRunner::get_stdout(child).unwrap());
        let mut output = String::new();

        reader.read_to_string(&mut output).unwrap();

        if output.is_empty() {
            return Err(RunnerError::NoOutput);
        }

        Ok(format!("```\n{}\n```", output))
    }

    fn get_stdout(child: Child) -> Result<ChildStdout, RunnerError> {
        child
            .stdout
            .ok_or_else(|| RunnerError::Other("Could not capture standard output.".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::runners::binary_runner::BinaryRunner;
    use crate::runners::runner_error::RunnerError;

    #[test]
    fn should_return_error_on_empty_input() {
        let out = BinaryRunner::run("not found", "");

        assert!(out.is_err());
        assert_eq!(out.unwrap_err(), RunnerError::NoInput);
    }
}
