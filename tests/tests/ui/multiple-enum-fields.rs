use tsap::param;

#[param]
enum Param<const C: bool> {
    NotSupported(f32, f32)
}

fn main() {}
