use terminal_ui_cliclack::{Choice, PromptSession, PromptTheme};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut session = PromptSession::new(PromptTheme::default())?;
    let choices = [
        Choice::new("inspect", "Inspect / 查看", "Read-only"),
        Choice::new("quit", "Quit / 退出", ""),
    ];
    let result = session.select("Action / 操作", &choices, &"inspect");
    drop(session);
    println!("{result:?}");
    Ok(())
}
