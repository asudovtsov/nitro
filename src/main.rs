use nitro::arena::Arena;
use nitro::arena_box::ArenaBox;

fn main() {
    let mut arena = Arena::new();
    let v0 = arena.place_box(0);
    let v1 = arena.place_box(1);
    let v2 = arena.place_box(2);
    v0.print();
    v1.print();
    v2.print();
}
