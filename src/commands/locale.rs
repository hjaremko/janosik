const CRASH_MESSAGE: &str = "wyjebał się, sprawdź poprawność wejścia";
const TIMEOUT_MESSAGE: &str = "działał zbyt długo, sprawdź poprawność wejścia";
const NOT_FOUND_MESSAGE: &str = "Nie znaleziono";
const NO_OUTPUT_MESSAGE: &str = "nic nie wypisał, sprawdź poprawność wejścia";
const NO_INPUT_MESSAGE: &str = "Nie podano wejścia, pamiętaj o ```";

pub fn crash_message(program_name: &str) -> String {
    format!("`{}` {}", program_name, CRASH_MESSAGE)
}

pub fn timeout_message(program_name: &str) -> String {
    format!("`{}` {}", program_name, TIMEOUT_MESSAGE)
}

pub fn not_found_message(program_name: &str) -> String {
    format!("{} ` {} `", NOT_FOUND_MESSAGE, program_name)
}

pub fn no_output_message(program_name: &str) -> String {
    format!("`{}` {}", program_name, NO_OUTPUT_MESSAGE)
}

pub fn no_input_message() -> String {
    NO_INPUT_MESSAGE.to_string()
}
