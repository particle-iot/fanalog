
use serialport::{available_ports,  SerialPort, SerialPortType};
use std::collections::{HashMap};
// use clap::{App, AppSettings, Arg};
use std::env;
use std::thread;
use std::time::{Duration, SystemTime};
use std::ops::Sub;

/// Largest chunk of bytes to read from serial port in one read
const MAX_SERIAL_BUF_READ: usize = 2048;
/// Minimum acceptable baud rate
const MIN_BAUD_RATE:u32 = 115200;
/// How long to wait between checks for serial devices plugged / unplugged
const PERIODIC_CHECK_TIME:Duration =  Duration::from_millis(500);

/// Collect a list of ports that we're interested in
fn collect_available_ports(available_set: &mut HashMap<String, String>) {
  if let Ok(avail_ports) = available_ports() {
    for port in avail_ports {

      match port.port_type {
        SerialPortType::UsbPort(info) => {
          if !available_set.contains_key(&port.port_name) {
            //add to list
            let sn_str = info.serial_number.unwrap_or(port.port_name.clone());
            // println!("sn_str: {}", sn_str);
            available_set.insert(port.port_name.clone(), sn_str);
            // println!("adding: {} {}", port.port_name, sn_str);

            // println!("    Type: USB");
            // println!("    VID:{:04x} PID:{:04x}", info.vid, info.pid);
            // println!(
            //   "     Serial Number: {}",
            //   info.serial_number.as_ref().map_or("", String::as_str)
            // );
            // println!(
            //   "      Manufacturer: {}",
            //   info.manufacturer.as_ref().map_or("", String::as_str)
            // );
            // println!(
            //   "           Product: {}",
            //   info.product.as_ref().map_or("", String::as_str)
            // );

          }
        }
        _ => {
          //println!("   Ignored Port: {}", port.port_name);
        }

      }

    }
  }
}

/// Given a list of available ports, add and remove serial ports from the active pool
fn maintain_active_port_list(available_set: &HashMap<String, String>, active_ports: &mut HashMap<String, Box<dyn SerialPort>>) {
  // first remove any active ports that went inactive
  let mut remove_list = Vec::new();
  for (port_name, _port_info) in active_ports.iter() {
    if !available_set.contains_key(port_name) {
      //TODO need to close these ports first?
      remove_list.push(port_name.clone());
    }
  }
  for remove_name in remove_list {
    println!("removing: {}", remove_name);
    active_ports.remove(&remove_name);
  }

  // now add any newly-available items
  for (port_name, _device_id) in available_set {
    if !active_ports.contains_key(port_name) {

      // use zero timeout: read only what's immediately available in the serial buffer,
      // don't wait for additional data to arrive
      let port_res = serialport::new(port_name, MIN_BAUD_RATE)
        .timeout(Duration::from_millis(0))
        .open();
      if let Ok(port) = port_res {
        active_ports.insert(port_name.clone(), port);
        println!("added: {}", port_name);
      }
      else {
        eprintln!("couldn't open port: {}", port_name);
      }
    }
  }
}


fn main() {
  eprintln!("fanalog");
  let unknown_device_id = "device_id_unknown".to_string();
  let mut serial_buf: Vec<u8> = vec![0; MAX_SERIAL_BUF_READ];

  let endpoint_url_res = env::var("COLLECTOR_ENDPOINT_URL");
  if !endpoint_url_res.is_ok() {
    eprintln!("You must define an environment variable eg `export COLLECTOR_ENDPOINT_URL=foo`");
    return;
  }

  let collector_endpoint_url = endpoint_url_res.unwrap();
  let mut active_ports_list = HashMap::new();
  let mut last_port_maintenance = SystemTime::now().sub(PERIODIC_CHECK_TIME);

  let mut available_ports_list = HashMap::new();

  let client = reqwest::blocking::Client::new();

  loop {
    // periodic port list maintenance
    let sys_time = SystemTime::now();
    let maintanance_gap = sys_time.duration_since(last_port_maintenance).unwrap_or(PERIODIC_CHECK_TIME);
    if maintanance_gap >= PERIODIC_CHECK_TIME {
      available_ports_list = HashMap::new(); //clear list
      collect_available_ports(&mut available_ports_list);
      maintain_active_port_list(&available_ports_list, &mut active_ports_list);
      last_port_maintenance = sys_time;
    }

    //println!("available_ports_list: {:?}", available_ports_list);

    let mut msg_count: u32 = 0;
    // TODO parallelize? with eg crossbeam
    for (port_name, port) in &mut active_ports_list {
      // read timeout should be preconfigured to be as short as possible
      if let Ok(read_size) = port.read(serial_buf.as_mut_slice()) {
        if read_size > 0 {
          msg_count += 1;
          //TODO cache this device ID
          let device_id = available_ports_list.get(port_name).unwrap_or(&unknown_device_id);
          let log_line = String::from_utf8_lossy( &serial_buf[..read_size] );
          let body = format!("{} [{}]: {}", device_id,  read_size, log_line);

          let _res = client.post(&collector_endpoint_url)
            .body(body)
            .send();

        }
      }
    }

    if 0 == msg_count {
      thread::yield_now();
    }

  }


}



