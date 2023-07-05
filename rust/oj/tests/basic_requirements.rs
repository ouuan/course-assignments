mod common;
use common::TestCase;

#[test]
fn test_01_15_pts_basic_judging() {
    // check basic judging with one job and one case
    TestCase::read("01_01_hello_world").run();
    // wrong results (WA, RE, CE)
    TestCase::read("01_02_wrong_results").run();
    // a job with multiple cases
    TestCase::read("01_03_multiple_cases").run();
    // a job that TLE (must be killed)
    TestCase::read("01_04_time_limit_exceeded").run();
    // strict compare mode
    TestCase::read("01_05_strict_compare").run();
}

#[test]
fn test_02_5_pts_multiple_language_support() {
    // check language support (e.g. C/C++)
    TestCase::read("02_01_cpp_support").run();
}

#[test]
fn test_03_10_pts_job_list() {
    // check job list support
    TestCase::read("03_01_job_list").run();
    TestCase::read("03_02_job_list_with_filter").run();
    TestCase::read("03_03_rejudging").run();
}

#[test]
fn test_04_5_pts_user_support() {
    // check user support
    TestCase::read("04_01_user_support").run();
}

#[test]
fn test_05_5_pts_ranklist_support() {
    // check global ranklist support after several submissions
    TestCase::read("05_01_global_ranklist").run();
    // check different scoring rule
    TestCase::read("05_02_scoring_rule").run();
    // check different tie breaker
    TestCase::read("05_03_tie_breaker").run();
}
