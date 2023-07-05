CREATE TABLE users (
    id INT NOT NULL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);
INSERT INTO users (id, name) VALUES (0, 'root');

CREATE TABLE contests (
    id INT NOT NULL PRIMARY KEY,
    name TEXT NOT NULL,
    "from" TIMESTAMP NOT NULL,
    "to" TIMESTAMP NOT NULL,
    submission_limit INT NOT NULL
);

CREATE TABLE contest_problems (
    rowid INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    contest_id INT NOT NULL,
    problem_id INT NOT NULL,
    FOREIGN KEY (contest_id) REFERENCES contests(id),
    UNIQUE (contest_id, problem_id)
);

CREATE TABLE contest_users (
    rowid INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    contest_id INT NOT NULL,
    user_id INT NOT NULL,
    FOREIGN KEY (contest_id) REFERENCES contests(id),
    FOREIGN KEY (user_id) REFERENCES users(id),
    UNIQUE (contest_id, user_id)
);

CREATE TABLE jobs (
    id INT NOT NULL PRIMARY KEY,
    created_time TIMESTAMP NOT NULL,
    updated_time TIMESTAMP NOT NULL,
    source_code TEXT NOT NULL,
    language TEXT NOT NULL,
    user_id INT NOT NULL,
    contest_id INT NOT NULL,
    problem_id INT NOT NULL,
    state TEXT NOT NULL CHECK(state IN ('Queueing', 'Running', 'Finished', 'Canceled')),
    result TEXT NOT NULL CHECK(result IN ('Waiting', 'Running', 'Accepted', 'Compilation Error', 'Compilation Success', 'Wrong Answer', 'Runtime Error', 'Time Limit Exceeded', 'Memory Limit Exceeded', 'System Error', 'SPJ Error', 'Skipped')),
    score DOUBLE NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
    -- contest_id is not foreign key because it can be zero
);

CREATE TABLE case_results (
    job_id INT NOT NULL,
    id INT NOT NULL,
    result TEXT NOT NULL CHECK(result IN ('Waiting', 'Running', 'Accepted', 'Compilation Error', 'Compilation Success', 'Wrong Answer', 'Runtime Error', 'Time Limit Exceeded', 'Memory Limit Exceeded', 'System Error', 'SPJ Error', 'Skipped')),
    time UNSIGNED BIG INT NOT NULL,
    memory UNSIGNED BIG INT NOT NULL,
    info TEXT NOT NULL,
    PRIMARY KEY (job_id, id),
    FOREIGN KEY (job_id) REFERENCES jobs(id)
);
