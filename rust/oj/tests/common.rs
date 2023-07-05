use assert_json_diff::{assert_json_matches_no_panic, CompareMode, Config};
use lazy_static::lazy_static;
use reqwest::blocking::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use serde_json;
use serde_json::Value;
use std::env::consts::EXE_EXTENSION;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Once;
use std::time::Duration;

// The code was originally written by Jack O'Connor (@oconnor663)
// Taken from https://github.com/oconnor663/os_pipe.rs/blob/f41c58e503e1efc5e4d0edfcd2e756b3a81b4232/src/lib.rs#L281-L314
// Downloaded at Aug 13, 2022
// Licensed under:
// The MIT License (MIT)
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.
fn build_and_find_path(name: &str) -> PathBuf {
    // This project defines some associated binaries for testing, and we shell out to them in
    // these tests. `cargo test` doesn't automatically build associated binaries, so this
    // function takes care of building them explicitly, with the right debug/release flavor.
    static CARGO_BUILD_ONCE: Once = Once::new();
    CARGO_BUILD_ONCE.call_once(|| {
        let mut build_command = Command::new("cargo");
        build_command.args(&["build", "--quiet"]);
        if !cfg!(debug_assertions) {
            build_command.arg("--release");
        }
        let build_status = build_command.status().unwrap();
        assert!(
            build_status.success(),
            "Cargo failed to build associated binaries."
        );
    });
    let flavor = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    Path::new("target")
        .join(flavor)
        .join(name)
        .with_extension(EXE_EXTENSION)
}

lazy_static! {
    static ref EXE_PATH: PathBuf = build_and_find_path("oj");
    static ref CLIENT: Client = Client::new();
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct TestRequest {
    path: String,
    method: String,
    content: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct TestResponse {
    status: u16,
    content: Value,
}

fn _default_true() -> bool {
    true
}
fn _default_false() -> bool {
    false
}
fn _default_timeout() -> u64 {
    3000
}
fn _default_poll_count() -> u64 {
    5
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct HttpComm {
    request: TestRequest,
    response: TestResponse,
    #[serde(default = "_default_timeout")]
    timeout: u64,
    #[serde(default = "_default_false")]
    poll_for_job: bool, // should be true in job submission request, except in non-blocking test
    #[serde(default = "_default_poll_count")]
    poll_count: u64,
    #[serde(default = "_default_false")]
    restart_server: bool, // restart server before sending request
}

pub struct TestCase {
    name: String,
    arguments: Vec<String>,
    data: Vec<HttpComm>, // a sequence of HTTP requests and responses
    prefix: String,      // the prefix of the path of the HTTP requests
    running_process: Option<Child>,
    stdout_file: PathBuf,
    stderr_file: PathBuf,
    http_file: PathBuf,
}

impl TestCase {
    pub fn read(name: &str) -> Self {
        let case_dir = Path::new("tests").join("cases");
        let config_file = case_dir.join(format!("{}.config.json", name));
        let data_file = case_dir.join(format!("{}.data.json", name));
        let stdout_file = case_dir.join(format!("{}.stdout", &name));
        let stderr_file = case_dir.join(format!("{}.stderr", &name));
        let http_file = case_dir.join(format!("{}.http", &name));
        std::fs::remove_file(&http_file).ok();

        let config: Value = serde_json::from_reader(File::open(&config_file).unwrap()).unwrap();
        let prefix = format!(
            "http://{}:{}",
            config["server"]["bind_address"].as_str().unwrap(),
            config["server"]["bind_port"]
        );

        Self {
            name: name.to_string(),
            arguments: vec![
                "--config".to_string(),
                config_file.to_str().unwrap().to_string(),
                "--flush-data".to_string(),
            ],
            data: serde_json::from_reader(File::open(data_file).unwrap()).unwrap(),
            prefix,
            running_process: None,
            stdout_file,
            stderr_file,
            http_file,
        }
    }

    fn log_and_send(
        &self,
        req: RequestBuilder,
    ) -> reqwest::Result<(reqwest::blocking::Response, File)> {
        let req = req.build()?;
        let mut http_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.http_file)
            .unwrap();

        writeln!(http_file, "###").ok();
        writeln!(http_file, "# Send request:").ok();
        writeln!(http_file, "{} {} HTTP/1.1", req.method(), req.url()).ok();
        for (name, value) in req.headers() {
            writeln!(http_file, "{}: {}", name, value.to_str().unwrap()).ok();
        }

        writeln!(http_file).ok();

        if let Some(body) = req.body().and_then(|b| b.as_bytes()) {
            http_file.write_all(body).ok();
        }

        writeln!(http_file).ok();

        let res = CLIENT.execute(req);

        match res {
            Ok(resp) => {
                writeln!(http_file, "###").ok();
                writeln!(http_file, "# Got response:").ok();
                writeln!(http_file, "# HTTP {}", resp.status().as_u16()).ok();
                for (name, value) in resp.headers() {
                    writeln!(http_file, "# {}: {}", name, value.to_str().unwrap()).ok();
                }
                writeln!(http_file, "# ").ok();
                write!(http_file, "# ").ok();

                Ok((resp, http_file))
            }
            Err(err) => {
                writeln!(http_file, "###").ok();
                writeln!(http_file, "Got error: {:?}", err).ok();
                Err(err)
            }
        }
    }

    fn start_server(&mut self, restart: bool) {
        // ensure no server is running
        CLIENT
            .post(&format!("{}/internal/exit", self.prefix))
            .send()
            .ok();
        // sleep 1 second for server shutdown
        std::thread::sleep(Duration::from_secs(1));

        let mut open_option = OpenOptions::new();
        open_option.create(true);
        if restart {
            open_option.append(true);
        } else {
            open_option.write(true).truncate(true);
        }

        let stdout = open_option
            .open(&self.stdout_file)
            .expect("failed to create stdout file");
        let stderr = open_option
            .open(&self.stderr_file)
            .expect("failed to create stderr file");

        let command = Command::new(EXE_PATH.as_os_str())
            .args(&self.arguments)
            .stdin(Stdio::piped())
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr))
            .spawn()
            .expect(
                format!(
                    "case {} incorrect: failed to execute server process",
                    self.name
                )
                .as_str(),
            );
        self.running_process = Some(command);
        // sleep 1 second for server startup
        std::thread::sleep(Duration::from_secs(1));

        // check that server is running
        for retry in 0..3 {
            if let Err(err) = CLIENT.get(&self.prefix).send() {
                if retry < 2 {
                    std::thread::sleep(Duration::from_secs(1));
                    continue;
                } else {
                    panic!(
                        "case {} incorrect: failed to connect to OJ server ({})",
                        self.name, err
                    );
                }
            } else {
                break;
            }
        }
    }

    fn kill_server(&mut self) {
        if let Some(mut child) = self.running_process.take() {
            child.kill().expect(
                format!("case {} incorrect: cannot kill server process", self.name).as_str(),
            );
        }
    }

    fn send_request_and_compare_response(&mut self, c: &HttpComm) -> Value {
        if c.restart_server {
            self.kill_server();
            // remove --flush-data before restarting server, then add it back
            let old_arguments = self.arguments.clone();
            self.arguments.remove(
                self.arguments
                    .iter()
                    .position(|x| x == "--flush-data")
                    .unwrap(),
            );
            self.start_server(true);
            self.arguments = old_arguments;
        }

        let url = format!("{}/{}", &self.prefix, &c.request.path);
        let method =
            reqwest::Method::from_bytes(&c.request.method.to_uppercase().as_bytes()).unwrap();

        let check_status_and_get_body = |url: &str, method: reqwest::Method| -> Value {
            let mut request = CLIENT
                .request(method.clone(), url)
                .timeout(Duration::from_millis(c.timeout));
            if let reqwest::Method::GET = method {
                // no json body
            } else {
                request = request.json(&c.request.content);
            }

            let (resp, mut http_file) = self
                .log_and_send(request)
                .expect(format!("case {} incorrect: HTTP request failed", self.name).as_str());
            let code = resp.status().as_u16();
            assert_eq!(
                code, c.response.status,
                "case {} incorrect: wrong status code",
                self.name
            );
            let json: Value = resp.json().expect(
                format!(
                    "case {} incorrect: cannot decode response body as JSON, status code is {}",
                    self.name, code
                )
                .as_str(),
            );

            serde_json::to_writer(&http_file, &json).ok();
            writeln!(http_file).ok();
            json
        };
        let mut body = check_status_and_get_body(&url, method);

        let job_finished = |body: &Value| -> bool { body["state"] == "Finished" };

        // polling for job
        if c.poll_for_job && !job_finished(&body) {
            // find job id in response
            let job_id;
            if let Value::Number(id) = &body["id"] as &Value {
                job_id = id
                    .as_u64()
                    .expect(format!("case {} incorrect: job id is not valid", self.name).as_str());
            } else {
                panic!(
                    "case {} incorrect: cannot get job id after submission",
                    self.name
                );
            }

            // polling until job is finished
            let poll_url = format!("{}/jobs/{}", &self.prefix, job_id);
            for _ in 0..c.poll_count {
                std::thread::sleep(Duration::from_secs(1));
                body = check_status_and_get_body(&poll_url.as_str(), reqwest::Method::GET);
                if job_finished(&body) {
                    break;
                }
            }
            if !job_finished(&body) {
                panic!(
                    "case {} incorrect: polling too many times for job {}",
                    self.name, body["id"]
                );
            }
        }

        // check final result
        if let Err(error) = assert_json_matches_no_panic(
            &body,
            &c.response.content,
            Config::new(CompareMode::Inclusive),
        ) {
            panic!(
                "case {} incorrect: wrong response\n\n{}\n\n",
                self.name, error
            );
        }
        body
    }

    pub fn run(&mut self) -> Vec<Value> {
        self.start_server(false);
        // send requests sequentially
        let res = self
            .data
            .clone()
            .iter()
            .map(|d| self.send_request_and_compare_response(d))
            .collect();
        self.kill_server();
        res
    }
}
