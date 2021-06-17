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

#[derive(Router)]
enum Router5 {
    #[to("<capture>")]
    Path {}, // Missing capture field
    #[not_found]
    NotFound,
}

#[derive(Router)]
enum Router6 {
    #[to("<capture>")]
    Path { not_capture: u32 }, // Wrong capture field name
    #[not_found]
    NotFound,
}

#[derive(Router)]
enum Router7 {
    #[to("<a>/<b>")]
    Path { b: u32, a: u32 }, // Wrong order
    #[not_found]
    NotFound,
}

#[derive(Router)]
enum Router8 {
    #[to("<a/b>")] // `a/b` is not an identifier
    Path { a: u32 },
    #[not_found]
    NotFound,
}

fn main() {}
