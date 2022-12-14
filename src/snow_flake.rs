use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::atomic::{Ordering, AtomicU64};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
/*
    Snowflake algorithm to generate unique identifiers in distributed systems .
    The unique identifier is an unsigned 64 bits integer.
    [1, 43] bits means the current UNIX timestamp in milliseconds . The max timestamp is 2^46-1=70368744177663. So the max date is 4199-11-24 .
    [44, 51] bits means the identifier of the server . So max identifier of server is 2^8-1=255.
    [52, 64] bits means the serial number in the same millisecond . So the number we can generate in one millisecond is 2^13=8191 .
*/
// #[derive(Clone)]
pub struct SnowFlake{
    server_shift:   i32,
    serial_shift:   i32,
    time_shift:     i32,
    last_ms:        u64,
    serial_arc:     Arc<Mutex<u64>>,
    server_id:      u64,
    max_serial_num: u64,
    id_num:         AtomicU64,
    atomic_last_ms: AtomicU64,
}

impl SnowFlake {
    pub fn new() -> SnowFlake{
        let server_shift =  8;
        let serial_shift = 13;
        // let serial_shift = 12;
        let snow_flake =  SnowFlake{
            server_shift: server_shift,
            serial_shift: serial_shift,
            time_shift: server_shift + serial_shift,
            last_ms: 0,
            serial_arc:  Arc::new(Mutex::new(0)),
            server_id: 0,
            max_serial_num: (1<<serial_shift) -1,
            id_num: AtomicU64::new(0),
            atomic_last_ms: AtomicU64::new(0),
        };
        return snow_flake;
    }
    
    /*
        @brief generate uniq_id with no lock
     */
    pub fn no_lock_uniq_id(&mut self) -> u64{
        let now = SystemTime::now();
        let now_ms = now.duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
        let max_try_time = 10;
        let mut flag = false;
        let mut serial_id:u64 = 0;
        for try_time in 0..max_try_time {
            serial_id = self.id_num.fetch_add(1, Ordering::Relaxed);
            if serial_id > self.max_serial_num{
                //???????????????0??????
                let last_ms2 = self.atomic_last_ms.load(Ordering::Relaxed);
                if now_ms == last_ms2{
                    //??????????????????????????????
                    if serial_id == self.max_serial_num + 1{
                        let now_micros = now.duration_since(UNIX_EPOCH).unwrap().as_micros() as u64;
                        thread::sleep(Duration::from_micros(1000-(now_micros%1000)));
                    }
                }
                self.id_num.store(0, Ordering::Relaxed);
            }else{
                if self.atomic_last_ms.load(Ordering::Relaxed) != now_ms{
                    self.id_num.store(0, Ordering::Relaxed);
                }
                flag = true;
                break;
            }
        }
        if !flag {
            return 0;
        }
        let uniq_id: u64 = (now_ms << (self.time_shift)) | (self.server_id << self.serial_shift) | serial_id;
        self.atomic_last_ms.store(now_ms, Ordering::Relaxed);
        return uniq_id;
    }

    pub fn next_uniq_id(&mut self) -> u64{
        let mut serial = self.serial_arc.lock().unwrap();
        let now = SystemTime::now();
        let now_ms = now.duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
        let now_micros = now.duration_since(UNIX_EPOCH).unwrap().as_micros() as u64;
        let mut serial_id = 0;
        if self.last_ms == now_ms{            
            if *serial < self.max_serial_num{
                *serial += 1;
            }else{
                thread::sleep(Duration::from_micros(1000-(now_micros%1000)));
                *serial = 0;
            }
            serial_id = *serial;
        }else{
            *serial = 0;
        }
        let uniq_id: u64 = (now_ms << (self.time_shift)) | (self.server_id << self.serial_shift) | serial_id;
        // println!("uniq_id:{} now_ms:{} serial_id:{} self.server_id:{}", uniq_id, now_ms, serial_id, self.server_id);
        self.last_ms = now_ms;
        return  uniq_id;
    }
}