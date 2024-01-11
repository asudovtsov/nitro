use nitro::*;

fn main() {
    let mut storage = Storage::new();

    let u32_ids = [
        storage.place::<u32>(0),
        storage.place::<u32>(1),
        storage.place::<u32>(2),
    ];

    let untyped_ids = [
        storage.id_mut().place::<u32>(0),
        storage.id_mut().place::<u8>(1),
        storage.id_mut().place::<String>("2".into()),
    ];

    println!("{}", u32_ids.len());
    println!("{}", untyped_ids.len());
}
