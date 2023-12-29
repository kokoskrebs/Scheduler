use std::cmp::max;
use std::collections::HashMap;

fn main() {
    println!("Hello, world!");

    //TODO Get the list of tasks from the parser
    //Go through the tasks and schedule each of them based on priority
        //Iterate through the calendars until they are all returning the same compatible start interval
        //Then schedule the task on that interval in each calendar
        //Unless it can't be scheduled, then we mark it as scheduled without putting in starting/ending intervals
}

struct TaskList {
    sequential_tasks: Vec<Task>,
    task_map: HashMap<String, usize>,
}

impl TaskList {
    fn iter(&self) -> impl Iterator<Item=&Task> {
        self.sequential_tasks.iter()
    }

    fn iter_mut(&mut self) -> impl Iterator<Item=&mut Task> {
        self.sequential_tasks.iter_mut()
    }

    fn get_by_key(&self, key: &str) -> Task {
        todo!()
    }

    fn get_by_index(&self, index: usize) -> Task {
        todo!()
    }

    fn add_task(&mut self, task: Task) {
        self.task_map.insert(task.task_id.clone(), self.sequential_tasks.len());
        self.sequential_tasks.push(task);
    }

    fn verify_well_formed(&self) -> bool {
        todo!()
    }
}

#[derive(Debug, Clone)]
struct Task {
    task_id: String,
    priority: u32,
    earliest_start_interval: u32,
    predecessors: Option<Vec<String>>,
    resource_needs: Option<HashMap<String, u16>>,
    configuration: Option<Configuration>,
    du: u32,
    scheduled: bool,
    start_interval: Option<u32>,
    complete_interval: Option<u32>,
}

#[derive(Debug, Clone)]
struct Configuration {
    statuses: Vec<String>,
}

// struct LatchingStatus {
//     status: String,
//     unlatching_task: String,
// }

trait ConstraintManager {
    fn schedule(&mut self, task: &Task, interval: u32);
    fn get_first_fit(&self, task: &Task, starting_interval: u32) -> Option<u32>;
}
#[derive(Debug)]
struct ResourceManager {
    calendar: HashMap<String, ResourceIntervals>,
    max_interval: u32,
}

impl ConstraintManager for ResourceManager {
    fn schedule(&mut self, task: &Task, interval: u32) {
        let Some(resources) = &task.resource_needs else { return; };

        for (resource, quantity) in resources.iter() {
            let Some(resource_intervals) = self.calendar.get_mut(resource) else { return; };
            resource_intervals.deduct(interval, task.du, *quantity);
        }
    }

    fn get_first_fit(&self, task: &Task, starting_interval: u32) -> Option<u32> {
        let mut i = max(starting_interval, task.earliest_start_interval);

        let Some(resources) = &task.resource_needs else { return Some(i) };

        while i + task.du - 1 <= self.max_interval {
            let mut iteration_start = i;
            for (resource, quantity) in resources.iter() {
                let Some(resource_intervals) = self.calendar.get(resource) else { return None; };
                let Some(resource_start) = resource_intervals.get_first_interval(*quantity, iteration_start, task.du) else { return None };

                iteration_start = max(iteration_start, resource_start);
            }

            if i == iteration_start {
                return Some(i);
            } else {
                i = iteration_start;
            }
        }

        None
    }
}

struct ConfigurationManager {
    config_intervals: Vec<Configuration>,
    status_relations: StatusRelationChart,
}

impl ConstraintManager for ConfigurationManager {
    fn schedule(&mut self, task: &Task, interval: u32) {
        let Some(task_config) = &task.configuration else { return; };

        for i in 0..task.du {
            for status in &task_config.statuses {
                if !self.config_intervals[(interval + i) as usize].statuses.contains(&status) {
                    self.config_intervals[(interval + i) as usize].statuses.push(status.clone());
                }
            }
        }
    }

    fn get_first_fit(&self, task: &Task, starting_interval: u32) -> Option<u32> {
        let starting_interval = max(starting_interval, task.earliest_start_interval);
        let Some(task_config) = &task.configuration else { return Some(starting_interval); };

        let mut last_incompat_interval = -1;
        let mut last_compat_interval = -1;

        for (i, interval) in self.config_intervals[starting_interval as usize..self.config_intervals.len()].iter().enumerate() {
            if !self.status_relations.get_config_compatibility(interval, task_config) {
                last_incompat_interval = i as i32;
                continue;
            }

            last_compat_interval = i as i32;

            if last_incompat_interval < last_compat_interval && (last_compat_interval - last_incompat_interval) as u32 == task.du {
                return Some((starting_interval as i32 + last_incompat_interval + 1) as u32);
            }
        }

        None
    }
}

#[derive(Debug, Clone)]
struct ResourceIntervals {
    first_available_interval: u32,
    intervals: Vec<u16>
}

impl ResourceIntervals {
    fn get_first_interval(&self, quantity: u16, starting_interval: u32, du: u32) -> Option<u32> {
        let mut starting_interval = max(starting_interval, self.first_available_interval) as usize;
        let interval_slice = &self.intervals[starting_interval..self.intervals.len()];

        let mut last_incompat_interval = -1;
        let mut last_compat_interval = -1;

        for (i, &available) in interval_slice.iter().enumerate() {
            if available < quantity {
                last_incompat_interval = i as i32;
                continue;
            }

            last_compat_interval = i as i32;

            if last_incompat_interval < last_compat_interval && (last_compat_interval - last_incompat_interval) as u32 == du {
                return Some((last_incompat_interval + starting_interval as i32 + 1) as u32);
            }
        }

        None
    }

    fn deduct(&mut self, interval: u32, du: u32, quantity: u16) {
        for i in interval..interval + du {
            let Some(interval_quantity) = self.intervals.get(i as usize) else { return; };
            self.intervals[i as usize] = interval_quantity - quantity;
        }
    }

    // fn satisfies(&self, resource_needs: &Option<ResourceInterval>) -> bool {
    //     let Some(resource_needs) = resource_needs else { return true; };
    //
    //     for (resource, quantity) in resource_needs.resources.iter() {
    //         match self.resources.get(resource) {
    //             Some(res_avail) if res_avail >= quantity => (),
    //             _ => return false,
    //         }
    //     }
    //
    //     true
    // }
}

#[derive(Debug)]
struct StatusRelationChart {
    status_relations: HashMap<(String, String), bool>,
}

impl StatusRelationChart {
    fn new() -> StatusRelationChart {
        StatusRelationChart { status_relations: HashMap::new() }
    }

    fn insert_relation(&mut self, status_one: &str, status_two: &str, relation: bool) {
        self.status_relations.insert((status_one.to_string(), status_two.to_string()), relation);
        self.status_relations.insert((status_two.to_string(), status_one.to_string()), relation);
    }

    fn get_relation(&self, status_one: &str, status_two: &str) -> Option<bool> {
        self.status_relations.get(&(status_one.to_string(), status_two.to_string())).copied()
    }

    fn get_config_compatibility(&self, list_one: &Configuration, list_two: &Configuration) -> bool {
        for status_one in list_one.statuses.iter() {
            for status_two in list_two.statuses.iter() {
                if self.get_relation(status_one, status_two) == Some(false) {
                    return false;
                }
            }
        }

        true
    }

    fn from_tsv (tsv: String) -> Self {
        todo!()
    }
}

// enum StatusRelation {
//     Compatible,
//     Incompatible,
// }

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn task_without_needs() {
        let mut calendar = ResourceManager{
            calendar: HashMap::new(),
            max_interval: 1,
        };

        let mut resource_interval = ResourceManager{
            calendar: HashMap::new(),
            max_interval: 1,
        };

        calendar.calendar.insert("Electrician".to_string(), ResourceIntervals{
            first_available_interval: 0,
            intervals: vec![2, 2],
        });

        calendar.calendar.insert("Mechanic".to_string(), ResourceIntervals{
            first_available_interval: 0,
            intervals: vec![2, 2],
        });

        let mut test_task = Task {
            task_id: "task_id".to_string(),
            priority: 1,
            earliest_start_interval: 0,
            predecessors: None,
            resource_needs: None,
            configuration: None,
            du: 1,
            scheduled: false,
            start_interval: None,
            complete_interval: None,
        };

        assert_eq!(Some(0), calendar.get_first_fit(&mut test_task, 0));
        assert_eq!(Some(1), calendar.get_first_fit(&mut test_task, 1));
    }

    #[test]
    fn tasks_with_needs() {
        let mut calendar = ResourceManager{
            calendar: HashMap::new(),
            max_interval: 1,
        };

        let mut needs = HashMap::new();
        needs.insert("Electrician".to_string(), 2);
        needs.insert("Mechanic".to_string(), 2);

        calendar.calendar.insert("Electrician".to_string(), ResourceIntervals{
            first_available_interval: 0,
            intervals: vec![2, 2],
        });

        calendar.calendar.insert("Mechanic".to_string(), ResourceIntervals{
            first_available_interval: 0,
            intervals: vec![2, 2],
        });

        let mut test_task1 = Task {
            task_id: "task_id1".to_string(),
            priority: 1,
            earliest_start_interval: 0,
            predecessors: None,
            resource_needs: Some(needs.clone()),
            configuration: None,
            du: 1,
            scheduled: false,
            start_interval: None,
            complete_interval: None,
        };

        let mut test_task2 = Task {
            task_id: "task_id2".to_string(),
            priority: 1,
            earliest_start_interval: 0,
            predecessors: None,
            resource_needs: Some(needs),
            configuration: None,
            du: 1,
            scheduled: false,
            start_interval: None,
            complete_interval: None,
        };

        assert_eq!(Some(0), calendar.get_first_fit(&mut test_task1, 0));
        assert_eq!(Some(1), calendar.get_first_fit(&mut test_task1, 1));
        calendar.schedule(&test_task1, 0);
        assert_eq!(Some(1), calendar.get_first_fit(&mut test_task2, 0));
        calendar.schedule(&test_task2, 1);

        //TODO Verify that the resource calendar properly discounted the resources used by the tasks.
    }

    #[test]
    fn too_needy_tasks() {
        let mut calendar = ResourceManager{
            calendar: HashMap::new(),
            max_interval: 1,
        };

        let mut needs = HashMap::new();
        needs.insert("Electrician".to_string(), 2);
        needs.insert("Mechanic".to_string(), 2);

        calendar.calendar.insert("Electrician".to_string(), ResourceIntervals{
            first_available_interval: 0,
            intervals: vec![1, 1],
        });

        calendar.calendar.insert("Mechanic".to_string(), ResourceIntervals{
            first_available_interval: 0,
            intervals: vec![1, 1],
        });

        let mut test_task = Task {
            task_id: "task_id1".to_string(),
            priority: 1,
            earliest_start_interval: 0,
            predecessors: None,
            resource_needs: Some(needs.clone()),
            configuration: None,
            du: 1,
            scheduled: false,
            start_interval: None,
            complete_interval: None,
        };

        assert_eq!(None, calendar.get_first_fit(&mut test_task, 0));
    }

    #[test]
    fn missing_needs_tasks() {
        let mut calendar = ResourceManager{
            calendar: HashMap::new(),
            max_interval: 1,
        };

        let mut needs = HashMap::new();
        needs.insert("Electrician".to_string(), 2);
        needs.insert("Mechanic".to_string(), 2);

        calendar.calendar.insert("Mechanic".to_string(), ResourceIntervals{
            first_available_interval: 0,
            intervals: vec![1, 1],
        });

        let mut test_task = Task {
            task_id: "task_id1".to_string(),
            priority: 1,
            earliest_start_interval: 0,
            predecessors: None,
            resource_needs: Some(needs.clone()),
            configuration: None,
            du: 1,
            scheduled: false,
            start_interval: None,
            complete_interval: None,
        };

        assert_eq!(None, calendar.get_first_fit(&mut test_task, 0));
    }

    //TODO Scheduling a task with configuration requirements
    #[test]
    fn tasks_with_config_reqs() {
        let mut chart = StatusRelationChart::new();
        chart.insert_relation("Hot", "Cold", false);

        assert_eq!(Some(false), chart.get_relation("Hot", "Cold"));
        assert_eq!(Some(false), chart.get_relation("Cold", "Hot"));
        assert_eq!(None, chart.get_relation("Hot", "Hot"));

        let mut config_calendar = ConfigurationManager {
            config_intervals: Vec::new(),
            status_relations: chart,
        };

        config_calendar.config_intervals.push(Configuration{statuses: Vec::new()});
        config_calendar.config_intervals.push(Configuration{statuses: Vec::new()});
        config_calendar.config_intervals.push(Configuration{statuses: Vec::new()});
        config_calendar.config_intervals.push(Configuration{statuses: Vec::new()});

        let mut test_task_one = Task {
            task_id: "task_id1".to_string(),
            priority: 1,
            earliest_start_interval: 0,
            predecessors: None,
            resource_needs: None,
            configuration: Some(Configuration{statuses: vec!["Hot".to_string()]}),
            du: 1,
            scheduled: false,
            start_interval: None,
            complete_interval: None,
        };

        let mut test_task_two = Task {
            task_id: "task_id2".to_string(),
            priority: 1,
            earliest_start_interval: 0,
            predecessors: None,
            resource_needs: None,
            configuration: Some(Configuration{statuses: vec!["Hot".to_string()]}),
            du: 1,
            scheduled: false,
            start_interval: None,
            complete_interval: None,
        };

        let mut test_task_three = Task {
            task_id: "task_id3".to_string(),
            priority: 1,
            earliest_start_interval: 0,
            predecessors: None,
            resource_needs: None,
            configuration: Some(Configuration{statuses: vec!["Cold".to_string()]}),
            du: 1,
            scheduled: false,
            start_interval: None,
            complete_interval: None,
        };

        assert_eq!(Some(0), config_calendar.get_first_fit(&test_task_one, 0));
        assert_eq!(Some(1), config_calendar.get_first_fit(&test_task_one, 1));
        assert_eq!(Some(0), config_calendar.get_first_fit(&test_task_two, 0));
        assert_eq!(Some(0), config_calendar.get_first_fit(&test_task_three, 0));

        config_calendar.schedule(&test_task_one, 0);

        assert_eq!(Some(0), config_calendar.get_first_fit(&test_task_two, 0));

        config_calendar.schedule(&test_task_two, 0);

        assert_eq!(Some(1), config_calendar.get_first_fit(&test_task_three, 0));

        config_calendar.schedule(&test_task_three, 1);

        assert!(config_calendar.config_intervals[0].statuses.contains(&"Hot".to_string()));
        assert!(config_calendar.config_intervals[1].statuses.contains(&"Cold".to_string()));
    }

    #[test]
    fn config_manager_different_early_start() {
        let mut chart = StatusRelationChart::new();
        chart.insert_relation("Hot", "Cold", false);

        assert_eq!(Some(false), chart.get_relation("Hot", "Cold"));
        assert_eq!(Some(false), chart.get_relation("Cold", "Hot"));
        assert_eq!(None, chart.get_relation("Hot", "Hot"));

        let mut config_calendar = ConfigurationManager {
            config_intervals: Vec::new(),
            status_relations: chart,
        };

        config_calendar.config_intervals.push(Configuration { statuses: Vec::new() });
        config_calendar.config_intervals.push(Configuration { statuses: Vec::new() });

        let mut test_task = Task {
            task_id: "task_id1".to_string(),
            priority: 1,
            earliest_start_interval: 1,
            predecessors: None,
            resource_needs: None,
            configuration: None,
            du: 1,
            scheduled: false,
            start_interval: None,
            complete_interval: None,
        };

        assert_eq!(Some(1), config_calendar.get_first_fit(&mut test_task, 0));
    }

    #[test]
    fn resource_manager_different_early_start() {
        let mut calendar = ResourceManager{
            calendar: HashMap::new(),
            max_interval: 1,
        };

        let mut test_task = Task {
            task_id: "task_id1".to_string(),
            priority: 1,
            earliest_start_interval: 1,
            predecessors: None,
            resource_needs: None,
            configuration: None,
            du: 1,
            scheduled: false,
            start_interval: None,
            complete_interval: None,
        };

        assert_eq!(Some(1), calendar.get_first_fit(&mut test_task, 0));
    }

    #[test]
    fn task_list_iter() {
        let mut task_list = TaskList{
            sequential_tasks: Vec::new(),
            task_map: HashMap::new(),
        };

        let mut test_task_one = Task {
            task_id: "task_id1".to_string(),
            priority: 1,
            earliest_start_interval: 0,
            predecessors: None,
            resource_needs: None,
            configuration: Some(Configuration{statuses: vec!["Hot".to_string()]}),
            du: 1,
            scheduled: false,
            start_interval: None,
            complete_interval: None,
        };

        let mut test_task_two = Task {
            task_id: "task_id2".to_string(),
            priority: 1,
            earliest_start_interval: 0,
            predecessors: None,
            resource_needs: None,
            configuration: Some(Configuration{statuses: vec!["Hot".to_string()]}),
            du: 1,
            scheduled: false,
            start_interval: None,
            complete_interval: None,
        };

        let mut test_task_three = Task {
            task_id: "task_id3".to_string(),
            priority: 1,
            earliest_start_interval: 0,
            predecessors: None,
            resource_needs: None,
            configuration: Some(Configuration{statuses: vec!["Cold".to_string()]}),
            du: 1,
            scheduled: false,
            start_interval: None,
            complete_interval: None,
        };

        task_list.add_task(test_task_one);
        task_list.add_task(test_task_two);
        task_list.add_task(test_task_three);

        let mut task_iter = task_list.iter();

        assert!(task_iter.next().unwrap().task_id == "task_id1");
        assert!(task_iter.next().unwrap().task_id == "task_id2");
        assert!(task_iter.next().unwrap().task_id == "task_id3");
    }

    #[test]
    fn task_with_itself_as_pred() {
        todo!()
    }

    #[test]
    fn task_adjacent_pred_loop () {
        todo!()
    }

    fn task_longer_pred_loop () {
        todo!()
    }
}



























