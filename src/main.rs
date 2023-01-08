use nitro::arena::Arena;
use nitro::arena_box::ArenaBox;

fn main() {
    let mut arena = Arena::new();
    let v0 = arena.place_box(0u8);
    let v1 = arena.place_box(1.);
    let v2 = arena.place_box(2usize);
    let v3 = arena.place_box(String::from("Hello"));
    v0.print();
    v1.print();
    v2.print();
    v3.print();
}
