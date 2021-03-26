
use serialport::{available_ports,  SerialPort, SerialPortType};
use std::collections::{HashMap};
use std::env;
use std::thread;
use std::time::{Duration, SystemTime};
use std::ops::Sub;
use std::collections::VecDeque;
use std::sync::{Mutex, Arc};

/// Largest chunk of bytes to read from serial port in one read
const MAX_SERIAL_BUF_READ: usize = 2048;
/// Minimum acceptable baud rate
const MIN_BAUD_RATE:u32 = 115200;
/// How long to wait between checks for serial devices plugged / unplugged
const PERIODIC_CHECK_TIME:Duration =  Duration::from_millis(500);
/// A minimal pause time to wait if there are no messages to send
const MINIMAL_PAUSE_TIME:Duration = Duration::from_millis(5);

/// Used to queue up log line reports and send them asynchronously to the log collector
struct AsyncLogReporter {
  report_queue: Mutex<VecDeque<String>>,
  client: reqwest::blocking::Client,
  target_url: String,
}

impl AsyncLogReporter {
  /// Add a report to the queue to be sent
  fn add_report(&self, report: &String) {
    if let Ok(mut queue) = self.report_queue.lock() {
     queue.push_back(report.clone());
    }
  }

  /// Continuously report queued logs to log server
  fn run_forever(&self) {
    loop {
      if let Ok(mut queue) = self.report_queue.lock() {
        if let Some(report) = queue.pop_front() {
          //println!("{}",report);
          let posting_res = self.client.post(&self.target_url)
            .body(report)
            .send();
          if posting_res.is_err() {
            eprintln!("reporting error: {:?}",posting_res);
          }
        }
      }
      else {
        thread::sleep(MINIMAL_PAUSE_TIME);
      }
    }
  }
}

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
        .timeout(Duration::from_millis(1))
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
  if collector_endpoint_url.len() < 4 {
    eprintln!("`COLLECTOR_ENDPOINT_URL` environment variable is empty: `{}`", collector_endpoint_url);
    return;
  }
  else {
    println!("COLLECTOR_ENDPOINT_URL valid!");
  }

  let client = reqwest::blocking::Client::new();

  let report_actor = AsyncLogReporter {
    report_queue: Default::default(),
    client,
    target_url: collector_endpoint_url,
  };

  let mut active_ports_list = HashMap::new();
  let mut last_port_maintenance = SystemTime::now().sub(PERIODIC_CHECK_TIME);
  let mut available_ports_list = HashMap::new();

  let reporter_wrap = Arc::new(report_actor);
  let bg_reporter = Arc::clone(&reporter_wrap);
  thread::spawn(move || {
    bg_reporter.run_forever();
  });

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

    let mut msg_count: u32 = 0;

    for (port_name, port) in &mut active_ports_list {
      // read timeout should be preconfigured to be as short as possible
      if let Ok(read_size) = port.read(serial_buf.as_mut_slice()) {
        if read_size > 0 {
          msg_count += 1;
          //TODO cache this device ID
          let device_id = available_ports_list.get(port_name).unwrap_or(&unknown_device_id);
          let log_line = String::from_utf8_lossy( &serial_buf[..read_size] );
          let time_since = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default();
          let body = format!("{} {}: {}", device_id,  time_since.as_millis(), log_line);
          reporter_wrap.add_report(&body);
        }
      }
    }

    // If there are no active ports, sleep for a while
    if 0 == active_ports_list.len() {
      thread::sleep(PERIODIC_CHECK_TIME);
    }
    else if 0 == msg_count {
      thread::sleep(MINIMAL_PAUSE_TIME);
    }

  }


}



