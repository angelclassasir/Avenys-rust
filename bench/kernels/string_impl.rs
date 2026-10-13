// P3: method delegating through a trait to another type.
trait Area { fn area(&self) -> i64; }
struct Rect { a: i64, b: i64 }
struct Cuad { a: i64 }
impl Area for Rect { #[inline(never)] fn area(&self) -> i64 { self.a * self.b } }
impl Area for Cuad { #[inline(never)] fn area(&self) -> i64 { self.a * self.a } }

struct Figura { alto: i64, ancho: i64 }

impl Figura {
    #[inline(never)]
    fn superficie(&self) -> i64 {
        if self.alto == self.ancho { Cuad { a: self.alto }.area() }
        else { Rect { a: self.alto, b: self.ancho }.area() }
    }
}

fn main() {
    let mut acc = OPACUE(987654321i64);
    let mut i: i64 = 0;
    while i < 8_000_000 {
        acc = (acc + Figura { alto: acc % 97, ancho: acc % 97 + 1 }.superficie()) % 1_000_003;
        i += 1;
    }
    println!("{}", acc);
}
