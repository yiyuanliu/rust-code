#![feature(test)]

extern crate test;

#[cfg(test)]
mod serde {
    use serde::{Deserialize, Serialize};
    #[derive(Serialize, Deserialize, Debug)]
    struct Student<'a> {
        name: &'a str,
        age: u8,
        id: u64,
        parents: Vec<&'a str>,
    }

    fn gen_student() -> Student<'static> {
        Student {
            name: "Li Xiaoming",
            age: 24,
            id: 9527,
            parents: vec!["Li Daming", "Wang Xiaohong"],
        }
    }

    mod flexbuffer {
        use super::*;

        #[bench]
        fn serialize(bench: &mut test::Bencher) {
            bench.iter(|| {
                let student = super::gen_student();
                let mut flexbuffer = flexbuffers::FlexbufferSerializer::default();
                student.serialize(&mut flexbuffer).unwrap();
            })
        }

        #[bench]
        fn deserialize(bench: &mut test::Bencher) {
            let student = super::gen_student();
            let mut flexbuffer = flexbuffers::FlexbufferSerializer::default();
            student.serialize(&mut flexbuffer).unwrap();
            let data = flexbuffer.take_buffer();
            println!("serde::flexbuffer: {}bytes", data.len());
            bench.iter(|| {
                let student1 = flexbuffers::from_slice::<Student>(data.as_slice()).unwrap();
                assert_eq!(student.id, student1.id);
            })
        }
    }

    mod bincode {
        use super::*;
        extern crate bincode as bc;

        #[bench]
        fn serialize(bench: &mut test::Bencher) {
            bench.iter(|| {
                let student = super::gen_student();
                let _data = bc::serialize(&student).unwrap();
            })
        }

        #[bench]
        fn deserialize(bench: &mut test::Bencher) {
            let student = super::gen_student();
            let data = bc::serialize(&student).unwrap();
            println!("serde::bincode: {}bytes", data.len());
            bench.iter(|| {
                let student1: Student = bc::deserialize(&data[..]).unwrap();
                assert_eq!(student.id, student1.id);
            })
        }
    }
}

mod rkyv {
    use rkyv::{
        archived_root,
        ser::{serializers::AllocSerializer, Serializer},
        with::Boxed,
        Archive, Serialize,
    };

    #[derive(Archive, Serialize)]
    struct Parent<'a> {
        // need a wrap...
        #[with(Boxed)]
        name: &'a str,
    }

    #[derive(Archive, Serialize)]
    struct Student<'a> {
        #[with(Boxed)]
        name: &'a str,
        age: u8,
        id: u64,
        // rkyv support & T, serde only support &str
        #[with(Boxed)]
        parents: &'a [Parent<'a>],
    }

    fn gen_student() -> Student<'static> {
        Student {
            name: "Li Xiaoming",
            age: 24,
            id: 9527,
            parents: &[
                Parent { name: "Li Daming" },
                Parent {
                    name: "Wang Xiaohong",
                },
            ],
        }
    }

    #[bench]
    fn serialize(bench: &mut test::Bencher) {
        bench.iter(|| {
            let student = gen_student();
            let mut serializer = AllocSerializer::<64>::default();
            serializer.serialize_value(&student).unwrap();
        })
    }

    #[bench]
    fn deserialize(bench: &mut test::Bencher) {
        let student = gen_student();
        let mut serializer = AllocSerializer::<64>::default();
        assert_eq!(serializer.pos(), 0);
        serializer.serialize_value(&student).unwrap();
        let data = serializer.into_serializer().into_inner();
        println!("serde::bincode: {}bytes", data.len());
        bench.iter(|| {
            let student1 = unsafe { archived_root::<Student>(&data[..]) };
            assert_eq!(student.id, student1.id);
        })
    }
}
