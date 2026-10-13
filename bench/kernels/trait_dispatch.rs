// P2: one trait, two impls.
trait Medible { fn valor(&self) -> i64; }
struct Cuadro { lado: i64 }
struct Circulo { radio: i64 }
impl Medible for Cuadro { #[inline(never)] fn valor(&self) -> i64 { self.lado * self.lado } }
impl Medible for Circulo { #[inline(never)] fn valor(&self) -> i64 { self.radio * self.radio * 3 } }

fn main() {
    let mut acc = OPACUE(987654321i64);
    let mut i: i64 = 0;
    while i < 8_000_000 {
        if acc % 2 == 0 { acc = (acc + Cuadro { lado: (acc + i) % 1_000_003 }.valor()) % 1_000_003; }
        else { acc = (acc + Circulo { radio: (acc + i) % 1_000_003 }.valor()) % 1_000_003; }
        i += 1;
    }
    println!("{}", acc);
}
