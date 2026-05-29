use crate::adapters::types::AgentTask;

pub fn sort_tasks_by_updated_at(mut tasks: Vec<AgentTask>) -> Vec<AgentTask> {
    tasks.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    tasks
}
