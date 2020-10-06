use crossbeam::channel::Sender;

pub enum ControllerMessage {
    Paused,
    Close,
}

pub fn setup_controller(senders: Vec<Sender<ControllerMessage>>) {
    // for c in stdin() {
    //     let event = c.expect("Couldn't access stdin event");

    //     match event {
    //         Event::Key(Key::Ctrl('c')) => {
    //             for sender in &senders {
    //                 sender
    //                     .send(ControllerMessage::Close)
    //                     .expect("Couldn't close a thread.");
    //             }
    //         }
    //         _ => {
    //             for sender in &senders {
    //                 sender
    //                     .send(ControllerMessage::Paused)
    //                     .expect("Couldn't close a thread.");
    //             }
    //         }
    //     };
    // }
}
