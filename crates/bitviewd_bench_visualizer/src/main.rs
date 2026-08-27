use bitviewd_bench_visualizer::Visualizer;

fn main() {
    let v = Visualizer::from_cargo_env().unwrap();
    v.generate().unwrap();
}
