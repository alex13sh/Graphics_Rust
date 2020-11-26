#![allow(dead_code, unused_imports)]

use iced::{
    Align, Column, Row, Scrollable, scrollable, Container, Element, Length,
    Text, text_input, TextInput, button, Button, 
    Application, window, Settings, executor, Subscription, Command, time,
};

mod graphic;

mod app_test_device;
mod app_graphic;

fn main() {
    println!("Hello World");
//     app_graphic::GraphicsApp::run(Settings::default());
    app_test_device::TestDeviceApp::run(Settings { 
        window: window::Settings {
            //size: (600, 500), //
            size: (1200, 800),
            resizable: true,
            .. Default::default()
        },
        flags: (),
        .. Settings::default()
    });
}

