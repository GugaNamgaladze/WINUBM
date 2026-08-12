use std::process::Command;

fn main() {

println!("checking if required assets are updated");

Command::new("sh")
    .arg("-c")
    .arg("python3 /home/pc/WINUBM/update_assets.py");

} 

