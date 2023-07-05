//! A judger worker runs at most a single job at the same time

use super::TMP_DIR;
use crate::config::{Config, ProblemType};
use crate::db::case_results::{self, CaseUpdate};
use crate::db::connection::ConnectionPool;
use crate::db::enums::JobResult;
use crate::db::jobs;
use crate::error::*;
use std::env::consts::EXE_EXTENSION;
use std::process::{Output, Stdio};
use std::time::{Duration, Instant};
use tokio::fs::{self, OpenOptions};
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio::time;

const COMPILE_TIME_LIMIT: Duration = Duration::from_secs(60);
const SPJ_TIME_LIMIT: Duration = Duration::from_secs(60);
const WAIT_EXTRA_DURATION: Duration = Duration::from_secs(1);

pub struct Worker {
    pub config: Config,
    pub job_receiver: async_channel::Receiver<i32>,
    /// It never sends anything,
    /// See <https://tokio.rs/tokio/topics/shutdown#waiting-for-things-to-finish-shutting-down>
    pub finished_sender: mpsc::Sender<()>,
    pub pool: ConnectionPool,
}

impl Worker {
    pub async fn work(self) {
        while let Ok(job_id) = self.job_receiver.recv().await {
            if self.job_receiver.is_closed() {
                break;
            }
            match self.do_job(job_id).await {
                Err(error) => {
                    log::error!("Judger error: (job_id = {}) {:?}", job_id, error);
                    jobs::finish_job(job_id, &JobResult::SystemError, 0.0, &self.pool).ok();
                }
                Ok(true) => log::info!("Job finished: {}", job_id),
                Ok(false) => log::info!("Job skipped: {}", job_id),
            }
        }
        log::info!("Worker stopped");
    }

    /// Do a job and returns whether the job is actually done instead of skipped on success.
    async fn do_job(&self, job_id: i32) -> ApiResult<bool> {
        // get job info
        let info = match jobs::fetch_job_for_judger(job_id, &self.pool)? {
            Some(info) => info,
            None => return Ok(false), // job is canceled
        };
        log::info!("Job received: {}", job_id);
        let language = self
            .config
            .language_map
            .get(&info.language)
            .ok_or_else(|| {
                ApiError::new(
                    ApiErrorType::Internal,
                    format!("Unknown language {}", info.language),
                )
            })?;
        let problem = self
            .config
            .problem_map
            .get(&info.problem_id)
            .ok_or_else(|| {
                ApiError::new(
                    ApiErrorType::Internal,
                    format!("Unknown problem {}", info.problem_id),
                )
            })?;

        // create temporary directory
        fs::create_dir_all(TMP_DIR).await?;
        let tmp_dir = tempfile::tempdir_in(TMP_DIR)?;

        // compile
        case_results::update_case(
            job_id,
            0,
            &CaseUpdate {
                result: JobResult::Running,
                time: 0,
                info: String::new(),
            },
            0.0,
            &self.pool,
        )?;
        let source_file_path = tmp_dir.path().join(&language.file_name);
        fs::write(&source_file_path, info.source_code).await?;
        let exe_file_path = tmp_dir
            .path()
            .join(format!("oj-solution-{}", job_id))
            .with_extension(EXE_EXTENSION);
        let compilation_status = Command::new(&language.command)
            .args(language.args.iter().map(|arg| match arg.as_str() {
                "%INPUT%" => source_file_path.as_os_str(),
                "%OUTPUT%" => exe_file_path.as_os_str(),
                arg => arg.as_ref(),
            }))
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .stdout(Stdio::null())
            .kill_on_drop(true)
            .status();
        let compilation_start = Instant::now();
        let compilation_success = match time::timeout(COMPILE_TIME_LIMIT, compilation_status).await
        {
            Ok(Ok(status)) if status.success() => true,
            _ => false,
        };
        let compilation_time = compilation_start.elapsed().as_micros();
        let compilation_result = CaseUpdate {
            result: if compilation_success {
                JobResult::CompilationSuccess
            } else {
                JobResult::CompilationError
            },
            time: compilation_time as i64,
            info: String::new(),
        };
        case_results::update_case(job_id, 0, &compilation_result, 0.0, &self.pool)?;
        if !compilation_success {
            jobs::finish_job(job_id, &JobResult::CompilationError, 0.0, &self.pool)?;
            return Ok(true);
        }

        // run on test cases
        let mut total_score = 0.0;
        let mut job_result = JobResult::Accepted;
        for subtask in &problem.packing {
            let mut subtask_skipped = false;
            let mut subtask_score = 0.0;
            for case_id in subtask.iter().cloned() {
                // handle skipped
                if subtask_skipped {
                    case_results::update_case(
                        job_id,
                        case_id + 1,
                        &CaseUpdate {
                            result: JobResult::Skipped,
                            time: 0,
                            info: String::new(),
                        },
                        total_score,
                        &self.pool,
                    )?;
                    continue;
                }

                // set state to running
                case_results::update_case(
                    job_id,
                    case_id + 1,
                    &CaseUpdate {
                        result: JobResult::Running,
                        time: 0,
                        info: String::new(),
                    },
                    total_score,
                    &self.pool,
                )?;

                let case = &problem.cases[case_id as usize];

                // get stdin and stdout for the child
                let input_file = OpenOptions::new()
                    .read(true)
                    .open(&case.input_file)
                    .await?
                    .into_std()
                    .await;
                // use file for spj and pipe for others
                let spj_output_path = tmp_dir.path().join(format!("{}-{}.out", job_id, case_id));
                let solution_stdout = match problem.tp {
                    ProblemType::Spj { .. } => OpenOptions::new()
                        .create_new(true)
                        .write(true)
                        .open(&spj_output_path)
                        .await?
                        .into_std()
                        .await
                        .into(),
                    _ => Stdio::piped(),
                };

                // construct solution command
                let mut solution_command = Command::new(&exe_file_path);
                solution_command
                    .stdin(input_file)
                    .stdout(solution_stdout)
                    .stderr(Stdio::null())
                    .kill_on_drop(true);

                // get judge result
                let case_update = match Self::run_solution(solution_command, case.time_limit).await
                {
                    Ok((solution_output, solution_time)) => {
                        let mut info = String::new();
                        let result = match &problem.tp {
                            // check answer
                            ProblemType::Spj { command, args } => {
                                let spj_output = Command::new(command)
                                    .args(args.into_iter().map(|arg| match arg.as_str() {
                                        "%OUTPUT%" => spj_output_path.as_os_str(),
                                        "%ANSWER%" => case.answer_file.as_ref(),
                                        arg => arg.as_ref(),
                                    }))
                                    .kill_on_drop(true)
                                    .output();
                                match time::timeout(SPJ_TIME_LIMIT, spj_output).await {
                                    Ok(Ok(spj_output)) if spj_output.status.success() => {
                                        match String::from_utf8(spj_output.stdout) {
                                            Err(_) => JobResult::SPJError,
                                            Ok(spj_output) => {
                                                let mut lines = spj_output.lines();
                                                let result = match lines.next() {
                                                    None => JobResult::SPJError,
                                                    Some(result) => {
                                                        if result == "Accepted" {
                                                            JobResult::Accepted
                                                        } else {
                                                            JobResult::WrongAnswer
                                                        }
                                                    }
                                                };
                                                if let Some(spj_info) = lines.next() {
                                                    info = String::from(spj_info);
                                                }
                                                result
                                            }
                                        }
                                    }
                                    _ => JobResult::SPJError,
                                }
                            }
                            tp => {
                                let answer = fs::read_to_string(&case.answer_file).await?;
                                let correct = match String::from_utf8(solution_output.stdout) {
                                    Err(_) => false,
                                    Ok(output) => match tp {
                                        ProblemType::Strict => output == answer,
                                        _ => Self::as_standard_check_iter(&output)
                                            .eq(Self::as_standard_check_iter(&answer)),
                                    },
                                };
                                if correct {
                                    JobResult::Accepted
                                } else {
                                    JobResult::WrongAnswer
                                }
                            }
                        };
                        CaseUpdate {
                            result,
                            time: solution_time.as_micros() as i64,
                            info,
                        }
                    }
                    Err((result, solution_time)) => CaseUpdate {
                        result,
                        time: solution_time.as_micros() as i64,
                        info: String::new(),
                    },
                };

                // update case result in database
                case_results::update_case(
                    job_id,
                    case_id + 1,
                    &case_update,
                    total_score,
                    &self.pool,
                )?;

                if case_update.result == JobResult::Accepted {
                    let ratio = match problem.tp {
                        ProblemType::DynamicRanking { ratio } => 1.0 - ratio,
                        _ => 1.0,
                    };
                    subtask_score += case.score * ratio;
                } else {
                    subtask_score = 0.0;
                    subtask_skipped = true;
                    if job_result == JobResult::Accepted {
                        job_result = case_update.result;
                    }
                }
            }
            total_score += subtask_score;
        }

        jobs::finish_job(job_id, &job_result, total_score, &self.pool)?;

        tmp_dir.close()?;

        Ok(true)
    }

    /// Run the command which is a compiled solution.
    /// Return (output, time) on success.
    /// Return (result, time) on failure.
    async fn run_solution(
        mut command: Command,
        time_limit: Duration,
    ) -> Result<(Output, Duration), (JobResult, Duration)> {
        let run_start = Instant::now();
        let child = command
            .spawn()
            .map_err(|_| (JobResult::RuntimeError, run_start.elapsed()))?;
        let output_timeout = time::timeout(
            time_limit.saturating_add(WAIT_EXTRA_DURATION),
            child.wait_with_output(),
        )
        .await;
        let time = run_start.elapsed();
        match output_timeout {
            Ok(output_result) if time <= time_limit => match output_result {
                Ok(output) if output.status.success() => Ok((output, time)),
                _ => Err((JobResult::RuntimeError, time)),
            },
            _ => Err((JobResult::TimeLimitExceeded, time)),
        }
    }

    /// Compare two outputs in standard mode by comparing the return value of this function
    fn as_standard_check_iter(output: &str) -> impl Iterator<Item = &str> {
        output
            .lines()
            .map(|line| line.trim_end())
            .rev()
            .skip_while(|line| line.is_empty())
    }
}
