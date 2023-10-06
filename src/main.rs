use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;
use rdev::{Button, EventType, Key};

lazy_static! {
    static ref CLICK_STATE: Mutex<ClickState> = Mutex::new(ClickState::WAITING);
    static ref CAN_SIMULATE: Mutex<bool> = Mutex::new(false);
}


#[tokio::main]
async fn main() {
    println!("Starting listener...");
    listen_for_clicks().await;
}


#[derive(Debug, Clone, PartialEq)]
enum ClickState {
    SIMULATING,
    CLICKED_LEFT,
    RELEASED_LEFT,
    WAITING,
}

async fn listen_for_clicks() {
    rdev::listen(|event| {
        let event_type = event.event_type.clone();


        match event_type {
            EventType::KeyPress(Key::BackQuote) => {
                let can_click = *CAN_SIMULATE.lock().unwrap();

                *CAN_SIMULATE.lock().unwrap() = !can_click;
                println!("Can simulate: {}", !can_click);
            },
            EventType::ButtonPress(Button::Left) => {
                println!("Left button pressed!");
                let current_state = CLICK_STATE.lock().unwrap().clone();

                if current_state != ClickState::SIMULATING {
                    *CLICK_STATE.lock().unwrap() = ClickState::CLICKED_LEFT;
                }
            },
            EventType::ButtonRelease(Button::Left) => {
                let current_state = CLICK_STATE.lock().unwrap().clone();

                if current_state != ClickState::SIMULATING && *CAN_SIMULATE.lock().unwrap() {
                    *CLICK_STATE.lock().unwrap() = ClickState::SIMULATING;
                    println!("Simulating...");
                    tokio::spawn(async {
                        send_button_action(&EventType::ButtonPress(Button::Left)).await;
                        send_button_action(&EventType::ButtonRelease(Button::Left)).await;
                    });
                } else {
                    println!("Finished simulation!");
                    *CLICK_STATE.lock().unwrap() = ClickState::RELEASED_LEFT;
                }
            },
            _ => ()
        }
    }).expect("Could not listen for events");
}

async fn send_button_action(event: &EventType) {
    let result = match rdev::simulate(event) {
        Ok(_) => "Success",
        Err(_) => "Failure",
    };
}