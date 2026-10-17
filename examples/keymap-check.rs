use std::error::Error;
use terminal_ui_contract::keymap::{KeymapRegistry, ResolveContext};
fn main() -> Result<(), Box<dyn Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("provide one explicit keymap file path")?;
    let text = std::fs::read_to_string(path)?;
    let map = KeymapRegistry::from_toml(&text, vec![], vec![])?;
    for hint in map.effective_hints(&ResolveContext::normal(&[])) {
        println!(
            "{}: {} [{}; {:?}]",
            hint.keys, hint.label, hint.action_id, hint.origin
        );
    }
    Ok(())
}
