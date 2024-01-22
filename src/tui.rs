use nix::libc::{kill, SIGTERM};
use ratatui::{
    backend::CrosstermBackend,
    layout::Margin,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use std::{
    collections::VecDeque,
    io::{self, Stderr},
    time::{Duration, Instant},
};

use crate::{
    log::{Log, LogEntry, LogError},
    nix_ext::{Sched, SchedCreationError},
};

type Terminal = ratatui::Terminal<CrosstermBackend<Stderr>>;

struct PeriodicallyUpdate<T> {
    pub val: T,
    pub freq: Duration,
    pub last_update: Instant,
}

impl<T> PeriodicallyUpdate<T>
where
    T: Default,
{
    fn new(freq: Duration) -> Self {
        Self {
            val: T::default(),
            freq,
            last_update: Instant::now(),
        }
    }
}

impl<T> PeriodicallyUpdate<T> {
    fn should_update(&mut self, now: Instant) -> bool {
        let dslu = now.duration_since(self.last_update);
        if dslu > self.freq {
            self.last_update = now;
            true
        } else {
            false
        }
    }
}

/// The state for out tui
pub struct Tui {
    logfile: Log,
    pid1: i32,
    pid2: i32,
    log_entries: PeriodicallyUpdate<VecDeque<LogEntry>>,
    sched1: PeriodicallyUpdate<Sched>,
    sched2: PeriodicallyUpdate<Sched>,
}

#[derive(Debug)]
pub enum TuiError {
    /// Something went wrong with stdout io stuffs
    Io(io::Error),
    LogError(LogError),
    SchedCreationError(SchedCreationError),
}

impl From<LogError> for TuiError {
    fn from(value: LogError) -> Self {
        Self::LogError(value)
    }
}

impl From<io::Error> for TuiError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<SchedCreationError> for TuiError {
    fn from(value: SchedCreationError) -> Self {
        Self::SchedCreationError(value)
    }
}

impl ToString for TuiError {
    fn to_string(&self) -> String {
        match self {
            Self::Io(..) => String::from("something went wrong with the tui. probably restart"),
            Self::LogError(err) => format!("{err}"),
            Self::SchedCreationError(err) => format!("{err}"),
        }
    }
}

impl Tui {
    const LOG_ENTRIES_UPDATE_FREQ: Duration = Duration::from_millis(200);
    /// The color used to distinguish process 1 from process 2
    const P1_COLOR: Color = Color::Rgb(255, 0, 255);
    /// The color used to distinguish process 2 from process 1
    const P2_COLOR: Color = Color::Yellow;

    /// Format a pid as a pixel
    fn fmt_pid_pixel(&self, pid: i32, include_text: bool) -> Span {
        if pid == self.pid1 {
            Span::styled(
                if include_text { "1" } else { " " },
                Style::default().bg(Self::P1_COLOR).fg(Color::Black),
            )
        } else if pid == self.pid2 {
            Span::styled(
                if include_text { "2" } else { " " },
                Style::default().bg(Self::P2_COLOR).fg(Color::Black),
            )
        } else {
            Span::from(if include_text { "?" } else { " " })
        }
    }

    fn draw(&mut self, terminal: &mut Terminal) -> Result<(), TuiError> {
        let spans_with_text = self
            .log_entries
            .val
            .iter()
            .map(|entry| self.fmt_pid_pixel(entry.pid, false))
            .collect::<Vec<_>>();

        terminal.draw(|f| {
            let logs_block = Block::default().borders(Borders::all()).title("Short-Log");
            let logs_block_rect = {
                let mut rect = f.size();
                rect.height = 3;
                rect
            };
            f.render_widget(logs_block, logs_block_rect);

            let logs_para = Paragraph::new(vec![Line::from(spans_with_text)]);
            let logs_para_rect = logs_block_rect.inner(&Margin::new(1, 1));
            f.render_widget(logs_para, logs_para_rect);

            let fsize = f.size();
            let build_sched_widget = |pid, sched: Sched| {
                let mut rect = logs_block_rect;
                rect.y += logs_block_rect.height;
                rect.width = fsize.width / 2;
                rect.height = fsize.height - logs_block_rect.height;
                let para = sched.as_para(rect.width as usize - 2);
                let block = Block::default().borders(Borders::all()).title({
                    let content = format!("Proc-{pid}");
                    let color = if pid == self.pid1 {
                        Self::P1_COLOR
                    } else {
                        Self::P2_COLOR
                    };
                    Span::styled(content, Style::default().fg(color))
                });
                (para, block, rect)
            };

            let (sched1_para, sched1_block, sched1_block_rect) =
                build_sched_widget(self.pid1, self.sched1.val);
            f.render_widget(sched1_block, sched1_block_rect);
            f.render_widget(sched1_para, sched1_block_rect.inner(&Margin::new(1, 1)));

            let (sched2_para, sched2_block, mut sched2_block_rect) =
                build_sched_widget(self.pid2, self.sched2.val);
            sched2_block_rect.x += sched2_block_rect.width;
            if fsize.width % 2 == 1 {
                sched2_block_rect.width += 1;
            }
            f.render_widget(sched2_block, sched2_block_rect);
            f.render_widget(sched2_para, sched2_block_rect.inner(&Margin::new(1, 1)));
        })?;

        Ok(())
    }

    fn run(&mut self) -> Result<(), TuiError> {
        let mut terminal = Self::init_terminal()?;

        loop {
            let now = Instant::now();

            if self.log_entries.should_update(now) {
                self.log_entries.val = self.logfile.read_entries(
                    (terminal.get_frame().size().width as usize)
                        .checked_sub(2)
                        .unwrap_or(0),
                )?;
            }

            if self.sched1.should_update(now) {
                self.sched1.val = Sched::of(self.pid1)?;
            }

            if self.sched2.should_update(now) {
                self.sched2.val = Sched::of(self.pid2)?;
            }

            self.draw(&mut terminal)?;

            if crossterm::event::poll(std::time::Duration::from_millis(250))? {
                // If a key event occurs, handle it
                if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                    if key.kind == crossterm::event::KeyEventKind::Press {
                        match key.code {
                            crossterm::event::KeyCode::Char('q') => break,
                            _ => {}
                        }
                    }
                }
            }
        }

        self.stop_workers();
        Self::reset_terminal()?;
        Ok(())
    }

    /// Boilerplate for initialising a crossterm terminal -- as recommended by
    /// the docs.
    fn init_terminal() -> Result<Terminal, TuiError> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(std::io::stderr(), crossterm::terminal::EnterAlternateScreen)?;
        let terminal = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;
        Ok(terminal)
    }

    /// Boilerplate for resetting terminal on application exit -- as recommended
    /// by the docs.
    fn reset_terminal() -> Result<(), TuiError> {
        crossterm::execute!(std::io::stderr(), crossterm::terminal::LeaveAlternateScreen)?;
        crossterm::terminal::disable_raw_mode()?;
        Ok(())
    }

    fn stop_workers(&self) {
        _ = unsafe { kill(self.pid1, SIGTERM) };
        _ = unsafe { kill(self.pid2, SIGTERM) };
    }

    pub fn start(pid1: i32, pid2: i32, logfile: Log) -> Result<(), TuiError> {
        Tui {
            logfile,
            pid1,
            pid2,
            log_entries: PeriodicallyUpdate::new(Self::LOG_ENTRIES_UPDATE_FREQ),
            sched1: PeriodicallyUpdate::new(Self::LOG_ENTRIES_UPDATE_FREQ),
            sched2: PeriodicallyUpdate::new(Self::LOG_ENTRIES_UPDATE_FREQ),
        }
        .run()
    }
}
