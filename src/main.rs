
use serialport::{available_ports, SerialPortType};
use std::collections::HashMap;


fn main() {
  println!("fanalog");

  let mut active_ports = HashMap::new();
  loop {
    // TODO check for newly-available ports
    if let Ok(avail_ports) = available_ports() {
	let mut cur_avail_ports = HashMap::new();
	for port in avail_ports {
 //           cur_avail_ports.insert ();

	  if !active_ports.contains_key(&port.port_name) {
            println!("inserted: {}",port.port_name );
          }
        }
    }     
    // 
  }

/*
    match available_ports() {
        Ok(ports) => {
            match ports.len() {
                0 => println!("No ports found."),
                1 => println!("Found 1 port:"),
                n => println!("Found {} ports:", n),
            };
            for p in ports {
                println!("  {}", p.port_name);
                match p.port_type {
                    SerialPortType::UsbPort(info) => {
                        println!("    Type: USB");
                        println!("    VID:{:04x} PID:{:04x}", info.vid, info.pid);
                        println!(
                            "     Serial Number: {}",
                            info.serial_number.as_ref().map_or("", String::as_str)
                        );
                        println!(
                            "      Manufacturer: {}",
                            info.manufacturer.as_ref().map_or("", String::as_str)
                        );
                        println!(
                            "           Product: {}",
                            info.product.as_ref().map_or("", String::as_str)
                        );
                    }
                    SerialPortType::BluetoothPort => {
                        println!("    Type: Bluetooth");
                    }
                    SerialPortType::PciPort => {
                        println!("    Type: PCI");
                    }
                    SerialPortType::Unknown => {
                        println!("    Type: Unknown");
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("{:?}", e);
            eprintln!("Error listing serial ports");
        }
    }

  */

}
