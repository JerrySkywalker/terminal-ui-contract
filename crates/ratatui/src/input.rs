//! Lossless supported event translation; unsupported enhanced keys remain explicit.
use crossterm::event as ct;
use terminal_ui_contract::input::*;

pub fn modifiers(value: ct::KeyModifiers) -> Modifiers {
    let mut result = Modifiers::NONE;
    for (from, to) in [
        (ct::KeyModifiers::SHIFT, Modifiers::SHIFT),
        (ct::KeyModifiers::CONTROL, Modifiers::CONTROL),
        (ct::KeyModifiers::ALT, Modifiers::ALT),
        (ct::KeyModifiers::SUPER, Modifiers::SUPER),
        (ct::KeyModifiers::HYPER, Modifiers::HYPER),
        (ct::KeyModifiers::META, Modifiers::META),
    ] {
        if value.contains(from) {
            result = result.union(to);
        }
    }
    result
}

pub fn normalize(event: ct::Event) -> InputEvent {
    match event {
        ct::Event::Key(key) => InputEvent::Key(KeyEvent {
            chord: KeyChord::new(
                match key.code {
                    ct::KeyCode::Char(c) => KeyCode::Char(c),
                    ct::KeyCode::Enter => KeyCode::Enter,
                    ct::KeyCode::Esc => KeyCode::Escape,
                    ct::KeyCode::Backspace => KeyCode::Backspace,
                    ct::KeyCode::Delete => KeyCode::Delete,
                    ct::KeyCode::Insert => KeyCode::Insert,
                    ct::KeyCode::Tab => KeyCode::Tab,
                    ct::KeyCode::BackTab => KeyCode::BackTab,
                    ct::KeyCode::Up => KeyCode::Up,
                    ct::KeyCode::Down => KeyCode::Down,
                    ct::KeyCode::Left => KeyCode::Left,
                    ct::KeyCode::Right => KeyCode::Right,
                    ct::KeyCode::PageUp => KeyCode::PageUp,
                    ct::KeyCode::PageDown => KeyCode::PageDown,
                    ct::KeyCode::Home => KeyCode::Home,
                    ct::KeyCode::End => KeyCode::End,
                    ct::KeyCode::F(n) => KeyCode::Function(n),
                    _ => KeyCode::Unsupported,
                },
                modifiers(key.modifiers),
            )
            .normalized(),
            kind: match key.kind {
                ct::KeyEventKind::Press => InputKind::Press,
                ct::KeyEventKind::Repeat => InputKind::Repeat,
                ct::KeyEventKind::Release => InputKind::Release,
            },
        }),
        ct::Event::Mouse(mouse) => InputEvent::Mouse(MouseEvent {
            column: mouse.column,
            row: mouse.row,
            modifiers: modifiers(mouse.modifiers),
            kind: match mouse.kind {
                ct::MouseEventKind::Down(button) => MouseKind::Press(mouse_button(button)),
                ct::MouseEventKind::Up(button) => MouseKind::Release(mouse_button(button)),
                ct::MouseEventKind::Drag(button) => MouseKind::Drag(mouse_button(button)),
                ct::MouseEventKind::Moved => MouseKind::Move,
                ct::MouseEventKind::ScrollUp => MouseKind::ScrollUp,
                ct::MouseEventKind::ScrollDown => MouseKind::ScrollDown,
                ct::MouseEventKind::ScrollLeft => MouseKind::ScrollLeft,
                ct::MouseEventKind::ScrollRight => MouseKind::ScrollRight,
            },
        }),
        ct::Event::Paste(text) => InputEvent::Paste(text),
        ct::Event::Resize(columns, rows) => InputEvent::Resize { columns, rows },
        ct::Event::FocusGained => InputEvent::FocusGained,
        ct::Event::FocusLost => InputEvent::FocusLost,
    }
}

fn mouse_button(value: ct::MouseButton) -> MouseButton {
    match value {
        ct::MouseButton::Left => MouseButton::Primary,
        ct::MouseButton::Right => MouseButton::Secondary,
        ct::MouseButton::Middle => MouseButton::Middle,
    }
}

/// Reports producer capability separately from the chord. A terminal without
/// disambiguation cannot promise Ctrl+digit delivery; never alias to a digit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChordSupport {
    Supported,
    RequiresDisambiguation,
    Unsupported,
}
pub fn chord_support(chord: KeyChord, disambiguated: bool) -> ChordSupport {
    if chord.code == KeyCode::Unsupported {
        ChordSupport::Unsupported
    } else if !disambiguated
        && chord.modifiers.contains(Modifiers::CONTROL)
        && matches!(chord.code, KeyCode::Char('0'..='9'))
    {
        ChordSupport::RequiresDisambiguation
    } else {
        ChordSupport::Supported
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn modifiers_kind_paste_mouse_and_unsupported_remain_explicit() {
        for kind in [
            ct::KeyEventKind::Press,
            ct::KeyEventKind::Repeat,
            ct::KeyEventKind::Release,
        ] {
            let InputEvent::Key(event) = normalize(ct::Event::Key(ct::KeyEvent::new_with_kind(
                ct::KeyCode::Char('3'),
                ct::KeyModifiers::CONTROL,
                kind,
            ))) else {
                panic!()
            };
            assert_eq!(event.chord.modifiers, Modifiers::CONTROL);
            assert_eq!(
                chord_support(event.chord, false),
                ChordSupport::RequiresDisambiguation
            );
            assert_eq!(admits_input(event.kind), kind == ct::KeyEventKind::Press);
        }
        assert_eq!(
            normalize(ct::Event::Paste("中文".into())),
            InputEvent::Paste("中文".into())
        );
        assert_eq!(
            normalize(ct::Event::Resize(24, 10)),
            InputEvent::Resize {
                columns: 24,
                rows: 10
            }
        );
        assert!(matches!(
            normalize(ct::Event::Key(ct::KeyEvent::new(
                ct::KeyCode::Null,
                ct::KeyModifiers::NONE
            ))),
            InputEvent::Key(KeyEvent {
                chord: KeyChord {
                    code: KeyCode::Unsupported,
                    ..
                },
                ..
            })
        ));
        assert!(matches!(
            normalize(ct::Event::Mouse(ct::MouseEvent {
                kind: ct::MouseEventKind::ScrollDown,
                column: 2,
                row: 3,
                modifiers: ct::KeyModifiers::ALT
            })),
            InputEvent::Mouse(MouseEvent {
                kind: MouseKind::ScrollDown,
                modifiers: Modifiers::ALT,
                ..
            })
        ));
    }
}
