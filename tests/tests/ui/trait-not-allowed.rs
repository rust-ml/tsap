use tsap::param;

#[param]
trait Param<const C: bool> {
    type Assoc;
}

fn main() {}
