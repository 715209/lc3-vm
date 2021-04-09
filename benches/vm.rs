#![feature(test)]

extern crate test;

#[cfg(test)]
mod tests {
    use lc3::Lc3;
    use test::Bencher;

    #[bench]
    fn bench_vm_32768x10_adds(b: &mut Bencher) {
        let mut lc3 = Lc3::default();
        lc3.load_image_file("./benches/bench.obj").unwrap();
        let state = lc3.into_state();

        b.iter(|| {
            let mut lc3 = Lc3::from_state(state.clone());
            lc3.run();
        });
    }
}
