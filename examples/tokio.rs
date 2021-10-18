use std::time::SystemTime;

const TOTAL_ITER: u128 = 100000;

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
    for _ in 0..TOTAL_ITER {
        tokio::task::spawn(async {});
    }
    let dur1 = begin.elapsed().unwrap();

    let begin = std::time::SystemTime::now();
    for _ in 0..TOTAL_ITER {
        tokio::task::spawn(async {}).await.unwrap();
    }
    let dur2 = begin.elapsed().unwrap();

    let begin = std::time::SystemTime::now();
    for _ in 0..TOTAL_ITER {
        tokio::task::yield_now().await;
    }
    let dur3 = begin.elapsed().unwrap();

    println!(
        " \tspawn {:3}ns, \tspawn + join {:5}ns, \tyeild {:3}ns",
        dur1.as_nanos() / TOTAL_ITER,
        dur2.as_nanos() / TOTAL_ITER,
        dur3.as_nanos() / TOTAL_ITER
    );
}

async fn bench_mpmc() {
    let begin = SystemTime::now();
    let total = TOTAL_ITER;
    let (send, recv) = async_channel::unbounded();
    tokio::task::spawn(async move {
        for i in 0..total {
            send.send(i).await.unwrap();
            tokio::task::yield_now().await;
        }
    });
    for i in 0..total {
        assert_eq!(recv.recv().await.unwrap(), i);
    }
    let duration = begin.elapsed().unwrap();
    println!("\tmpsc 4byte {:5}ns", duration.as_nanos() / total);
}

fn runtime(t: Thread, f: Feature) -> tokio::runtime::Runtime {
    let mut builder = match t {
        Thread::Current => {
            print!("\tcurrent_thread::");
            tokio::runtime::Builder::new_current_thread()
        }
        Thread::Multiple => {
            print!("\tmulti_thread::");
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

    println!("spawn:");
    println!("    mpmc:");
    for thread in threads {
        for feature in features {
            runtime(thread, feature).block_on(async move {
                tokio::task::spawn(bench_mpmc()).await.unwrap();
            });
        }
    }
    println!("    spawn & yield:");
    for thread in threads {
        for feature in features {
            runtime(thread, feature).block_on(async move { tokio::task::spawn(benchmark()).await.unwrap() });
        }
    }

    println!("block_on:");
    println!("    mpmc:");
    for thread in threads {
        for feature in features {
            runtime(thread, feature).block_on(bench_mpmc());
        }
    }
    println!("   yield & spawn:");
    for thread in threads {
        for feature in features {
            runtime(thread, feature).block_on(benchmark());
        }
    }
}
