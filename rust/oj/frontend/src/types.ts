export interface Job {
  id: number,
  created_time: string,
  updated_time: string,
  submission: {
    source_code: string,
    language: string,
    user_id: number,
    contest_id: number,
    problem_id: number,
  },
  state: string,
  result: string,
  score: number,
  cases: Array<{
    id: number,
    result: string,
    time: number,
    memory: number,
    info: string,
  }>,
}
