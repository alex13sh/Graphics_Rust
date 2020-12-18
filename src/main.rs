#![allow(dead_code, unused_imports)]

use iced::{
    Align, Column, Row, Scrollable, scrollable, Container, Element, Length,
    Text, text_input, TextInput, button, Button, 
    Application, window, Settings, executor, Subscription, Command, time,
};

mod graphic;

mod app_test_device;
mod app_graphic;
mod app_stand_graphic;

fn main() {
    println!("Hello World");
//     app_graphic::GraphicsApp::run(Settings::default());
//     app_stand_graphic::App::run(Settings::default());
    app_stand_graphic::App::run(Settings { 
        window: window::Settings {
            size: (1920, 1050),
            resizable: true,
            .. Default::default()
        },
        flags: (),
        .. Settings::default()
    });
}

