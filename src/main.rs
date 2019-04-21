use num_format::{Buffer, Locale};
use std::io;

fn print(total: &u64) {
    let mut buf = Buffer::default();
    buf.write_formatted(total, &Locale::en);
    println!("{}", buf)
}

fn main() -> io::Result<()> {
    let n = 123478u64;

    print(&n);
    Ok(())
}
