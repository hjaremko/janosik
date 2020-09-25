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

pub fn add_protip_message(_content: &str, task: &str) -> String {
    format!("Dodano protip do `{}`", task)
}

pub fn all_tasks_message(all_protips: &[String]) -> String {
    format!("Dostępne listy protipów: `{:?}`", all_protips)
}

pub fn invalid_protip_id_message() -> String {
    "Musisz podać numer protipa do usunięcia".to_string()
}

pub fn delete_protip_message(protip_id: &u32) -> String {
    format!("Usunięto protip nr {}", protip_id)
}
