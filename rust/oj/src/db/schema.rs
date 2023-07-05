// @generated automatically by Diesel CLI.

diesel::table! {
    case_results (job_id, id) {
        job_id -> Integer,
        id -> Integer,
        result -> crate::db::enums::JobResultMapping,
        time -> BigInt,
        memory -> BigInt,
        info -> Text,
    }
}

diesel::table! {
    contest_problems (rowid) {
        rowid -> Integer,
        contest_id -> Integer,
        problem_id -> Integer,
    }
}

diesel::table! {
    contest_users (rowid) {
        rowid -> Integer,
        contest_id -> Integer,
        user_id -> Integer,
    }
}

diesel::table! {
    contests (id) {
        id -> Integer,
        name -> Text,
        from -> Timestamp,
        to -> Timestamp,
        submission_limit -> Integer,
    }
}

diesel::table! {
    jobs (id) {
        id -> Integer,
        created_time -> Timestamp,
        updated_time -> Timestamp,
        source_code -> Text,
        language -> Text,
        user_id -> Integer,
        contest_id -> Integer,
        problem_id -> Integer,
        state -> crate::db::enums::JobStateMapping,
        result -> crate::db::enums::JobResultMapping,
        score -> Double,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        name -> Text,
    }
}

diesel::joinable!(case_results -> jobs (job_id));
diesel::joinable!(contest_problems -> contests (contest_id));
diesel::joinable!(contest_users -> contests (contest_id));
diesel::joinable!(contest_users -> users (user_id));
diesel::joinable!(jobs -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    case_results,
    contest_problems,
    contest_users,
    contests,
    jobs,
    users,
);
