// A6: binary search over a sorted vector.
fn main() {
    let mut seed = std::hint::black_box(987654321i64);
    let n: i64 = 4096;
    let mut v: Vec<i64> = Vec::with_capacity(n as usize);
    for k in 0..n { v.push(k * 2); }
    let mut round: i64 = 0;
    while round < 2000 {
        for probe in 0..256 {
            let target = (probe * 7 + round) % 8192;
            let mut idx: i64 = -1;
            let mut lo: i64 = 0;
            let mut hi: i64 = n - 1;
            while lo <= hi {
                let mid = lo + (hi - lo) / 2;
                if v[mid as usize] == target { idx = mid; break; }
                if v[mid as usize] < target { lo = mid + 1; } else { hi = mid - 1; }
            }
            seed = (seed + idx + probe) % 1000003;
        }
        round += 1;
    }
    println!("{}", seed);
}
