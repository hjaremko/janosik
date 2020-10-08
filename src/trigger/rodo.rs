use crate::trigger::Trigger;

// todo: read triggers from database
pub struct Rodo;

impl Trigger for Rodo {
    fn message() -> String {
        r#"W związku z zapytaniem, informuję, iż problem z nagrywaniem wiąże się z
naruszeniem RODO wobec STUDENTÓW.

Dla mnie (na tę chwilę) jest to o tyle niezrozumiałe (mimo konkretnych
"ściśle prawniczych" argumentów podniesionych przez Panią Tokarczyk), że
nawet gdyby wszyscy studenci PROSILI o nagrywanie zajęć, to zgadzając się
na to, naruszamy RODO.

Absurd!!!
p.niemiec"#
            .to_string()
    }

    fn name() -> String {
        "RODO notice".to_string()
    }

    fn contains_trigger(content: &str) -> bool {
        const TRIGGER_WORDS: &[&str] = &["nagrywa", "nagra", "absurd", "tokarczyk"];
        let content = content.to_lowercase();

        for &word in TRIGGER_WORDS {
            if content.contains(word) {
                return true;
            }
        }

        false
    }
}
