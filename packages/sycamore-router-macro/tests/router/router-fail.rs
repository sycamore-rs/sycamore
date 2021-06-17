use sycamore_router::Router;

#[derive(Router)]
struct Router1 {}

// Missing #[not_found]
#[derive(Router)]
enum Router2 {}

#[derive(Router)]
enum Router3 {
    #[not_found]
    NotFound(i32), // Cannot have field
}

#[derive(Router)]
enum Router4 {
    #[to("<capture>")]
    Path, // Missing capture field
    #[not_found]
    NotFound,
}

fn main() {}
