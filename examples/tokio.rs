#[derive(Clone, Copy)]
enum Thread {
    Current,
    Multiple,
}

#[derive(Clone, Copy)]
enum Feature {
    None,
    Io,
    Timer,
    All,
}

async fn benchmark() {
    let begin = std::time::SystemTime::now();
    for _ in 0..100000 {
        tokio::task::spawn(async {});
    }
    let dur1 = begin.elapsed().unwrap();

    let begin = std::time::SystemTime::now();
    for _ in 0..100000 {
        tokio::task::spawn(async {}).await.unwrap();
    }
    let dur2 = begin.elapsed().unwrap();

    let begin = std::time::SystemTime::now();
    for _ in 0..100000 {
        tokio::task::yield_now().await;
    }
    let dur3 = begin.elapsed().unwrap();

    println!(
        " \tspawn {:3}ns, \tspawn + join {:5}ns, \tyeild {:3}ns",
        dur1.as_nanos() / 100000,
        dur2.as_nanos() / 100000,
        dur3.as_nanos() / 100000
    );
}

fn runtime(t: Thread, f: Feature) -> tokio::runtime::Runtime {
    let mut builder = match t {
        Thread::Current => {
            print!("current_thread::");
            tokio::runtime::Builder::new_current_thread()
        }
        Thread::Multiple => {
            print!("multi_thread::");
            tokio::runtime::Builder::new_multi_thread()
        }
    };
    match f {
        Feature::None => {
            print!("none\t:");
            builder.build().unwrap()
        }
        Feature::Io => {
            print!("io\t:");
            builder.enable_io().build().unwrap()
        }
        Feature::Timer => {
            print!("time\t:");
            builder.enable_time().build().unwrap()
        }
        Feature::All => {
            print!("all\t:");
            builder.enable_all().build().unwrap()
        }
    }
}

fn main() {
    let threads = [Thread::Current, Thread::Multiple];
    let features = [Feature::None, Feature::Io, Feature::Timer, Feature::All];
    for thread in threads {
        for feature in features {
            runtime(thread, feature).block_on(benchmark());
        }
    }
}
