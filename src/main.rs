#[macro_use]
extern crate chan;
extern crate ncurses;
extern crate rmatrix;
extern crate chan_signal;

use std::env;
use ncurses::*;
use chan_signal::Signal;

use rmatrix::Matrix;
use rmatrix::config::Config;

fn main() {
    // Get command line args
    let mut config = Config::new();

    // Get `$TERM`
    let term = env::var("TERM").unwrap_or(String::from(""));

    // Force `$TERM` to be 'linux' if the user asked
    if config.force && term.as_str() != "linux" {
        env::set_var("TERM", "linux");
    }

    // Save the terminal state and start up ncurses
    rmatrix::ncurses_init();

    // Register for UNIX signals
    let signal = chan_signal::notify(&[Signal::INT, Signal::WINCH]);

    // Create the board
    let mut matrix = Matrix::new();

    // Main event loop
    loop {
        // Check for SIGINT or SIGWINCH
        chan_select! {
            default => {},
            signal.recv() -> signal => {
                if let Some(signal) = signal {
                    match signal {
                        // Terminate ncurses properly on SIGINT
                        Signal::INT => rmatrix::finish(),
                        // Redraw the screen on SIGWINCH
                        Signal::WINCH => {
                            rmatrix::resize_window();
                            matrix = Matrix::new();
                        },
                        _ => {}
                    }
                }
            },
        }

        // Handle a keypress
        let keypress = wgetch(stdscr());
        if keypress != ERR {
            // Exit if in screensaver mode
            if config.screensaver {
                rmatrix::finish();
            }

            // Update config based on user input
            config.update_from_keypress(keypress as u8 as char);
            // Check any config changes mean you need to exit the loop
            if config.should_break {
                break;
            }
        }

        // Updaate and redraw the board
        matrix.arrange(&config);
        matrix.draw(&config);
    }

    // Reset the old `$TERM` value if you changed it
    if config.force && term.as_str() != "" {
        env::set_var("TERM", term.as_str());
    }
    rmatrix::finish()
}
