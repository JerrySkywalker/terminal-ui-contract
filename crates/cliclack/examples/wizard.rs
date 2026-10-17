use terminal_ui_cliclack::{Choice, PromptOutcome, PromptSession, PromptTheme};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut session = PromptSession::new(PromptTheme::default())?;
    let choices = [
        Choice::new("en", "English", ""),
        Choice::new("zh", "中文", ""),
    ];
    let selected = session.multiselect("Languages", &choices, &["en"], false);
    if !matches!(selected, PromptOutcome::Submitted(_)) {
        return Ok(());
    }
    let name = session.input("Name", "", false);
    let confirm = session.confirm("Submit draft?", false);
    drop(session);
    println!("{selected:?} {name:?} {confirm:?}");
    Ok(())
}
