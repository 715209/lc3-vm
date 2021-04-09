use std::{env, io};

use lc3::Lc3;

fn main() -> Result<(), io::Error> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Not enough arguments. Add filename e.g. lc3-vm 2024.obj");
        return Ok(());
    }

    let mut lc3 = Lc3::default();
    lc3.load_image_file(&args[1])?;
    lc3.run();

    Ok(())
}
