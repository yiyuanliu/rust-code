#[derive(Clone, Copy)]
enum Thread {
    Current,
    Multiple,
}

#[derive(Clone, Copy)]
enum Future {
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
        " \tspawn {}ns, \tspawn + join {}ns, \tyeild {}ns",
        dur1.as_nanos() / 100000,
        dur2.as_nanos() / 100000,
        dur3.as_nanos() / 100000
    );
}

fn runtime(t: Thread, f: Future) -> tokio::runtime::Runtime {
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
        Future::None => {
            print!("none:");
            builder.build().unwrap()
        }
        Future::Io => {
            print!("io:");
            builder.enable_io().build().unwrap()
        }
        Future::Timer => {
            print!("time:");
            builder.enable_time().build().unwrap()
        }
        Future::All => {
            print!("all:");
            builder.enable_all().build().unwrap()
        }
    }
}

fn main() {
    let threads = [Thread::Current, Thread::Multiple];
    let features = [Future::None, Future::Io, Future::Timer, Future::All];
    for thread in threads {
        for feature in features {
            runtime(thread, feature).block_on(benchmark());
        }
    }
}
