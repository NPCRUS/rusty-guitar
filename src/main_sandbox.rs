
#[derive(Debug)]
struct TestStruct {
    id: i8,
    name: String
}

fn main() {
    let mut list = vec![
        TestStruct { id: 1, name: "test1".to_owned()},
        TestStruct { id: 2, name: "test2".to_owned()}
    ];

    for entity in list.iter_mut() {
        entity.id = 3
    }

    debug!("{:?}", list);
}