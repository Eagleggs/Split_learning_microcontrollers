use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::mpsc;
use serde_json::from_str;
use algo::operations::Mapping;
use algo::WeightUnit;
use crate::nodes::{Coordinator, Message, Worker};


pub fn decode_u128(input: &Vec<u8>) -> Vec<usize> {
    let mut next_mcus = Vec::new();
    let mut offset = 0;
    for t in input {
        for i in 0..8 {
            if (t >> i) & 0b1 == 0b1 {
                next_mcus.push(offset + i)
            }
        }
        offset += 8;
    }
    next_mcus
}
pub fn coordinator_send(
    next_mcus: Vec<usize>,
    send: &Vec<mpsc::Sender<Message>>,
    val: f32,
    end_pos: &Vec<(u16, u8, u32)>,
    cur_phase: usize,
    count: u32,
) {
    next_mcus.into_iter().for_each(|x| {
        send[x].send(Message::Work(Some(val))).expect("Coordinator send failed");
        for e in end_pos {
            if e.0 == cur_phase as u16 && e.1 == x as u8 && e.2 == count {
                send[x].send(Message::Work(None)).expect("Coordinator send none failed");
            }
        }
    });
}
pub fn wait_for_signal(rec: &mpsc::Receiver<Message>){
    loop{
        match rec.recv() {
            Ok(Message::StartTransmission) => { break;}
            _ => {}
        }
    }
}
pub fn decode_worker(path: &str,line_number: usize) -> Result<Worker, Box<dyn std::error::Error>>{
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    for (index, l) in reader.lines().enumerate() {
        // Check if this is the desired line
        if index == line_number {
            let line = l?;
            // Parse the JSON from the line
            let worker: Worker = from_str(&line)?;
            return Ok(worker);
        }
    }
    // If the line is not found, return an error
    Err("Line not found in the JSON file")?
}
pub fn decode_coordinator(path: &str,line_number: usize) -> Result<Coordinator, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    for (index, l) in reader.lines().enumerate() {
        // Check if this is the desired line
        if index == line_number {
            let line = l?;
            // Parse the JSON from the line
            let coordinator: Coordinator = from_str(&line)?;
            return Ok(coordinator);
        }
    }
    // If the line is not found, return an error
    Err("Line not found in the JSON file")?
}
pub fn generate_test_input(width:usize,height:usize,channel:usize)->Vec<Vec<Vec<f32>>>{
    let mut input: Vec<Vec<Vec<f32>>> = vec![vec![vec![0.; width]; height]; 3];
    for c in 0..channel {
        for i in 0..height {
            for j in 0..width {
                input[c][i][j] = (c * width * height + i * height + j) as f32;
            }
        }
    }
    input
}
pub fn flatten_3d_array(arr: Vec<Vec<Vec<f32>>>) -> Vec<f32> {
    let mut flattened_vec = Vec::new();
    for i in 0..arr.len() {
        for j in 0..arr[0].len() {
            for k in 0..arr[0][0].len() {
                flattened_vec.push(arr[i][j][k]);
            }
        }
    }

    flattened_vec
}