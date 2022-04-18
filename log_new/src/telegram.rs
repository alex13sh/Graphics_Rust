use std::process::Command;

pub fn send_text(text: &str) {
    let mut cmd = Command::new("tepe").arg("send")
        .arg("-t").arg("673166809:AAFK3kJQn9v40fttsbuAQ9PTT0396QER5uQ")
        .arg("-c 420586828") // Чат со мной
        .arg("-m").arg(text)
        .spawn().unwrap()
        .wait().unwrap();
}

pub fn send_file(cap: &str, file: &str) {
    Command::new("tepe").arg("send")
        .arg("-t").arg("673166809:AAFK3kJQn9v40fttsbuAQ9PTT0396QER5uQ")
        .arg("-c 420586828") // Чат со мной
        .arg("-m").arg(cap)
        .arg("--").arg(file)
        .spawn().unwrap()
        .wait().unwrap();
}
