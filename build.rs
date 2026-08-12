use std::process::Command;

println!("checking if required assets are updated ")

fn main() {
Command::new("sh")
    .args("-c")
    .args("python3 /home/pc/WINUBM/update_assets.py")

}
