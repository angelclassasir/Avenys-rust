// A4: sorting.
fn main() {
    let mut seed = std::hint::black_box(987654321i64);
    let mut round: i64 = 0;
    while round < 200 {
        let mut v: Vec<i64> = Vec::with_capacity(2000);
        let mut k: i64 = 0;
        while k < 2000 {
            v.push((k * 2654435761i64 + round) % 1000003);
            k += 1;
        }
        v.sort();
        seed = (seed + v[0] + v[1999]) % 1000003;
        round += 1;
    }
    println!("{}", seed);
}
