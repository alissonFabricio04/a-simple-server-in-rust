use std::char::from_u32;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::thread;

use regex::Regex;

const BYTE_LENGTH: u8 = 8;

fn main() {
    TcpListener::bind("127.0.0.1:8080")
        .expect("Falha ao vincular o endereço")
        .incoming()
        .for_each(|result| match result {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_client(stream);
                });
            }
            Err(e) => {
                eprintln!("Erro ao aceitar a conexão: {} \n\n", e);
            }
        });
}

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    match stream.read(&mut buffer) {
        Ok(_) => {
            println!("Recebido {} bytes \n\n", buffer.len());
            let request = String::from_utf8_lossy(&buffer[..]);
            println!("{request} \n\n");

            let body =
                if let Some(line) = request.lines().collect::<Vec<&str>>().iter().rev().next() {
                    line.replace("\0", "")
                } else {
                    String::new()
                };

            let re = Regex::new(r"[ \[\]]").unwrap();
            let bytes: Vec<&str> = re
                .split(&body)
                .collect::<Vec<&str>>()
                .iter()
                .filter(|&s| !s.is_empty())
                .cloned()
                .collect();

            let string_concats_bytes = *bytes.get(0).expect("Bits invalidos");

            let mut final_message = String::new();

            let mut res_bytes_string = String::from(string_concats_bytes);
            for _ in 0..(string_concats_bytes.len() / usize::from(BYTE_LENGTH)) {
                // example:
                // 010011110110110001100001001000000110110101110101011011100110010001101111
                // 01001111
                // 0110110001100001001000000110110101110101011011100110010001101111

                let (byte, res) = res_bytes_string.split_at(usize::from(BYTE_LENGTH));

                let byte_in_number_repr = isize::from_str_radix(byte, 2).unwrap();
                let character = from_u32(byte_in_number_repr as u32).unwrap();

                final_message.push_str(&character.to_string());
                res_bytes_string = (&res).to_string();

                println!("\n\n")
            }

            save_bytes_in_file("data.bin", buffer);

            let response =
                format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\n {final_message}");
            if let Err(e) = stream.write_all(response.as_bytes()) {
                eprintln!("Erro ao enviar a resposta: {} \n\n", e);
            }
        }
        Err(e) => {
            eprintln!("Erro ao ler do stream: {} \n\n", e);
        }
    }
}

fn save_bytes_in_file(filename: &str, ref buffer: [u8; 1024]) {
    if !Path::new(filename).is_file() {
        File::create(filename.to_string())
            .expect("Não foi possível criar o arquivo")
            .write_all(buffer)
            .expect("Não foi possível escrever no arquivo");
    } else {
        OpenOptions::new()
            .append(true)
            .open(filename)
            .expect("Não foi possível abrir o arquivo")
            .write(buffer)
            .expect("Não foi possível escrever no arquivo");
    }
}
