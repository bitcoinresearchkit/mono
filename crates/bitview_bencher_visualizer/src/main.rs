use bitview_bencher_visualizer::Visualizer;

fn main() {
    let v = Visualizer::from_cargo_env().unwrap();
    v.generate_all_charts().unwrap();
}
