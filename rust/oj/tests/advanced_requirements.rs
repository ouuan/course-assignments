mod common;
use common::TestCase;
use std::collections::BTreeMap;

#[test]
fn test_adv_01_10_pts_contest_support() {
    // check contest support
    // 1. create two contests
    // 2. check user-contest relation
    // 3. check ranklist for each contest
    TestCase::read("adv_01_contest_support").run();
}

#[test]
fn test_adv_02_10_pts_persistent_storage() {
    // check persistent storage
    // 1. create job
    // 2. restart server
    // 3. query job list
    TestCase::read("adv_02_persistent_storage").run();
}

#[test]
fn test_adv_03_10_pts_non_blocking_judging() {
    // send multiple jobs with sleep in the code, then poll each of them simultaneously
    // check:
    // 1. all submissions must return immediately (poll_for_job = false)
    // 2. all jobs must be finished afterwards (poll_for_job = true)
    TestCase::read("adv_03_nonblocking_judging").run();
}

#[test]
fn test_adv_04_10_pts_resource_limit() {
    // check that memory usage is not 0
    let results = TestCase::read("adv_04_01_report_memory_usage").run();
    assert_eq!(
        results.len(),
        1,
        "case adv_04_01_report_memory_usage incorrect"
    );
    assert!(
        results[0].as_object().unwrap()["cases"].as_array().unwrap()[1]
            .as_object()
            .unwrap()["memory"]
            .as_u64()
            .unwrap()
            > 0,
        "case adv_04_01_report_memory_usage incorrect: memory usage should be greater than 0"
    );

    // limit the memory usage to 10MB, allocate 40MB memory in submission, then check that the job result is MLE
    TestCase::read("adv_04_02_limit_memory_usage").run();
}

#[test]
fn test_adv_05_5_pts_packed_judging() {
    // check that packed judging is supported (cases in groups are skipped)
    TestCase::read("adv_05_packed_judging").run();
}

#[test]
fn test_adv_06_10_pts_special_judge() {
    // check that special judge is supported
    // use a Python script to compare float numbers with tolerance
    TestCase::read("adv_06_special_judge").run();
}

#[test]
fn test_adv_07_10_pts_dynamic_ranking() {
    // check that dynamic ranking is supported
    let results = TestCase::read("adv_07_dynamic_ranking").run();
    assert_eq!(
        results.len(),
        6,
        "case test_10_pts_dynamic_ranking incorrect"
    );

    // this is a overly simplified version of dynamic ranking
    let mut min_time = BTreeMap::from([(0, 0), (1, 0), (2, 0)]);

    for result in &results[2..5] {
        let user_id = result.as_object().unwrap()["submission"]
            .as_object()
            .unwrap()["user_id"]
            .as_u64()
            .unwrap();
        let time = result.as_object().unwrap()["cases"].as_array().unwrap()[1]
            .as_object()
            .unwrap()["time"]
            .as_u64()
            .unwrap();
        min_time.insert(user_id, time);
    }

    let min_time = *min_time.values().min().unwrap();

    let mut scores = BTreeMap::from([(0, 0.0), (1, 0.0), (2, 0.0)]);
    let dynamic_ranking_ratio = 0.5;

    for result in &results[2..5] {
        let user_id = result.as_object().unwrap()["submission"]
            .as_object()
            .unwrap()["user_id"]
            .as_u64()
            .unwrap();
        let time = result.as_object().unwrap()["cases"].as_array().unwrap()[1]
            .as_object()
            .unwrap()["time"]
            .as_u64()
            .unwrap();
        let score = 100.0 * (1.0 - dynamic_ranking_ratio)
            + 100.0 * min_time as f64 / time as f64 * dynamic_ranking_ratio;
        scores.insert(user_id, score);
    }

    for ranking in results[5].as_array().unwrap() {
        let user_id = ranking.as_object().unwrap()["user"].as_object().unwrap()["id"]
            .as_u64()
            .unwrap();
        let score = ranking.as_object().unwrap()["scores"].as_array().unwrap()[0]
            .as_f64()
            .unwrap();
        assert!(
            f64::abs((score - scores[&user_id]) / scores[&user_id]) < 1e-3,
            "case test_10_pts_dynamic_ranking incorrect"
        );
    }
}
