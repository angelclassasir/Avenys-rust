// P1: struct method calls.
struct Contador { n: i64 }

impl Contador {
    #[inline(never)]
    fn total(&self) -> i64 { self.n * 2 + 1 }
}

fn main() {
    let mut acc = OPACUE(987654321i64);
    let mut i: i64 = 0;
    while i < 20_000_000 {
        let c = Contador { n: (acc + i) % 1_000_003 };
        acc = (acc + c.total()) % 1_000_003;
        i += 1;
    }
    println!("{}", acc);
}
