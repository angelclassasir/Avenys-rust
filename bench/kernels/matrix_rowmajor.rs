// A5: 2D access pattern.
fn main() {
    let mut seed = std::hint::black_box(987654321i64);
    let n: i64 = 128;
    let mut round: i64 = 0;
    while round < 800 {
        let mut m: Vec<i64> = Vec::with_capacity((n * n) as usize);
        let mut r: i64 = 0;
        while r < n {
            let mut c: i64 = 0;
            while c < n {
                m.push(r * n + c + round);
                c += 1;
            }
            r += 1;
        }
        let mut acc: i64 = 0;
        let mut c: i64 = 0;
        while c < n {
            let mut r: i64 = 0;
            while r < n {
                acc += m[(r * n + c) as usize];
                r += 1;
            }
            c += 1;
        }
        seed = (seed + acc) % 1000003;
        round += 1;
    }
    println!("{}", seed);
}
