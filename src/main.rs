#![feature(plugin, decl_macro)]
#![plugin(rocket_codegen)]

#[get("/")]
fn index() -> &'static str {
  "Hello, world!"
}

fn main() {
  rocket::ignite()
    .mount("/", routes![index])
    .launch();
}
