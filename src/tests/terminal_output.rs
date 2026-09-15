use std::{
    cell::RefCell,
    io::{self, Write},
    rc::Rc,
};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{Terminal, TerminalOptions, Viewport, backend::CrosstermBackend, layout::Rect};

use super::support::make_test_app;
use crate::{
    terminal::{buffered_backend, draw_frame},
    ui,
};

const BEGIN: &[u8] = b"\x1b[?2026h";
const END: &[u8] = b"\x1b[?2026l";

#[derive(Default)]
struct Output {
    bytes: Vec<u8>,
    writes: usize,
    fail_write_once: bool,
    fail_flush_once: bool,
    fail_end: bool,
}

#[derive(Clone, Default)]
struct Capture(Rc<RefCell<Output>>);

impl Write for Capture {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let mut output = self.0.borrow_mut();
        if std::mem::take(&mut output.fail_write_once) {
            return Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "frame write failed",
            ));
        }
        if output.fail_end && bytes == END {
            return Err(io::Error::other("end update failed"));
        }
        output.writes += 1;
        output.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        if std::mem::take(&mut self.0.borrow_mut().fail_flush_once) {
            return Err(io::Error::other("frame flush failed"));
        }
        Ok(())
    }
}

fn fixed_terminal<W: Write>(
    backend: CrosstermBackend<W>,
    area: Rect,
) -> Terminal<CrosstermBackend<W>> {
    Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Fixed(area),
        },
    )
    .unwrap()
}

#[test]
fn dialog_open_and_close_batch_unchanged_output_inside_synchronized_frames() {
    for (width, height) in [(80, 24), (160, 50), (240, 80)] {
        let area = Rect::new(0, 0, width, height);
        let mut app = make_test_app(100, usize::from(height));
        app.set_screen_area(area);
        let original = Capture::default();
        let buffered = Capture::default();
        let mut original_terminal = fixed_terminal(CrosstermBackend::new(original.clone()), area);
        let mut buffered_terminal = fixed_terminal(buffered_backend(buffered.clone()), area);
        original_terminal
            .draw(|frame| ui::draw(frame, &app))
            .unwrap();
        draw_frame(&mut buffered_terminal, |frame| ui::draw(frame, &app)).unwrap();

        for (key, open) in [(KeyCode::Char('c'), true), (KeyCode::Esc, false)] {
            *original.0.borrow_mut() = Output::default();
            *buffered.0.borrow_mut() = Output::default();
            app.on_key(KeyEvent::new(key, KeyModifiers::NONE)).unwrap();
            assert_eq!(app.show_column_picker, open);
            original_terminal
                .draw(|frame| ui::draw(frame, &app))
                .unwrap();
            draw_frame(&mut buffered_terminal, |frame| ui::draw(frame, &app)).unwrap();

            let original = original.0.borrow();
            let buffered = buffered.0.borrow();
            let payload = buffered
                .bytes
                .strip_prefix(BEGIN)
                .unwrap()
                .strip_suffix(END)
                .unwrap();
            assert_eq!(payload, original.bytes);
            assert!(
                buffered.writes * 10 < original.writes,
                "full-screen diffs should be batched"
            );
            println!(
                "{width}x{height} dialog_open={open}: {} bytes, writer calls {} -> {}",
                original.bytes.len(),
                original.writes,
                buffered.writes,
            );
        }
    }
}

#[test]
fn cursor_updates_are_inside_the_synchronized_frame() {
    let output = Capture::default();
    let mut terminal = fixed_terminal(buffered_backend(output.clone()), Rect::new(0, 0, 8, 2));
    draw_frame(&mut terminal, |frame| {
        frame.render_widget("Input", frame.area());
        frame.set_cursor_position((5, 0));
    })
    .unwrap();
    let output = output.0.borrow();
    assert!(output.bytes.starts_with(BEGIN));
    assert!(output.bytes.ends_with(END));
    let text = String::from_utf8_lossy(&output.bytes);
    assert!(text.contains("\x1b[?25h"));
    assert!(text.contains("\x1b[1;6H"));
}

#[test]
fn failed_frame_write_still_ends_synchronized_update_and_preserves_draw_error() {
    let output = Capture::default();
    let mut terminal = fixed_terminal(CrosstermBackend::new(output.clone()), Rect::new(0, 0, 8, 2));
    let error = draw_frame(&mut terminal, |frame| {
        output.0.borrow_mut().fail_write_once = true;
        frame.render_widget("Dialog", frame.area());
    })
    .unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::BrokenPipe);
    assert!(output.0.borrow().bytes.starts_with(BEGIN));
    assert!(output.0.borrow().bytes.ends_with(END));
}

#[test]
fn failed_buffer_flush_still_ends_synchronized_update() {
    let output = Capture::default();
    output.0.borrow_mut().fail_flush_once = true;
    let mut terminal = fixed_terminal(buffered_backend(output.clone()), Rect::new(0, 0, 8, 2));
    let error = draw_frame(&mut terminal, |frame| {
        frame.render_widget("Dialog", frame.area());
    })
    .unwrap_err();
    assert_eq!(error.to_string(), "frame flush failed");
    assert!(output.0.borrow().bytes.starts_with(BEGIN));
    assert!(output.0.borrow().bytes.ends_with(END));
}

#[test]
fn failed_end_update_is_reported_after_a_successful_draw() {
    let output = Capture::default();
    output.0.borrow_mut().fail_end = true;
    let mut terminal = fixed_terminal(CrosstermBackend::new(output.clone()), Rect::new(0, 0, 8, 2));
    let error = draw_frame(&mut terminal, |frame| {
        frame.render_widget("Dialog", frame.area());
    })
    .unwrap_err();
    assert_eq!(error.to_string(), "end update failed");
}
