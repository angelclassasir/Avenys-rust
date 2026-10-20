// A8: partitioning a vector into two by a predicate.
fn main() {
    let mut seed = std::hint::black_box(987654321i64);
    let mut round: i64 = 0;
    while round < 40000 {
        let mut src: Vec<i64> = Vec::with_capacity(128);
        let mut k: i64 = 0;
        while k < 128 {
            src.push(k * 7 + round);
            k += 1;
        }
        let mut even: Vec<i64> = Vec::with_capacity(128);
        let mut odd: Vec<i64> = Vec::with_capacity(128);
        let mut j: i64 = 0;
        while j < 128 {
            let x = src[j as usize];
            if x % 2 == 0 { even.push(x); } else { odd.push(x); }
            j += 1;
        }
        seed = (seed + even.len() as i64 + odd.len() as i64) % 1000003;
        round += 1;
    }
    println!("{}", seed);
}
