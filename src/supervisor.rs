use crate::utils::timedate;
use serde::{Deserialize, Serialize};
use xmlrpc::Request;
use std::cmp::max;

#[derive(Debug, Deserialize, Serialize)]
pub struct Process {
    name: String,
    state: String,
    process_name: String,
    pid: i32,
    uptime: String,
}

pub struct SupervisorService {
    server_url: String,
}

impl SupervisorService {
    pub fn new(server_url: String) -> Self {
        SupervisorService { server_url }
    }

    pub fn process_list(&self) -> Vec<Process> {
        let request = Request::new("supervisor.getAllProcessInfo");
        let response = request.call_url(&self.server_url).unwrap();

        let response: Vec<Process> = response
            .as_array()
            .unwrap()
            .to_vec()
            .iter()
            .map(|value| {
                let value = value.as_struct().unwrap();
                let state = value
                    .get("statename")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string();

                let start_time = value.get("start").unwrap().as_i64().unwrap();
                let stop_time = value.get("stop").unwrap().as_i64().unwrap();

                let now_time = value.get("now").unwrap().as_i64().unwrap();
                let uptime = timedate::diff_for_humans(max(start_time, stop_time), now_time);

                Process {
                    name: value.get("group").unwrap().as_str().unwrap().to_string(),
                    state,
                    process_name: value.get("name").unwrap().as_str().unwrap().to_string(),
                    pid: value.get("pid").unwrap().as_i32().unwrap(),
                    uptime,
                }
            })
            .collect();

        response
    }

    pub fn start_process(&self, process_name: String) -> bool {
        let request = Request::new("supervisor.startProcessGroup").arg(process_name.clone());
        let response = request.call_url(&self.server_url);

        match response {
            Ok(_) => true,
            Err(error) => {
                log::error!(
                    "Error in start process {}. message: {}",
                    process_name,
                    error.to_string()
                );
                return false;
            }
        }
    }

    pub fn start_all_process(&self) -> bool {
        let request = Request::new("supervisor.startAllProcesses");
        let response = request.call_url(&self.server_url);

        match response {
            Ok(_) => true,
            Err(error) => {
                log::error!(
                    "Error in start all process's. message: {}",
                    error.to_string()
                );
                return false;
            }
        }
    }

    pub fn stop_process(&self, process_name: String) -> bool {
        let request = Request::new("supervisor.stopProcessGroup").arg(process_name.clone());
        let response = request.call_url(&self.server_url);

        match response {
            Ok(_) => true,
            Err(error) => {
                log::error!(
                    "Error in stop process {}. message: {}",
                    process_name,
                    error.to_string()
                );
                return false;
            }
        }
    }

    pub fn stop_all_process(&self) -> bool {
        let request = Request::new("supervisor.stopAllProcesses");
        let response = request.call_url(&self.server_url);

        match response {
            Ok(_) => true,
            Err(error) => {
                log::error!(
                    "Error in stop all process's. message: {}",
                    error.to_string()
                );
                return false;
            }
        }
    }

    pub fn reload_supervisor(&self) -> bool {
        let request = Request::new("supervisor.startProcess");
        let response = request.call_url(&self.server_url);

        match response {
            Ok(_) => true,
            Err(error) => {
                log::error!("Error in reload supervisor. message: {}", error.to_string());
                return false;
            }
        }
    }
}
