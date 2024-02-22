use std::collections::HashMap;
use std::fmt::format;
use algo::{Layer, operations};
use algo::operations::{distribute_weight, get_input_mapping, mark_end};
use std::fs;
use std::fs::{File, OpenOptions, read};
use std::io::{BufRead, BufReader, Write};
use serde_json::from_str;
use crate::nodes::{Coordinator, Worker};

pub fn distribute_mapping_weight(layers:HashMap<i32,Box<dyn Layer>>,number_of_workers: u8,input_shape:(usize,usize,usize),output_dir:String){
    if !fs::metadata(&output_dir).is_ok() {
        // If it doesn't exist, create the folder
        match fs::create_dir_all(&output_dir) {
            Ok(_) => println!("Folder created successfully"),
            Err(e) => eprintln!("Error creating folder: {}", e),
        }
    }
    let mut input_shape = vec![input_shape.0, input_shape.1, input_shape.2];
    for i in 1..=layers.len() {
        let layer = layers.get(&(i as i32)).expect("getting layer failed");
        let weight = distribute_weight(layer,number_of_workers);
        let raw_mapping = get_input_mapping(layer,number_of_workers,input_shape);
        let e_pos = mark_end(&raw_mapping, number_of_workers);
        let mappings = operations::analyse_mapping(
            raw_mapping.clone(),
            number_of_workers,
            number_of_workers,
            e_pos,
        );
        input_shape = layer.get_output_shape().into_iter().map(|x| x as usize).collect();
        println!("{:?}",input_shape);
        match layer.identify() {
            "Convolution" =>{
                for i in 0..number_of_workers{
                    let mut worker = Worker{
                        weights: weight[i as usize].clone(),
                        inputs: vec![],
                        status: false,
                    };
                    let serialized_worker = serde_json::to_string(&worker).unwrap();
                    let file_name = format!("worker_{:?}.json",i);
                    // Write the JSON string to a file
                    let mut file = OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open("./".to_string() + &output_dir + "/"  + &file_name)
                        .unwrap();
                    writeln!(file, "{}", serialized_worker).unwrap();
                }
                let mut coordinator = Coordinator{
                    mapping: mappings,
                    batch_norm: vec![],
                    operations: vec![],
                };
                let serialized_coordinator = serde_json::to_string(&coordinator).unwrap();
                let file_name = "Coordinator.json".to_string();
                let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("./".to_string() + &output_dir + "/" + &file_name)
                    .unwrap();
                writeln!(file, "{}", serialized_coordinator).unwrap();
            }
            "Batchnorm2d" => {
                let file_name = format!("Coordinator.json");
                let file_path = "./".to_string() + &output_dir + "/"  + &file_name;
                let file = File::open(&file_path).unwrap();
                let reader = BufReader::new(file);
                let lines : Vec<String> = reader.lines().map(|x| x.unwrap()).collect();
                if let Some(last_line) = lines.last() {
                    // Replace the last line with the new JSON
                    let mut coordinator:Coordinator = from_str(last_line).unwrap();
                    coordinator.batch_norm = weight[0][0].data.clone();
                    coordinator.operations.push(1);
                    let serialized_coordinator = serde_json::to_string(&coordinator).unwrap();
                    let updated_lines: Vec<String> = lines.into_iter().rev().skip(1).rev().chain(vec![serialized_coordinator]).collect();
                    // Open the file for writing, truncating it in the process
                    let mut file = OpenOptions::new().write(true).truncate(true).open(file_path).unwrap();
                    // Write the updated content back to the file
                    for line in updated_lines {
                        writeln!(&mut file, "{}", line).unwrap();
                    }
                }
            }
            "Relu6" => {
                let file_name = format!("Coordinator.json");
                let file_path = "./".to_string() + &output_dir + "/"  + &file_name;
                let file = File::open(&file_path).unwrap();
                let reader = BufReader::new(file);
                let lines : Vec<String> = reader.lines().map(|x| x.unwrap()).collect();
                if let Some(last_line) = lines.last() {
                    // Replace the last line with the new JSON
                    let mut coordinator:Coordinator = from_str(last_line).unwrap();
                    coordinator.operations.push(2);
                    let serialized_coordinator = serde_json::to_string(&coordinator).unwrap();
                    let updated_lines: Vec<String> = lines.into_iter().rev().skip(1).rev().chain(vec![serialized_coordinator]).collect();
                    // Open the file for writing, truncating it in the process
                    let mut file = OpenOptions::new().write(true).truncate(true).open(file_path).unwrap();
                    // Write the updated content back to the file
                    for line in updated_lines {
                        writeln!(&mut file, "{}", line).unwrap();
                    }
                }
            }
            _ => {}
        }
    }
}