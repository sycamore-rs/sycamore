use sycamore_router::Router;

#[derive(Router)]
enum Router1 {
    #[not_found]
    NotFound,
}

#[derive(Router)]
enum Router2 {
    #[to("/")]
    Home,
    #[to("/about")]
    About,
    #[not_found]
    NotFound,
}

#[derive(Router)]
enum Router3 {
    #[to("/hello/<name>/<age>")]
    Hello(String, u32),
    #[to("/account/<id>")]
    Account(u32),
    #[not_found]
    NotFound,
}

#[derive(Router)]
enum Router4 {
    #[to("/hello/<name>/<age>")]
    Hello { name: String, age: u32 },
    #[to("/account/<id>")]
    Account { id: u32 },
    #[not_found]
    NotFound,
}

fn main() {}
