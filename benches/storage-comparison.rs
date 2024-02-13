use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use froggy::Pointer;
use rand::{thread_rng, Rng};
use shared_arena::ArenaBox;
use shipyard::Get;

#[derive(edict::Component, shipyard::Component, Debug, Default)]
struct Pos {
    x: f32,
    y: f32,
}

#[derive(edict::Component, shipyard::Component, Debug, Default)]
struct Rect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

#[derive(edict::Component, shipyard::Component, Debug, Default)]
struct Margins {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
}

#[derive(edict::Component, shipyard::Component, Debug, Default)]
struct Opacity {
    opacity: f32,
}

#[derive(edict::Component, shipyard::Component, Debug, Default)]
struct Visible {
    visible: bool,
}

type Composed = (Pos, Rect, Margins, Opacity, Visible);

fn insert(c: &mut Criterion) {
    let size = 10_000;

    let mut g = c.benchmark_group("Insret");
    g.bench_function("vec", |b| {
        b.iter_batched_ref(
            || {
                let mut s = Vec::new();
                for _ in 0..size {
                    s.push(Composed::default());
                }
                s
            },
            |s| {
                for _ in 0..size {
                    s.pop();
                }
            },
            BatchSize::SmallInput,
        )
    });
    g.bench_function("generational-arena", |b| {
        b.iter_batched_ref(
            generational_arena::Arena::new,
            |s| {
                for _ in 0..size {
                    s.insert(Composed::default());
                }
            },
            BatchSize::SmallInput,
        )
    });
    g.bench_function("shared-arena", |b| {
        b.iter_batched_ref(
            || {
                (
                    shared_arena::Arena::<Composed>::new(),
                    Vec::<ArenaBox<Composed>>::with_capacity(size),
                )
            },
            |(s, ids)| {
                for _ in 0..size {
                    ids.push(s.alloc(Composed::default()));
                }
            },
            BatchSize::SmallInput,
        )
    });
    g.bench_function("blink", |b| {
        b.iter_batched_ref(
            blink_alloc::Blink::new,
            |s| {
                for _ in 0..size {
                    s.put(Composed::default());
                }
            },
            BatchSize::SmallInput,
        )
    });
    g.bench_function("froggy", |b| {
        b.iter_batched_ref(
            || {
                (
                    froggy::Storage::<Composed>::new(),
                    Vec::<Pointer<Composed>>::with_capacity(size),
                )
            },
            |(s, ids)| {
                for _ in 0..size {
                    ids.push(s.create(Composed::default()));
                }
            },
            BatchSize::SmallInput,
        )
    });
    g.bench_function("hecs", |b| {
        b.iter_batched_ref(
            hecs::World::new,
            |s| {
                for _ in 0..size {
                    s.spawn(Composed::default());
                }
            },
            BatchSize::SmallInput,
        )
    });
    g.bench_function("shipyard", |b| {
        b.iter_batched_ref(
            shipyard::World::new,
            |s| {
                for _ in 0..size {
                    s.add_entity(Composed::default());
                }
            },
            BatchSize::SmallInput,
        )
    });
    g.bench_function("edict", |b| {
        b.iter_batched_ref(
            edict::World::new,
            |s| {
                for _ in 0..size {
                    s.spawn(Composed::default());
                }
            },
            BatchSize::SmallInput,
        )
    });
    g.bench_function("nitro", |b| {
        b.iter_batched_ref(
            nitro::Storage::new,
            |s| {
                for _ in 0..size {
                    s.place(Composed::default());
                }
            },
            BatchSize::SmallInput,
        )
    });
}

// fn insert_batch(c: &mut Criterion) {
//     let size = 10_000;
//     let mut g = c.benchmark_group("Insret batch");
//     g.bench_function("hecs", |b| {
//         b.iter_batched_ref(
//             hecs::World::new,
//             |s| {
//                 s.spawn_batch((0..size).map(|_| Composed::default()))
//                     .for_each(|_| {});
//             },
//             BatchSize::SmallInput,
//         )
//     });
//     g.bench_function("nitro", |b| {
//         b.iter_batched_ref(
//             nitro::Storage::new,
//             |s| {
//                 let mut placer = s.placer::<Composed>();
//                 for _ in 0..size {
//                     placer.place(Composed::default());
//                 }
//             },
//             BatchSize::SmallInput,
//         )
//     });
//     todo!()
// }

fn remove(c: &mut Criterion) {
    let size = 10_000;

    let mut g = c.benchmark_group("Remove");
    g.bench_function("vec", |b| {
        b.iter_batched_ref(
            || {
                let mut s = Vec::with_capacity(size);
                for _ in 0..size {
                    s.push(Composed::default());
                }
                s
            },
            |s| {
                for _ in 0..size {
                    s.pop();
                }
            },
            BatchSize::SmallInput,
        )
    });
    g.bench_function("generational-arena", |b| {
        b.iter_batched_ref(
            || {
                let mut ids = Vec::with_capacity(size);
                let mut s = generational_arena::Arena::new();
                for _ in 0..size {
                    ids.push(s.insert(Composed::default()));
                }
                (s, ids)
            },
            |(s, ids)| {
                for id in ids.iter() {
                    s.remove(*id);
                }
            },
            BatchSize::SmallInput,
        )
    });
    g.bench_function("shared-arena", |b| {
        b.iter_batched_ref(
            || {
                let mut ids = Vec::with_capacity(size);
                let s = shared_arena::Arena::new();
                for _ in 0..size {
                    ids.push(s.alloc(Composed::default()));
                }
                (s, ids)
            },
            |(_, ids)| {
                for _ in 0..size {
                    ids.pop();
                }
            },
            BatchSize::SmallInput,
        )
    });
    g.bench_function("froggy", |b| {
        b.iter_batched_ref(
            || {
                let mut ids = Vec::with_capacity(size);
                let mut s = froggy::Storage::new();
                for _ in 0..size {
                    ids.push(s.create(Composed::default()));
                }
                (s, ids)
            },
            |(_, ids)| {
                for _ in 0..size {
                    ids.pop();
                }
            },
            BatchSize::SmallInput,
        )
    });
    g.bench_function("hecs", |b| {
        b.iter_batched_ref(
            || {
                let mut ids = Vec::with_capacity(size);
                let mut s = hecs::World::new();
                for _ in 0..size {
                    ids.push(s.spawn(Composed::default()));
                }
                (s, ids)
            },
            |(s, ids)| {
                for id in ids.iter() {
                    let _ = s.remove::<Composed>(*id);
                }
            },
            BatchSize::SmallInput,
        )
    });
    g.bench_function("shipyard", |b| {
        b.iter_batched_ref(
            || {
                let mut ids = Vec::with_capacity(size);
                let mut s = shipyard::World::new();
                for _ in 0..size {
                    ids.push(s.add_entity(Composed::default()));
                }
                (s, ids)
            },
            |(s, ids)| {
                for id in ids.iter() {
                    let _ = s.remove::<Composed>(*id);
                }
            },
            BatchSize::SmallInput,
        )
    });
    g.bench_function("edict", |b| {
        b.iter_batched_ref(
            || {
                let mut ids = Vec::with_capacity(size);
                let mut s = edict::World::new();
                for _ in 0..size {
                    ids.push(s.spawn(Composed::default()));
                }
                (s, ids)
            },
            |(s, ids)| {
                for id in ids.iter() {
                    let _ = s.remove::<Composed>(*id);
                }
            },
            BatchSize::SmallInput,
        )
    });
    g.bench_function("nitro", |b| {
        b.iter_batched_ref(
            || {
                let mut ids = Vec::with_capacity(size);
                let mut s = nitro::Storage::new();
                for _ in 0..size {
                    ids.push(s.place(Composed::default()));
                }
                (s, ids)
            },
            |(s, ids)| {
                for id in ids.iter() {
                    s.remove::<Composed>(id);
                }
            },
            BatchSize::SmallInput,
        )
    });
}

fn get(c: &mut Criterion) {
    let size = 10_000;
    let mut rng = thread_rng();
    let indexes = Vec::from_iter((0..size).map(|_| rng.gen_range(0..size)));

    let mut g = c.benchmark_group("Get");
    g.bench_function("vec", |b| {
        b.iter_batched_ref(
            || {
                let mut s = Vec::with_capacity(size);
                for _ in 0..size {
                    s.push(Composed::default());
                }
                s
            },
            |s| {
                for i in indexes.iter() {
                    black_box(s.get(*i));
                }
            },
            BatchSize::SmallInput,
        )
    });
    g.bench_function("generational-arena", |b| {
        b.iter_batched_ref(
            || {
                let mut ids = Vec::with_capacity(size);
                let mut s = generational_arena::Arena::new();
                for _ in 0..size {
                    ids.push(s.insert(Composed::default()));
                }
                (s, ids)
            },
            |(s, ids)| {
                for i in indexes.iter() {
                    black_box(s.get(ids[*i]));
                }
            },
            BatchSize::SmallInput,
        )
    });
    g.bench_function("shared-arena", |b| {
        b.iter_batched_ref(
            || {
                let mut ids = Vec::with_capacity(size);
                let s = shared_arena::Arena::new();
                for _ in 0..size {
                    ids.push(s.alloc(Composed::default()));
                }
                (s, ids)
            },
            |(_, ids)| {
                for i in indexes.iter() {
                    black_box(&ids[*i]);
                }
            },
            BatchSize::SmallInput,
        )
    });
    g.bench_function("froggy", |b| {
        b.iter_batched_ref(
            || {
                let mut ids = Vec::with_capacity(size);
                let mut s = froggy::Storage::new();
                for _ in 0..size {
                    ids.push(s.create(Composed::default()));
                }
                (s, ids)
            },
            |(s, ids)| {
                for i in indexes.iter() {
                    black_box(&s[&ids[*i]]);
                }
            },
            BatchSize::SmallInput,
        )
    });
    g.bench_function("hecs", |b| {
        b.iter_batched_ref(
            || {
                let mut ids = Vec::with_capacity(size);
                let mut s = hecs::World::new();
                for _ in 0..size {
                    ids.push(s.spawn(Composed::default()));
                }
                (s, ids)
            },
            |(s, ids)| {
                for i in indexes.iter() {
                    let _ = black_box(s.get::<&Composed>(ids[*i]));
                }
            },
            BatchSize::SmallInput,
        )
    });
    g.bench_function("shipyard", |b| {
        b.iter_batched_ref(
            || {
                let mut ids = Vec::with_capacity(size);
                let mut s = shipyard::World::new();
                for _ in 0..size {
                    ids.push(s.add_entity(Composed::default()));
                }
                (s, ids)
            },
            |(s, ids)| {
                for i in indexes.iter() {
                    let rects = s.borrow::<shipyard::View<Rect>>().unwrap();
                    let _ = black_box(rects.get(ids[*i]).unwrap());
                }
            },
            BatchSize::SmallInput,
        )
    });

    g.bench_function("edict", |b| {
        b.iter_batched_ref(
            || {
                let mut ids = Vec::with_capacity(size);
                let mut s = edict::World::new();
                for _ in 0..size {
                    ids.push(s.spawn(Composed::default()));
                }
                (s, ids)
            },
            |(s, ids)| {
                for i in indexes.iter() {
                    let _ = black_box(s.query_one::<&Composed>(ids[*i]));
                }
            },
            BatchSize::SmallInput,
        )
    });
    g.bench_function("nitro", |b| {
        b.iter_batched_ref(
            || {
                let mut ids = Vec::with_capacity(size);
                let mut s = nitro::Storage::new();
                for _ in 0..size {
                    ids.push(s.place(Composed::default()));
                }
                (s, ids)
            },
            |(s, ids)| {
                for i in indexes.iter() {
                    let _ = black_box(s.get::<Composed>(&ids[*i]));
                }
            },
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, insert, remove, get);
criterion_main!(benches);
