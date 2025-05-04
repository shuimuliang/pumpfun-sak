use criterion::{criterion_group, criterion_main, Criterion};
use redis::{Commands, Connection};

fn redis_rpush_lpop(conn: &mut Connection, queue: &str, value: &str) {
    // RPUSH operation
    conn.rpush::<&str, &str, ()>(queue, value).unwrap();

    // LPOP operation
    let _: String = conn.lpop(queue, None).unwrap();
}

fn criterion_benchmark_redis_queue(c: &mut Criterion) {
    // let rt = Runtime::new().unwrap();
    let redis_url = "redis://127.0.0.1/";
    let queue = "benchmark_queue";
    let value = "test_value";

    let client = redis::Client::open(redis_url).unwrap();
    let mut conn = client.get_connection().unwrap();

    c.bench_function("redis_rpush_lpop", |b| {
        b.iter(|| redis_rpush_lpop(&mut conn, queue, value));
    });
}

criterion_group!(benches, criterion_benchmark_redis_queue);
criterion_main!(benches);
