use std::io::{self, BufWriter, Stdout, Write};

use crossterm::{
    execute, queue,
    terminal::{BeginSynchronizedUpdate, EndSynchronizedUpdate},
};
use ratatui::{Frame, Terminal, backend::CrosstermBackend};

pub(crate) type AppTerminal = Terminal<CrosstermBackend<BufWriter<Stdout>>>;

pub(crate) fn buffered_backend<W: Write>(writer: W) -> CrosstermBackend<BufWriter<W>> {
    // Batch the many small cursor, style, and cell writes produced by a full-screen diff.
    CrosstermBackend::new(BufWriter::with_capacity(64 * 1024, writer))
}

pub(crate) fn draw_frame<W: Write>(
    terminal: &mut Terminal<CrosstermBackend<W>>,
    render: impl FnOnce(&mut Frame<'_>),
) -> io::Result<()> {
    queue!(terminal.backend_mut(), BeginSynchronizedUpdate)?;
    let draw_result = terminal.draw(render).map(|_| ());
    // Do not return a draw error before releasing the terminal's presentation hold.
    let end_result = execute!(terminal.backend_mut(), EndSynchronizedUpdate);
    draw_result.and(end_result)
}
