use num_format::{Buffer, Locale};
use packed_simd::u8x64;
use std::io;
use std::io::Read;

fn print(total: &u64) {
    let mut buf = Buffer::default();
    buf.write_formatted(total, &Locale::en);
    println!("{}", buf)
}

fn newlines(bytes: [u8; 64]) -> u8x64 {
    u8x64::from(bytes)
        .eq(u8x64::splat(b'\n'))
        .select(u8x64::splat(1), u8x64::splat(0))
}

fn main() -> io::Result<()> {
    let mut total = 0u64;
    let mut sums = u8x64::splat(0);

    let mut b = [0; 64];
    let sin = io::stdin();
    let sin = sin.lock();
    let mut sin = io::BufReader::with_capacity(1 << 23, sin);

    let mut n = 0;
    loop {
        match sin.read(&mut b) {
            Ok(read) => {
                if read == 64 {
                    sums += newlines(b);
                    if n == u8::max_value() {
                        for i in 0..64 {
                            total += sums.extract(i) as u64;
                        }
                        sums &= 0;
                        n = 0;
                    } else {
                        n += 1;
                    }
                } else if read == 0 {
                    break;
                } else {
                    for &c in b[0..read].iter() {
                        if c == b'\n' {
                            total += 1
                        }
                    }
                }
            }

            Err(e) => {
                if e.kind() != io::ErrorKind::Interrupted {
                    return Err(e);
                }
            }
        }
    }

    if n > 0 {
        for i in 0..64 {
            total += sums.extract(i) as u64;
        }
    }

    print(&total);
    Ok(())
}
