use std::collections::HashMap;

use anyhow::Result;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use kvs::KvsEngine;
use rand::{thread_rng, Rng, RngCore};
const SIMPLE:usize = 10000;

pub fn criterion_benchmark(c: &mut Criterion) -> Result<()> {
    let mut kv = kvs::KvStore::open(".")?;
    let mut sled = kvs::SledEngine::open(".")?;
    let mut map = HashMap::new();
    for _ in 0..SIMPLE + 10 {
        let mut rng = thread_rng();
        let n_key: u32 = rng.gen_range(0, 100000);
        let n_value: u32 = rng.gen_range(0, 100000);
        let mut data_key = [0u8; 100000];
        let mut data_value = [0u8; 100000];
        rng.fill_bytes(&mut data_key);
        rng.fill_bytes(&mut data_value);
        let mut s_key = String::new();
        let mut s_value = String::new();
        for i in 0..n_key {
            let x = data_key[i as usize];
            s_key.push(x as char);
        }
        for i in 0..n_value {
            let x = data_value[i as usize];
            s_value.push(x as char);
        }
        map.insert(s_key.clone(), s_value.clone());
    }
    c.bench_function("kvs_write", |b| {
        b.iter(|| {
            let mut rng = thread_rng();
            let n = rng.gen_range(0, SIMPLE);
            let mut iter = map.keys().into_iter();
            let mut x = iter.next().unwrap();
            for _ in 0..n {
                x = iter.next().unwrap();
            }
            let x = x;
            let value = map.get(x);
            kv.set(x.clone(), value.unwrap().clone()).unwrap();
        })
    });

    c.bench_function("kvs_read", |b| {
        b.iter(|| {
            let mut rng = thread_rng();
            let n = rng.gen_range(0, SIMPLE);
            let mut iter = map.keys().into_iter();
            let mut x = iter.next().unwrap();
            for _ in 0..n {
                x = iter.next().unwrap();
            }
            let x = x;
            let _v = map.get(x).unwrap();
            let _val = kv.get(x.clone()).unwrap();
            // assert_eq!(val, v.clone())
        })
    });

    c.bench_function("sled_write", |b| {
        b.iter(|| {
            let mut rng = thread_rng();
            let n = rng.gen_range(0, SIMPLE);
            let mut iter = map.keys().into_iter();
            let mut x = iter.next().unwrap();
            for _ in 0..n {
                x = iter.next().unwrap();
            }
            let x = x;
            let value = map.get(x);
            sled.set(x.clone(), value.unwrap().clone()).unwrap();
        })
    });

    c.bench_function("sled_read", |b| {
        b.iter(|| {
            let mut rng = thread_rng();
            let n = rng.gen_range(0, SIMPLE);
            let mut iter = map.keys().into_iter();
            let mut x = iter.next().unwrap();
            for _ in 0..n {
                x = iter.next().unwrap();
            }
            let x = x;
            let _v = map.get(x).unwrap();
            let _val = sled.get(x.clone()).unwrap();
            // assert_eq!(val, v.clone())
        })
    });
    Ok(())
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
