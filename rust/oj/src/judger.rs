//! Distribute and run jobs.

mod worker;

use crate::config::Config;
use crate::db::connection::ConnectionPool;
use crate::error::ApiResult;
use std::time::Duration;
use tokio::sync::mpsc::{self, error::TryRecvError};
use tokio::time;
use tokio::{fs, signal};
use worker::Worker;

const TMP_DIR: &str = "tmp/judger";

/// Add jobs in the job queue to be received by `Worker`s.
#[derive(Clone)]
pub struct JobAdder {
    job_sender: async_channel::Sender<i32>,
}

impl JobAdder {
    pub fn add_job(&self, id: i32) -> ApiResult<()> {
        self.job_sender.send_blocking(id)?;
        Ok(())
    }
}

/// Gracefully wait for unfinished jobs.
/// See <https://tokio.rs/tokio/topics/shutdown#waiting-for-things-to-finish-shutting-down>
pub struct JobWaiter {
    job_sender: async_channel::Sender<i32>,
    finished_receiver: mpsc::Receiver<()>,
}

impl JobWaiter {
    pub async fn wait(mut self) {
        self.job_sender.close();
        time::sleep(Duration::from_millis(10)).await;
        match self.finished_receiver.try_recv() {
            Err(TryRecvError::Disconnected) => {}
            _ => {
                eprintln!("Waiting for judgers to finish... Press Ctrl+C to forcefully exit.");
                tokio::select! {
                    _ = self.finished_receiver.recv() => {
                        eprintln!("Judgers finished.");
                    },
                    _ = signal::ctrl_c() => {
                        eprintln!("Ctrl+C received. Forcefully exiting.");
                    },
                }
            }
        }
        // TMP_DIR should be empty when gracefully exiting, but remove it to be safe
        match fs::remove_dir_all(TMP_DIR).await {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            result => result.expect(&format!(
                "failed to remove temporary directory [{}]",
                TMP_DIR
            )),
        }
    }
}

/// Spawn `Worker`s and return `JobAdder` to add jobs and `JobWaiter` to wait for `Worker`s to
/// finish.
pub fn create_judger(pool: ConnectionPool, config: &Config) -> (JobAdder, JobWaiter) {
    let (job_sender, job_receiver) = async_channel::unbounded();
    let (finished_sender, finished_receiver) = mpsc::channel(1);

    let worker_count = (num_cpus::get() / 2).max(1);

    log::info!("Starting {} workers", worker_count);

    // spawn workers in new tasks
    for _ in 0..worker_count {
        let worker = Worker {
            config: config.clone(),
            job_receiver: job_receiver.clone(),
            finished_sender: finished_sender.clone(),
            pool: pool.clone(),
        };
        tokio::spawn(worker.work());
    }

    // Add unfinished jobs to the queue at startup
    for job_id in
        crate::db::jobs::get_unfinished_jobs(&pool).expect("failed to get unfinished jobs")
    {
        job_sender
            .send_blocking(job_id)
            .expect("failed to add job in the queue");
    }

    (
        JobAdder {
            job_sender: job_sender.clone(),
        },
        JobWaiter {
            job_sender,
            finished_receiver,
        },
    )
}
