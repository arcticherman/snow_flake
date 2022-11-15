mod snow_flake;
use std::vec;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
pub use snow_flake::SnowFlake;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut th_list: Vec<JoinHandle<()>> = Vec::new();
        let th_num= 2;
        let loop_num = 2;
        let arc_op_num = Arc::new(Mutex::new(0));
        let mut snow_flake = SnowFlake::new();
        for th_index in 0..th_num{
            let th_op_num = Arc::clone(&arc_op_num);
            let th = thread::spawn(move || {
                for i in 0..loop_num{
                    let mut op_num = th_op_num.lock().unwrap();
                    *op_num += 1;
                    // let uniq_id = snow_flake.next_uniq_id();
                    // println!("th_id:{:?} th i:{} uniq_id:{}", thread::current().id(), i, uniq_id);
                    println!("th_id:{:?} th i:{}", thread::current().id(), i);
                    thread::sleep(Duration::from_millis(1));
                }
            });
            th_list.push(th);
        }
        for th in th_list{
            th.join().unwrap();
        }
        let op_num = arc_op_num.lock().unwrap();
        println!("op_num:{}", *op_num);
        let uniq_num = 100000;
        let mut now = SystemTime::now();
        let st_ms = now.duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
        for i in 0..uniq_num{
            let uniq_id = snow_flake.next_uniq_id();
            // println!("uniq_id:{:b}", uniq_id);
        }
        now = SystemTime::now();
        let ed_ms = now.duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
        println!("Use {} ms to generate {} unique identifiers .", (ed_ms - st_ms), uniq_num);
        assert_eq!(*op_num, th_num * loop_num);
    }
}
