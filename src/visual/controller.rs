use crossbeam::channel::Sender;
use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyModifiers};

pub enum ControllerMessage {
    Paused,
    Close,
}

type Senders = Vec<Sender<ControllerMessage>>;

pub fn setup_controller(senders: Senders) -> crossterm::Result<()> {
    loop {
        match read()? {
            Event::Key(event) => {
                println!("Detected a key: {:?}", event);
                if match_key_event(event, &senders) {
                    break;
                }
            }
            _ => {}
        };
    }

    Ok(())
}

/// If this function returns true, this thread should exit (it got a ctrl-C command).
fn match_key_event(event: KeyEvent, senders: &Senders) -> bool {
    match (event.code, event.modifiers) {
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
            close_all(senders);
            true
        }
        (_, _) => {
            pause(senders);
            false
        }
    }
}

fn pause(senders: &Senders) {
    for sender in senders {
        sender
            .send(ControllerMessage::Paused)
            .expect("Couldn't pause a thread.");
    }
}

fn close_all(senders: &Senders) {
    for sender in senders {
        sender
            .send(ControllerMessage::Close)
            .expect("Couldn't close a thread.");
    }
}
