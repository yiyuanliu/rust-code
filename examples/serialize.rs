const ITERATIONS: usize = 100000;

mod serde {
    use serde::{Deserialize, Serialize};
    #[derive(Serialize, Deserialize, Debug)]
    struct Student<'a> {
        name: &'a str,
        age: u8,
        id: u64,
        parents: Vec<&'a str>,
    }

    pub fn flexbuffer(name: &str, parents: &[&str]) {
        let student = Student {
            name,
            age: 26,
            id: 9527,
            parents: parents.to_owned(),
        };

        let begin = std::time::SystemTime::now();
        for _ in 0..super::ITERATIONS {
            let mut flexbuffer = flexbuffers::FlexbufferSerializer::default();
            student.serialize(&mut flexbuffer).unwrap();
        }
        let dur_serialize = begin.elapsed().unwrap();

        let mut flexbuffer = flexbuffers::FlexbufferSerializer::default();
        student.serialize(&mut flexbuffer).unwrap();
        let data = flexbuffer.take_buffer();
        let data_len = data.len();

        let begin = std::time::SystemTime::now();
        for _ in 0..super::ITERATIONS {
            let student1 = flexbuffers::from_slice::<Student>(data.as_slice()).unwrap();
            assert_eq!(student.id, student1.id);
        }
        let dur_deserialize = begin.elapsed().unwrap();
        println!(
            "serde + flexbuffer: data len: {}bytes, serialize: {}ns, deserialize: {}ns",
            data_len,
            dur_serialize.as_nanos() / (super::ITERATIONS as u128),
            dur_deserialize.as_nanos() / (super::ITERATIONS as u128),
        );
    }

    pub fn bincode(name: &str, parents: &[&str]) {
        let student = Student {
            name,
            age: 26,
            id: 9527,
            parents: parents.to_owned(),
        };

        let begin = std::time::SystemTime::now();
        for _ in 0..super::ITERATIONS {
            let data = bincode::serialize(&student).unwrap();
            assert!(data.len() != 0);
        }
        let dur_serialize = begin.elapsed().unwrap();

        let data = bincode::serialize(&student).unwrap();
        let data_len = data.len();

        let begin = std::time::SystemTime::now();
        for _ in 0..super::ITERATIONS {
            let student1: Student = bincode::deserialize(&data[..]).unwrap();
            assert_eq!(student.id, student1.id);
        }
        let dur_deserialize = begin.elapsed().unwrap();
        println!(
            "serde + bincode: data len: {}bytes, serialize: {}ns, deserialize: {}ns",
            data_len,
            dur_serialize.as_nanos() / (super::ITERATIONS as u128),
            dur_deserialize.as_nanos() / (super::ITERATIONS as u128),
        );
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

    pub fn benchmark(name: &str, parents: &[&str]) {
        let student = Student {
            name,
            age: 26,
            id: 9527,
            parents: unsafe { std::mem::transmute(parents) },
        };

        let begin = std::time::SystemTime::now();
        for _ in 0..super::ITERATIONS {
            let mut serializer = AllocSerializer::<64>::default();
            serializer.serialize_value(&student).unwrap();
        }
        let dur_serialize = begin.elapsed().unwrap();

        let mut serializer = AllocSerializer::<64>::default();
        assert_eq!(serializer.pos(), 0);
        serializer.serialize_value(&student).unwrap();
        let data_len = serializer.pos();
        let data = serializer.into_serializer().into_inner();

        let begin = std::time::SystemTime::now();
        for _ in 0..super::ITERATIONS {
            let student1 = unsafe { archived_root::<Student>(&data[0..data_len]) };
            assert_eq!(student.id, student1.id);
        }
        let dur_deserialize = begin.elapsed().unwrap();
        println!(
            "rkyv: data len: {}bytes, serialize: {}ns, deserialize: {}ns",
            data_len,
            dur_serialize.as_nanos() / (super::ITERATIONS as u128),
            dur_deserialize.as_nanos() / (super::ITERATIONS as u128),
        );
    }
}

fn main() {
    let name = "Li Xiaoming";
    let parents = ["Li Daming", "Wang Xiaohong"];
    serde::flexbuffer(name, &parents);
    serde::bincode(name, &parents);
    rkyv::benchmark(name, &parents);
}
