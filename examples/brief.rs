use nitro::*;

fn main() {
    let mut storage = Storage::new();
    let ids = [
        storage.place::<u32>(0),
        storage.place::<u8>(1),
        storage.place::<String>("2".into()),
    ];

    assert_eq!(0, *storage.get::<u32>(&ids[0]));
    assert_eq!(1, *storage.get::<u8>(&ids[1]));
    assert_eq!("2", storage.get::<String>(&ids[2]));

    *storage.get_mut::<String>(&ids[2]) = "str".into();
    assert_eq!("str", storage.get::<String>(&ids[2]));

    let value = storage.remove::<u32>(&ids[0]).unwrap();
    assert_eq!(value, 0);

    for id in ids.iter() {
        storage.erase(id);
    }
}
