use clap::{ArgGroup, Parser};
use pulp::{Scalar512b, Simd};
use std::io::{self, BufRead, Read};

const LANES: usize = Scalar512b::U8_LANES;
const GROUP: usize = LANES * u8::MAX as usize;
const AT_ONCE: usize = GROUP * 4;

/// Simple program to count the number of occurances of a byte in stdin
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(group(
    ArgGroup::new("wantg")
        .required(true)
        .multiple(false) // This is the default, ensuring one and only one
))]
struct Opts {
    /// The byte to count
    #[arg(short, long, default_value_t = '\n', group = "wantg")]
    want: char,

    /// Helper to set want to null
    #[arg(short = '0', long, group = "wantg")]
    null: bool,

    /// Helper to set want to a newline
    #[arg(short = 'n', long, group = "wantg")]
    newline: bool,

    /// Comma separated output
    #[arg(short, long)]
    comma: bool,
}

fn main() -> std::result::Result<(), io::Error> {
    let opts = Opts::parse();

    let want;

    if opts.null {
        want = 0;
    } else if opts.newline {
        want = b'\n';
    } else {
        let c = opts.want;
        if !c.is_ascii() {
            panic!("only ascii characters supported");
        }
        want = c as u8;
    }

    let total = count_buf(io::stdin().lock(), want)?;

    if opts.comma {
        print_total(total);
    } else {
        println!("{}", total);
    }

    Ok(())
}

fn print_total(total: u64) {
    use num_format::{Buffer, Locale};

    let mut buf = Buffer::default();
    buf.write_formatted(&total, &Locale::en);

    println!("{}", buf)
}

fn count_buf<R>(reader: R, want: u8) -> Result<u64, io::Error>
where
    R: Read,
{
    let mut bin = io::BufReader::with_capacity(AT_ONCE, reader);
    let mut total = 0u64;

    loop {
        let b = bin.fill_buf()?;
        let l = b.len();

        if l == 0 {
            break;
        }

        total += count(b, want);
        bin.consume(l);
    }

    Ok(total)
}

// the macro creates a `count` function
#[pulp::with_simd(count = pulp::Arch::new())]
#[inline(always)]
fn count_with_simd<S: Simd>(simd: S, v: &[u8], want: u8) -> u64 {
    let (head, tail) = S::as_simd_u8s(v);

    let mut sum = simd.splat_u8s(0);
    let add = simd.splat_u8s(1);

    let mask = simd.splat_u8s(want);

    let mut total = 0u64;
    let mut bits = [0u8; 32];

    for (n, h) in head.iter().enumerate() {
        let eq = simd.equal_u8s(*h, mask);
        let seq = simd.transmute_u8s_m8s(eq);
        let to_add = simd.and_u8s(seq, add);

        sum = simd.add_u8s(sum, to_add);

        if n.is_multiple_of(u8::MAX as usize) {
            simd.partial_store_u8s(&mut bits, sum);
            for b in &mut bits {
                total += *b as u64;
                *b = 0;
            }
            sum = simd.splat_u8s(0);
        }
    }

    simd.partial_store_u8s(&mut bits, sum);
    for b in bits {
        total += b as u64;
    }

    for &h in tail {
        if h == want {
            total += 1;
        }
    }

    total
}

#[cfg(test)]
mod tests {
    use super::{AT_ONCE, count_buf};

    use std::io;

    #[test]
    fn no_overflow() {
        let want = 7u8;
        const SIZE: usize = AT_ONCE + 1;

        let buf = [want; SIZE];
        let c = io::Cursor::new(buf);

        assert_eq!(SIZE, count_buf(c, want).unwrap() as usize);
    }
}
