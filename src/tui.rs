use std::{time::Duration, thread};

use cursive::views::{Dialog, TextView};

use crate::log::Log;

pub struct Tui {}

impl Tui {
    pub fn start(logfile: &Log) {

        // // Creates the cursive root - required for every application.
        // let mut siv = cursive::default();

        // // Creates a dialog with a single "Quit" button
        // siv.add_layer(
        //     Dialog::around(TextView::new("Hello Dialog!"))
        //         .title("Cursive")
        //         .button("Quit", |s| s.quit()),
        // );

        // // Starts the event loop.
        // siv.run();
    }
}
