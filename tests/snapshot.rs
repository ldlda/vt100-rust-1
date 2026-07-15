const SCROLLBACK_ROWS: usize = 100;

#[test]
fn snapshot_preserves_wrapped_contents_and_cursor() {
    let mut source = vt100::Parser::new(3, 5, SCROLLBACK_ROWS);
    source.process(b"123456789");

    let mut restored = vt100::Parser::new(3, 5, SCROLLBACK_ROWS);
    restored.process(&source.screen().snapshot_formatted(SCROLLBACK_ROWS));

    assert_eq!(restored.screen().contents(), source.screen().contents());
    assert_eq!(
        restored.screen().cursor_position(),
        source.screen().cursor_position(),
    );
}

#[test]
fn snapshot_restores_hidden_primary_after_alternate_screen_exit() {
    let mut source = vt100::Parser::new(3, 20, SCROLLBACK_ROWS);
    source.process(
        b"\x1b[31mshell history\r\n$ btop\x1b[?1049h\x1b[34mbtop dashboard",
    );

    let mut restored = vt100::Parser::new(3, 20, SCROLLBACK_ROWS);
    restored.process(&source.screen().snapshot_formatted(SCROLLBACK_ROWS));

    assert!(restored.screen().alternate_screen());
    assert_eq!(restored.screen().contents(), "btop dashboard");
    assert_eq!(restored.screen().fgcolor(), vt100::Color::Idx(4));

    source.process(b"\x1b[?1049l");
    restored.process(b"\x1b[?1049l");
    assert_eq!(restored.screen().contents(), source.screen().contents());
    assert_eq!(
        restored.screen().cursor_position(),
        source.screen().cursor_position(),
    );
    assert_eq!(restored.screen().fgcolor(), vt100::Color::Idx(1));

    restored.screen_mut().set_scrollback(usize::MAX);
    assert!(restored.screen().contents().contains("shell history"));
    assert!(!restored.screen().contents().contains("btop dashboard"));
}

#[test]
fn snapshot_does_not_change_the_source_scrollback_viewport() {
    let mut source = vt100::Parser::new(3, 20, SCROLLBACK_ROWS);
    for line in 0..10 {
        source.process(format!("history {line}\r\n").as_bytes());
    }
    source.screen_mut().set_scrollback(4);

    let before_contents = source.screen().contents();
    let before_state = source.screen().state_formatted();
    let before_scrollback = source.screen().scrollback();
    let _ = source.screen().snapshot_formatted(SCROLLBACK_ROWS);

    assert_eq!(source.screen().contents(), before_contents);
    assert_eq!(source.screen().state_formatted(), before_state);
    assert_eq!(source.screen().scrollback(), before_scrollback);
}

#[test]
fn snapshot_does_not_resurrect_scrollback_cleared_by_ed3() {
    let mut source = vt100::Parser::new(3, 20, SCROLLBACK_ROWS);
    for line in 0..10 {
        source.process(format!("history {line}\r\n").as_bytes());
    }

    source.screen_mut().set_scrollback(usize::MAX);
    assert!(source.screen().scrollback() > 0);
    source.screen_mut().set_scrollback(0);

    // This is the relevant portion of a terminfo-aware `clear`: ED 3 drops
    // saved lines, then ED 2 redraws an empty viewport at home.
    source.process(b"\x1b[3J\x1b[2J\x1b[H$ ");
    source.screen_mut().set_scrollback(usize::MAX);
    assert_eq!(source.screen().scrollback(), 0);

    let mut restored = vt100::Parser::new(3, 20, SCROLLBACK_ROWS);
    restored.process(&source.screen().snapshot_formatted(SCROLLBACK_ROWS));
    restored.screen_mut().set_scrollback(usize::MAX);

    assert_eq!(restored.screen().scrollback(), 0);
    assert_eq!(restored.screen().contents(), "$ ");
    assert!(!restored.screen().contents().contains("history"));
}
