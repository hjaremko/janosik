use crate::trigger::Trigger;
use simsearch::SimSearch;

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

    fn frequency() -> i32 {
        40
    }

    fn contains_trigger(content: &str) -> bool {
        const TRIGGER_WORDS: &[&str] = &["nagrywa", "nagra", "absurd", "tokarczyk"];
        let mut engine: SimSearch<u32> = SimSearch::new();
        let content = content.to_lowercase();
        engine.insert(1, &content);

        for &word in TRIGGER_WORDS {
            if !engine.search(word).is_empty() {
                return true;
            }
        }

        false
    }
}
