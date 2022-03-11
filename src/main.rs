use serial2::SerialPort;
use serial2::*;
use std::error::Error;
use std::io;
//use std::path::Path;
//use std::io::IoSlice;
use std::path::PathBuf;
use std::{thread, time};
fn main() -> Result<(), Box<dyn Error>> {
    //-> Result<(), Box<dyn Error>>
    println!("Hello, pcuart1");
    let mut vports: Vec<PathBuf> = Vec::new(); // let v: Vec<i32> = Vec::new();
    match serial2::SerialPort::available_ports() {
        Err(e) => {
            eprintln!("Failed to enumerate serial ports: {}", e);
            std::process::exit(1);
        }
        Ok(ports) => {
            eprintln!("Found {} ports", ports.len());
            for port in ports {
                println!("{}", port.display());
                vports.push(port);
            }
        }
    }

    let mut input = String::new();
    let mut used_port = String::new();
    io::stdin().read_line(&mut input)?;
    let slice = &input[0..input.len() - 1];
    let read_num: usize = slice.parse()?;
    println!("num {}", read_num);
    if vports.len() >= read_num {
        used_port.push_str(
            vports
                .get(read_num)
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
                .as_str(),
        );
    } else {
        panic!("Error: Port not found");
    }
    println!("select port: {}", used_port);

    // open port
    {
        let port = SerialPort::open(used_port.as_str(), |mut x: Settings| {
            x.set_stop_bits(StopBits::Two);
            x.set_parity(Parity::None);
            x.set_flow_control(FlowControl::None);
            x.set_char_size(CharSize::Bits8);
            let _ch = x.set_baud_rate(4800);
            Ok(x)
        })?;

        println!("Port opened. Enter - next request, q - close program");

        'outer: loop {
            let tx_b: [u8; 2] = [0x58, 0xaa];
            let mut rx_b = [0; 40];
            //for v in tx_b {
            //        println!("{}", v);
            //    }
            //let vt = IoSlice::new(&tx_b[0..1]);
            //let _txc = port.write_vectored(&[vt])?;
            let _txc = port.write(&tx_b)?;
            //let mut vr = IoSlice::new(&tx_b[0..1]);
            thread::sleep(time::Duration::from_millis(500));
            let rres = port.read(&mut rx_b);
            let rxc = rres.unwrap_or(0);
            println!("count {}", rxc);

            for i in 0..rxc {
                print!("{:02X} ", rx_b.get(i).unwrap());
            }
            println!("");

            if rxc > 34 {
                // > 33
                /*
                rx_b[6] = 0x0f;
                rx_b[5] = 0xe1;
                rx_b[4] = 0xa2;
                rx_b[12] = 0x42;
                rx_b[11] = 0x41;
                rx_b[10] = 0x7c;
                rx_b[18] = 0x01;
                rx_b[17] = 0xdf;
                rx_b[16] = 0xbe;*/

                let mut ivol: u32 = 0;
                ivol |= (rx_b[12] as u32) << 16;
                ivol |= (rx_b[11] as u32) << 8;
                ivol |= rx_b[10] as u32;

                let mut icur: u32 = 0;
                icur |= (rx_b[6] as u32) << 16;
                icur |= (rx_b[5] as u32) << 8;
                icur |= rx_b[4] as u32;

                let mut ipow: u32 = 0;
                ipow |= (rx_b[18] as u32) << 16;
                ipow |= (rx_b[17] as u32) << 8;
                ipow |= rx_b[16] as u32;
                //println!("{:#X}", ivol);
                let vol = (ivol as f32) * 1.218 * (7.0 * 270.0 + 0.56) / (79931.0 * 0.56 * 1000.0);
                let cur = (icur as f32) * 1.218 / (324004.0 * 10.0) * 0.96;
                let pow = (ipow as f32) * 1.218 * 1.218 * (7.0 * 270.0 + 0.56)
                    / (4046.0 * (10.0 * 0.56) * 1000.0);

                println!(
                    "Voltage: {:.2}  Current: {:.3}  APower: {:.2}",
                    vol, cur, pow
                );
            } // rxc
            io::stdin().read_line(&mut input)?;
            if input.contains("q") {
                break 'outer;
            }
        } // loop
    } // close port after exit

    Ok(())
}
