// mod structs;
// use structs::*;
// mod invertor_engine;
// use invertor_engine::InvertorEngine;
use meln_logic::*;

fn main() {
    println!("!!Hello, world!");
    
    let dvij = InvertorEngine::new();
    let _sub = dvij.messages.get_stream()
        .subscribe(|message| {dbg!(&message);});
        
    dvij.set_speed(12_000);
    dvij.set_hz(300);
}
