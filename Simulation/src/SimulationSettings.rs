use std::sync::{Arc, mpsc, Mutex};
use std::thread;
use crate::nodes::Message;
use crate::{perform_work, util};
use crate::util::{decode_coordinator, decode_worker};

pub fn preparation_phase(){
    todo!()
} //distribute weight, analyse mapping,distribute coordinators,distribute workers write into files.
pub fn c_1_w60_simulation(){// 创建一个消息发送者和多个消息接收者

    let (coordinator_sender, coordinator_receiver) = mpsc::channel::<Message>();

    let mut handles = vec![];
    let mut worker_send_channel  = vec![];
    for _ in 0..60 {
        let (worker_sender, worker_receiver) = mpsc::channel::<Message>();
        let coordinator_sender_clone = coordinator_sender.clone();

        let handle = thread::spawn(move || {
            // Worker线程的接收端
            loop{
                let mut worker = decode_worker("todo");
                worker.receive(&worker_receiver);
                if worker.status == false { break; }
                worker.work(&coordinator_sender_clone,&worker_receiver)
            }
        });

        // 主线程将Worker线程的发送端和句柄保存在Vec中
        handles.push( handle);
        worker_send_channel.push(worker_sender);
    }

    let coordinator_handle = thread::spawn(move ||{
        loop{
            let mut coordinator = decode_coordinator("todo");
            coordinator.receive_and_send(&coordinator_receiver,&worker_send_channel,60);
        }
    });
    handles.push(coordinator_handle);
    // 等待所有Worker线程完成
    for handle in handles {
        handle.join().unwrap();
    }


}//start the simulation
