mod structs;
use structs::invertor_dvij::*;

fn main() {
    println!("!!Hello, world!");
    
    let dvij = InvertorDvij::new();
    let _sub = dvij.messages.get_stream()
        .subscribe(|message| {dbg!(&message);});
        
    dvij.set_speed(12_000);
    dvij.set_hz(300);
}
