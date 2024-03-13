use crate::nodes::{Coordinator, Message, Worker};
use crate::util::{
    decode_coordinator, decode_worker, flatten_3d_array, generate_test_input, test_equal,
};
use chrono::prelude::*;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Instant;
use algo::QuantizedWeightUnit;
use algo::util::{pre_processing, read_and_store_image};

pub fn preparation_phase() {
    todo!()
} //distribute weight, analyse mapping,distribute coordinators,distribute workers write into files.
pub fn c_1_simulation(num_workers: u8) {
    // 创建一个消息发送者和多个消息接收者

    let (coordinator_sender, coordinator_receiver) = mpsc::channel::<Message<f32>>();
    let start_time = Instant::now();
    let mut handles = vec![];
    let mut worker_send_channel = vec![];
    for worker_id in 0..num_workers {
        let (worker_sender, worker_receiver) = mpsc::channel::<Message<f32>>();
        let coordinator_sender_clone = coordinator_sender.clone();
        let file_name = format!("./Simu/worker_{:?}.json", worker_id);
        let handle = thread::spawn(move || {
            let mut phase = 0;
            let mut buffer = Vec::new();
            // Worker线程的接收端
            loop {
                if phase >= 53 {
                    phase = 0
                };
                let mut worker = decode_worker(&file_name, phase, buffer).unwrap();
                println!(
                    "worker{:?} start receiving,time:{:?}",
                    worker_id,
                    start_time.elapsed()
                );
                worker.receive(&worker_receiver, worker_id);
                println!(
                    "worker{:?} finished receiving,time:{:?}",
                    worker_id,
                    start_time.elapsed()
                );
                if worker.status == true {
                    break;
                }
                if phase == 52 {
                    worker.adaptive_pooling();
                }
                buffer = worker.work(&coordinator_sender_clone, &worker_receiver, worker_id); //buffer is the data received while working
                phase += 1;
            }
            println!("worker{:?}, exited", worker_id);
        });

        // 主线程将Worker线程的发送端和句柄保存在Vec中
        handles.push(handle);
        worker_send_channel.push(worker_sender);
    }
    let file_name = "./Simu/Coordinator.json";
    let coordinator_handle = thread::spawn(move || {
        let mut phase = 0;
        loop {
            match decode_coordinator(file_name, phase) {
                Ok(mut coordinator) => {
                    coordinator.receive_and_send(
                        &coordinator_receiver,
                        &worker_send_channel,
                        num_workers,
                    );
                    println!("phase{:?} finished", phase);
                    phase += 1;
                }
                Err(me) => {
                    let coodinator = Coordinator {
                        mapping: vec![],
                        // operations: vec![],
                    };
                    let result_vec = coodinator.receive_and_terminate(
                        &coordinator_receiver,
                        &worker_send_channel,
                        num_workers,
                    );
                    println!("{:?}",result_vec);
                    // test_equal(result_vec);
                    break;
                }
            }
        }
    });
    handles.push(coordinator_handle);
    //intput
    let image = pre_processing(read_and_store_image(r"C:\Users\Lu JunYu\CLionProjects\Split_learning_microcontrollers_\Algorithms\images\img.png").unwrap());
    let input = flatten_3d_array(image);
    let num_per_cpu = ((224 * 224 * 3) as f32 / num_workers as f32).ceil() as u32;
    //jump start the simulation
    let mut count = 0;
    for i in 0..num_workers {
        let coordinator_sender_clone = coordinator_sender.clone();
        for j in 0..num_per_cpu {
            if count < input.len() {
                coordinator_sender_clone
                    .send(Message::Result(Some(input[count])))
                    .expect("Start failed");
            }
            count += 1;
        }
        coordinator_sender_clone
            .send(Message::Result(None))
            .expect("start failed");
    }
    // 等待所有Worker线程完成
    for handle in handles {
        handle.join().unwrap();
    }
} //start the simulation
pub fn c_1_simulation_quant(num_workers: u8) {
    // 创建一个消息发送者和多个消息接收者
    let residual_connections = vec![
        vec![10, 15], //10,15
        vec![20, 25], //20,25
        vec![25, 30], //25,30,
        vec![35, 40], //35,40
        vec![40, 45], //40,45
        vec![45, 50], //45,50
        vec![55, 60], //55,60
        vec![60, 65], //60,65
        vec![70, 75], //70,75
        vec![75, 80], //75,80
    ];
    let (coordinator_sender, coordinator_receiver) = mpsc::channel::<Message<u8>>();
    let start_time = Instant::now();
    let mut handles = vec![];
    let mut worker_send_channel = vec![];
    for worker_id in 0..num_workers {
        let (worker_sender, worker_receiver) = mpsc::channel::<Message<u8>>();
        let coordinator_sender_clone = coordinator_sender.clone();
        let file_name = format!("./Simu/worker_{:?}.json", worker_id);
        let handle = thread::spawn(move || {
            let mut phase = 0;
            let mut buffer = Vec::new();
            // Worker线程的接收端
            loop {
                if phase >= 6 {
                    phase = 0
                };
                let mut worker: Worker<QuantizedWeightUnit,u8> = decode_worker(&file_name, phase, buffer).unwrap();
                println!(
                    "worker{:?} start receiving,time:{:?}",
                    worker_id,
                    start_time.elapsed()
                );
                worker.receive_q(&worker_receiver, worker_id);
                println!(
                    "worker{:?} finished receiving,time:{:?}",
                    worker_id,
                    start_time.elapsed()
                );
                if worker.status == true {
                    break;
                }
                if phase == 52 {
                    worker.adaptive_pooling_q();
                }
                buffer = worker.work_q(&coordinator_sender_clone, &worker_receiver, worker_id); //buffer is the data received while working
                phase += 1;
            }
            println!("worker{:?}, exited", worker_id);
        });

        // 主线程将Worker线程的发送端和句柄保存在Vec中
        handles.push(handle);
        worker_send_channel.push(worker_sender);
    }
    let file_name = "./Simu/Coordinator.json";
    let coordinator_handle = thread::spawn(move || {
        let mut residual : Vec<u8> = Vec::new();
        let mut parameters_res : ((u8,u8,u8),(f32,f32,f32));
        let mut phase = 0;
        loop {
            match decode_coordinator(file_name, phase) {
                Ok(mut coordinator) => {
                    coordinator.receive_and_send_q(
                        &coordinator_receiver,
                        &worker_send_channel,
                        num_workers,
                    );
                    println!("phase{:?} finished", phase);
                    phase += 1;
                }
                Err(me) => {
                    let coodinator = Coordinator {
                        mapping: vec![],
                        // operations: vec![],
                    };
                    let result_vec = coodinator.receive_and_terminate_q(
                        &coordinator_receiver,
                        &worker_send_channel,
                        num_workers,
                    );
                    println!("{:?}",result_vec);
                    // test_equal(result_vec);
                    break;
                }
            }
        }
    });
    handles.push(coordinator_handle);
    //intput
    let image = pre_processing(read_and_store_image(r"C:\Users\Lu JunYu\CLionProjects\Split_learning_microcontrollers_\Algorithms\images\img.png").unwrap());
    let raw_input = flatten_3d_array(image);
    let input = raw_input.into_iter().map(|x| (x / 0.017818455 + 114.38545).round().clamp(0.,255.) as u8 ).collect::<Vec<u8>>(); //input quantization
    let num_per_cpu = ((224 * 224 * 3) as f32 / num_workers as f32).ceil() as u32;
    //jump start the simulation
    let mut count = 0;
    for i in 0..num_workers {
        let coordinator_sender_clone = coordinator_sender.clone();
        for j in 0..num_per_cpu {
            if count < input.len() {
                coordinator_sender_clone
                    .send(Message::Result(Some(input[count])))
                    .expect("Start failed");
            }
            count += 1;
        }
        coordinator_sender_clone
            .send(Message::Result(None))
            .expect("start failed");
    }
    // 等待所有Worker线程完成
    for handle in handles {
        handle.join().unwrap();
    }
} //start the simulation